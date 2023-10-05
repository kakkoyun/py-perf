use log::{debug, error, info, trace};
use pprof::Symbol;

use std::collections::HashMap;
use std::os::fd::{AsFd, AsRawFd};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::thread::ScopedJoinHandle;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{fmt, thread};

use libbpf_rs::skel::{OpenSkel, SkelBuilder};
use libbpf_rs::{MapFlags, PerfBufferBuilder, ProgramType};

use anyhow::{bail, Context, Result};
use crossbeam::channel::{bounded, select, tick, unbounded, Receiver};
use plain::Plain;
use py_spy::version::Version;
use serde_yaml;

use crate::bindings;
use crate::bindings::{PythonVersionOffsets, PYPERF_STACK_WALKING_PROGRAM_IDX};
use crate::bpf::pyperf::{PyperfSkel, PyperfSkelBuilder};
use crate::perf_event;
use crate::process_info::ProcessInfo;
use crate::profile::Profile;
use crate::python_readers::any_as_u8_slice;
use crate::python_versions::PYTHON_VERSION_CONFIGS_YAML;

// TODO(kakkoyun): Matches this with error codes in the pyperf.h !!
#[derive(Default, Clone, Debug)]
pub struct Stats {
    pub total_events: u32,
    // Events discarded due to the kernel buffer being full.
    pub lost_event_errors: u32,
    // Failed to retrieve sample due to a failed read from a map.
    pub map_reading_errors: u32,
    // The stack is not complete.
    pub truncated_stacks: u32,
    // How many times have we bumped into garbled data.
    pub garbled_data_errors: u32,
}

impl Stats {
    #[must_use]
    pub const fn total_errors(&self) -> u32 {
        self.lost_event_errors
            + self.map_reading_errors
            + self.truncated_stacks
            + self.garbled_data_errors
    }

    #[must_use]
    pub const fn stack_errors(&self) -> u32 {
        self.map_reading_errors + self.truncated_stacks + self.garbled_data_errors
    }
}

impl fmt::Display for Stats {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f)?;
        writeln!(f, "total events: {}", self.total_events)?;
        writeln!(f, "total errors: {}", self.total_errors())?;
        writeln!(f, "lost event errors: {}", self.lost_event_errors)?;
        writeln!(f, "map reading errors: {}", self.map_reading_errors)?;
        writeln!(f, "truncated stacks: {}", self.truncated_stacks)?;
        writeln!(f, "garbled data errors: {}", self.garbled_data_errors)?;

        Ok(())
    }
}

unsafe impl Plain for PythonVersionOffsets {}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct SupportedVersion {
    idx: u32,
    version: Version,
    offsets: PythonVersionOffsets,
}

pub struct SupportedVersions {
    versions: HashMap<String, SupportedVersion>,
}

impl SupportedVersions {
    /// Returns a new instance of `SupportedVersions`.
    /// This function will read the supported Python versions from the `PYTHON_VERSION_CONFIGS_YAML`
    /// and populate the `versions` map.
    ///
    /// # Errors
    /// This function will return an error if the `PYTHON_VERSION_CONFIGS_YAML` is not valid YAML.
    pub fn new() -> Result<Self> {
        let versions = Self::read_supported_version_offsets()?;
        Ok(Self { versions })
    }

    #[must_use]
    pub fn get(&self, version: &Version) -> Option<&SupportedVersion> {
        let version_string = format!("python{}.{}", version.major, version.minor);
        self.versions.get(&version_string)
    }

    #[must_use]
    pub fn version_string(version: &Version) -> String {
        format!("python{}.{}", version.major, version.minor)
    }

    fn read_supported_version_offsets() -> Result<HashMap<String, SupportedVersion>> {
        let mut supported_python_versions: HashMap<String, SupportedVersion> = HashMap::new();
        for (i, python_version_config_yaml) in PYTHON_VERSION_CONFIGS_YAML.iter().enumerate() {
            let python_version_config: PythonVersionOffsets =
                serde_yaml::from_str(python_version_config_yaml)?;
            let v = Version {
                major: u64::from(python_version_config.major_version),
                minor: u64::from(python_version_config.minor_version),
                patch: u64::from(python_version_config.patch_version),
                // TODO(kakkoyun): Add release flags to the config file.
                release_flags: String::new(),
            };
            let version_string = Self::version_string(&v);
            supported_python_versions.insert(
                version_string,
                SupportedVersion {
                    idx: u32::try_from(i)?,
                    version: v.clone(),
                    offsets: python_version_config,
                },
            );
        }
        Ok(supported_python_versions)
    }
}

// TODO(kakkoyun): Consider renaming to profiler.
pub struct PyPerf<'a> {
    // TODO(kakkoyun): It's better to have a local version of this and return it in start(profile).
    pub stats: Arc<RwLock<Stats>>,

    frequency: u64,
    duration: Duration,
    started_at: Option<SystemTime>,

    supported_versions: SupportedVersions,
    processes: Vec<ProcessInfo>,

    bpf: PyperfSkel<'a>,
}

impl<'a> PyPerf<'a> {
    /// Returns a new instance of `PyPerf`.
    /// This function will open and load the BPF module.
    /// It will also populate the `process_info_map` with the given `pids`.
    ///
    /// # Errors
    /// This function will return an error if the BPF module fails to load.
    /// It will also return an error if the `process_info_map` fails to update.
    pub fn new(duration: Duration, frequency: u64) -> Result<PyPerf<'a>> {
        // Open and load the BPF module.
        let mut skel_builder = PyperfSkelBuilder::default();
        skel_builder.obj_builder.debug(true);

        let mut open_skel = skel_builder.open()?;

        debug!("verbose_bpf_logging set to {}", true);
        open_skel.rodata().verbose = true;

        for prog in open_skel.obj.progs_iter_mut() {
            prog.set_prog_type(ProgramType::PerfEvent);
        }

        let bpf = open_skel.load()?;
        for prog in bpf.obj.progs_iter() {
            debug!(
                "open prog: {} has {} instructions",
                prog.name(),
                prog.insn_cnt()
            );
        }

        let supported_versions = SupportedVersions::new()?;
        Ok(PyPerf {
            frequency,
            duration,

            started_at: None,
            supported_versions,

            bpf,
            processes: Vec::new(),
            stats: Arc::new(RwLock::new(Stats::default())),
        })
    }

    // TODO(kakkoyun): Rename to register?
    /// Start recording the samples for the given `pids`.
    ///
    /// # Errors
    /// This function will return an error if it fails to send the `pids` to the BPF space.
    pub fn record(&mut self, pid: i32) -> Result<()> {
        let process_info =
            ProcessInfo::new(pid).context(format!("failed to fetch process info: {pid}"))?;

        debug!("python process: \n{}", process_info);
        self.processes.push(process_info);

        // let children = process_info.children()?;
        // for child in children {
        //     debug!("python process: \n{}", child);
        //     self.processes.push(child);
        // }

        if self.processes.is_empty() {
            bail!("No Python processes found to profile!");
        }
        info!("found python processes: {}", self.processes.len());

        // let bpf = self.bpf.clone();
        // let mut bpf = bpf.write().unwrap();
        let mut maps = self.bpf.maps_mut();
        for proc in &self.processes {
            let offsets = match self.supported_versions.get(&proc.version) {
                Some(supported_version) => supported_version.offsets,
                None => bail!(format!("unsupported Python version: {}", proc.version)),
            };

            let py_version = u32::try_from(proc.version.major * 100 + proc.version.minor)?;
            let key = py_version.to_le_bytes();
            // let value = unsafe { any_as_u8_slice(&offsets) };
            let value = unsafe { plain::as_bytes(&offsets) };
            maps.version_specific_offsets()
                .update(&key, value, MapFlags::ANY)
                .context("failed to update version specific offsets map")?;

            let key = proc.pid.to_le_bytes();
            let bpf_proc_info = crate::bindings::ProcessInfo {
                thread_state_addr: proc.thread_state_address,
                interpreter_addr: proc.interpreter_address,
                py_version,
            };
            let value = unsafe { any_as_u8_slice(&bpf_proc_info) };
            maps.pid_to_process_info()
                .update(&key, value, MapFlags::ANY)
                .context("failed to update process info map")?;
        }
        Ok(())
    }

    // TODO(kakkoyun): Rename to profile?
    /// Start the profiler.
    /// This function will block until the profiler is stopped.
    /// The profiler can be stopped by sending a message to the `stop_channel_rx` channel.
    ///
    /// # Errors
    /// This function will return an error if the profiler fails to start.
    ///
    /// # Panics
    /// This function will panic if the profiler fails to attach the perf event.
    pub fn start(&mut self, stop_channel_rx: &Receiver<()>) -> Result<Profile> {
        if self.processes.is_empty() {
            bail!("No Python processes found to profile!");
        }
        info!("starting profiler");

        let pid = self.processes[0].pid;
        let mut fds = Vec::new();
        for i in 0..num_cpus::get() {
            // TODO(kakkoyun): Support multiple processes if there exists.
            let perf_fd = unsafe { perf_event::setup(i.try_into()?, self.frequency, Some(pid)) }?;
            fds.push(perf_fd);
        }

        // let bpf = self.bpf.clone();
        // let mut bpf = bpf.write().unwrap();
        let mut links = Vec::new();
        for fd in fds {
            let prog = self.bpf.obj.prog_mut("on_event").unwrap();
            let link = prog.attach_perf_event(fd)?;
            links.push(link);
        }

        for prog in self.bpf.obj.progs_iter_mut() {
            debug!(
                "program type: {}, name: {}, flags: {}, section: {}",
                prog.prog_type(),
                prog.name(),
                prog.flags(),
                prog.section()
            );
        }

        // Insert stack walking program.
        let idx: i32 = PYPERF_STACK_WALKING_PROGRAM_IDX.try_into().unwrap();
        let val = self
            .bpf
            .obj
            .prog("walk_python_stack")
            .unwrap()
            .as_fd()
            .as_raw_fd();

        // let bpf = self.bpf.clone();
        // let mut bpf = bpf.write().unwrap();
        let mut maps = self.bpf.maps_mut();
        let programs = maps.programs();
        programs
            .update(&idx.to_le_bytes(), &val.to_le_bytes(), MapFlags::ANY)
            .unwrap();

        debug!(
            "profiling duration: {}, frequency: {}",
            humantime::format_duration(self.duration),
            self.frequency
        );

        let (sender, receiver) = unbounded();

        let maps = self.bpf.maps();
        let events = maps.events();
        let stats = self.stats.clone();
        // let cb_sender = sender.clone();
        let perf_buffer = PerfBufferBuilder::new(events)
            .sample_cb(|cpu: i32, data: &[u8]| {
                trace!("received sample from cpu: {}", cpu);
                sender
                    .send((cpu, data.to_vec()))
                    .expect("could not send signal on channel.");
            })
            .lost_cb(|cpu: i32, count: u64| {
                trace!("lost {} events on CPU {}", count, cpu);
                handle_lost_events(stats.clone(), cpu, count);
            })
            .build()?;

        // TODO(kakkoyun): Enable ringbuffer
        // let ring_buffer = libbpf_rs::RingBufferBuilder::new()
        //     .add(events, |data: &[u8]| -> i32 {
        //         handle_event(0, data, &self.stats.clone());
        //         0
        //     })
        //     .unwrap()
        //     .build();

        self.started_at = Some(SystemTime::now());
        info!("profiler started recording...");

        let (done, stop) = bounded::<()>(1);
        let profile = thread::scope(|s| {
            let duration = self.duration;

            let processor: ScopedJoinHandle<Profile> = s.spawn({
                let duration = self.duration;
                let frequency = self.frequency;
                let started_at = self.started_at;
                let receiver: Receiver<(i32, Vec<u8>)> = receiver.clone();

                move || {
                    let mut profile = Profile::new(duration, frequency);
                    profile.start_time = started_at;

                    loop {
                        select! {
                            recv(receiver) -> sample  => match sample {
                                Ok((cpu, data)) => {
                                trace!("received sample from cpu: {}", cpu);
                                let mut sample = bindings::Sample::default();
                                plain::copy_from_bytes(&mut sample, &data[..])
                                    .expect("data buffer was too short");
                                self.handle_sample(self.stats.clone(), &mut profile, cpu, sample);
                                trace!("sample handled! Waiting for the next one...");
                                }
                                Err(_) => continue,
                            },
                            recv(stop) -> _ => {
                                debug!("stopping profiling...");
                                break;
                            }
                        }
                    }
                    debug!("sample processor is done!");
                    profile
                }
            });

            // let sender = sender.clone();
            let ticks = tick(duration);
            loop {
                select! {
                    recv(ticks) -> _ => {
                        debug!("TICK!");
                        if let Err(err) = perf_buffer.poll(Duration::from_millis(100)) {
                            debug!("polling perf buffer failed with {:?}", err);
                        }
                    }
                    recv(stop_channel_rx) -> _ => {
                        debug!("stopping profiling...");
                        drop(done);
                        break;
                    }
                }
            }
            debug!("profiling is stopped");
            processor.join().unwrap()
        });
        debug!("profiler is done!");

        let stats = stats.read().unwrap();
        info!("stats: {}", stats);

        Ok(profile)
    }

    // TODO(kakkoyun): Probably better than redundant data that we have.
    // fn populate_python_version_map(
    //     supported_versions: &HashMap<SupportedVersion, u32>,
    //     versions: &mut libbpf_rs::Map,
    // ) -> Result<()> {
    //     use crate::python_readers::any_as_u8_slice;
    //     for (version, i) in supported_versions.iter() {
    //         let key: u32 = i.clone();
    //         let value = unsafe { any_as_u8_slice(&version.offsets) };
    //         versions.update(&key.to_le_bytes(), value, MapFlags::ANY)?;
    //     }
    //     Ok(())
    // }

    fn handle_sample(
        &self,
        stats: Arc<RwLock<Stats>>,
        profile: &mut Profile,
        cpu: i32,
        raw_sample: bindings::Sample,
    ) {
        let stats = stats.clone();

        let maps = self.bpf.maps();
        let symbols = maps.symbols();
        let mut id_to_symbol = HashMap::new();
        for stack_bytes in symbols.keys() {
            match symbols.lookup(&stack_bytes, MapFlags::ANY) {
                Ok(Some(id_bytes)) => {
                    let mut symbol = bindings::Symbol::default();
                    plain::copy_from_bytes(&mut symbol, &stack_bytes)
                        .expect("data buffer was too short");
                    let id = u32::from_le_bytes(id_bytes.try_into().expect("parse frame id bytes"));
                    id_to_symbol.insert(id, symbol);
                }
                _ => continue,
            }
        }
        stats.write().unwrap().total_events += 1;

        let now = now_formatted();

        // TODO(kakkoyun): Check this could be used as thread_name!
        let comm_str = std::str::from_utf8(&raw_sample.comm)
            .unwrap()
            .trim_end_matches(char::from(0));
        // NOTICE: It's similar to str_from_u8_nul

        // if recv_stack.stack_status == ruby_stack_status_STACK_INCOMPLETE {
        //     error!("truncated stack");
        //     self.stats.truncated_stacks += 1;
        //     continue;
        // }

        // TODO(kakkoyun): Record as metric.
        assert!(raw_sample.pid != 0, "pid is zero, this should never happen");

        debug!(
            "cpu: {} received: {:9} pid: {:6} tid: {:<6} comm: {:<16} kernel: {} user: {}",
            cpu,
            now,
            raw_sample.pid,
            raw_sample.tid,
            comm_str,
            raw_sample.native_stack_count_key.kernel_stack_id,
            raw_sample.native_stack_count_key.user_stack_id
        );

        // let timestamp = UNIX_EPOCH + Duration::from_nanos(sample.timestamp);
        let timestamp = UNIX_EPOCH + Duration::from_secs(raw_sample.timestamp);

        // TODO(kakkoyun): Handle native stack!

        let stack = raw_sample.stack;
        let mut read_frame_count = 0;
        let mut frames: Vec<(String, String, String, u32)> = Vec::new();
        for symbol_id in &stack.frames {
            // Don't read past the last frame.
            if read_frame_count >= stack.len {
                break;
            }

            match id_to_symbol.get(symbol_id) {
                Some(symbol) => {
                    let file_bytes: Vec<u8> = symbol.file.iter().map(|&c| c as u8).collect();
                    let file_name = unsafe { str_from_u8_nul(&file_bytes) };
                    if file_name.is_err() {
                        stats.write().unwrap().garbled_data_errors += 1;
                        continue;
                    }
                    let file_name = file_name
                        .expect("file name should be valid unicode")
                        .to_string();

                    let class_bytes: Vec<u8> = symbol.class.iter().map(|&c| c as u8).collect();
                    let class_name = unsafe { str_from_u8_nul(&class_bytes) };
                    if class_name.is_err() {
                        stats.write().unwrap().garbled_data_errors += 1;
                        continue;
                    }
                    let class_name = class_name
                        .expect("class name should be valid unicode")
                        .to_string();

                    let func_bytes: Vec<u8> = symbol.func.iter().map(|&c| c as u8).collect();
                    let func_name = unsafe { str_from_u8_nul(&func_bytes) };
                    if func_name.is_err() {
                        stats.write().unwrap().garbled_data_errors += 1;
                        continue;
                    }
                    let func_name = func_name
                        .expect("function name should be valid unicode")
                        .to_string();

                    let line = symbol.line;

                    frames.push((file_name, class_name, func_name, line));
                    read_frame_count += 1;
                }
                None => {
                    stats.write().unwrap().map_reading_errors += 1;
                }
            }
        }

        // TODO(kakkoyun): Utilize weight. Aggregate in BPF and send.}
        let mut sample = Vec::new();
        for (file_name, class_name, func_name, line) in frames {
            // trace!(
            //     "file: {:<32} class: {:<32} func: {:<32} line: {:<4}",
            //     file_name,
            //     class_name,
            //     func_name,
            //     line
            // );
            sample.push(Symbol {
                name: Some(format!("{}::{}", class_name, func_name).into_bytes()),
                addr: None,
                lineno: Some(line),
                filename: Some(PathBuf::from(file_name)),
            });
        }
        profile.add_sample(raw_sample.tid as u64, timestamp, sample, 1)
    }
}

unsafe impl Plain for bindings::Sample {}
unsafe impl Plain for bindings::Symbol {}

fn handle_lost_events(stats: Arc<RwLock<Stats>>, cpu: i32, count: u64) {
    stats.write().unwrap().lost_event_errors += u32::try_from(count).unwrap();
    error!("lost {} events on CPU {}", count, cpu);
}

// unsafe impl Plain for pyperf_bss_types::event {}

// TODO(kakkoyun): Clean this up.
// fn handle_event(cpu: i32, data: &[u8], stats: &Arc<RwLock<Stats>>) {
//     stats.write().unwrap().total_events += 1;

//     let mut event = pyperf_bss_types::event::default();
//     plain::copy_from_bytes(&mut event, data).expect("data buffer was too short");

//     let now = now_formatted();
//     let comm_str = std::str::from_utf8(&event.comm)
//         .unwrap()
//         .trim_end_matches(char::from(0));
//     let msg_str = std::str::from_utf8(&event.msg)
//         .unwrap()
//         .trim_end_matches(char::from(0));

//     info!(
//         "cpu: {} received: {:9} pid: {:6} tid: {:<6} uid: {:<6} comm: {:<16} msg: {:<256} kernel: {} user: {}",
//         cpu, now, event.pid, event.tid, event.uid, comm_str, msg_str, event.kernel_stack_id, event.user_stack_id
//     );
// }

fn now_formatted() -> String {
    use time::macros::format_description;
    use time::OffsetDateTime;

    OffsetDateTime::now_local().map_or_else(
        |_| "00:00:00".to_string(),
        |now| {
            let format = format_description!("[hour]:[minute]:[second]");
            now.format(&format)
                .unwrap_or_else(|_| "00:00:00".to_string())
        },
    )
}

// TODO(kakkoyun): Can we find an alternative?
use std::str::Utf8Error;

pub unsafe fn str_from_u8_nul(utf8_src: &[u8]) -> Result<&str, Utf8Error> {
    let nul_range_end = utf8_src
        .iter()
        .position(|&c| c == b'\0')
        .unwrap_or(utf8_src.len()); // default to length if no `\0` present
    ::std::str::from_utf8(&utf8_src[0..nul_range_end])
}
