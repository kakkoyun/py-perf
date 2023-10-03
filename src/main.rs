#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]
#![warn(clippy::perf)]

use chrono::{DateTime, Utc};
use log::{debug, error, info, trace};
use std::fs::File;
use std::process::exit;
use std::time::Duration;

use anyhow::{anyhow, Error, Result};
use clap::{Parser, ValueEnum};
use crossbeam::channel::{unbounded, Receiver};
use env_logger::Env;
use nix::sys::utsname::uname;
use nix::unistd::Uid;

use py_perf::arch;
use py_perf::py_perf::PyPerf;

#[derive(ValueEnum, Copy, Clone, Debug)]
enum OutputType {
    Pprof,
    Flamegraph,
    Folded,
}

#[derive(Parser, Debug)]
struct InfoSubcommand {}

#[derive(Parser, Debug)]
struct RecordSubcommand {
    /// Python process IDs to profile.
    #[clap(short, long)]
    pid: i32,
    /// Profiling duration to use.
    #[clap(short, long, default_value = "10s")]
    duration: Option<humantime::Duration>,
    /// The frequency at which profiling data is collected. e.g., 19 samples per second.
    #[clap(long, short = 'q', default_value = "19")]
    frequency: Option<u64>,
    /// The output format to use.
    /// Valid values are: `pprof`, `flamegraph` and `folded`.
    /// The default value is `pprof`.
    #[clap(short, long, default_value = "pprof")]
    format: Option<OutputType>,
}

#[derive(clap::Subcommand, Debug)]
enum Command {
    /// Record profiles from a running process.
    Record(RecordSubcommand),
    /// Print information about host.
    Info(InfoSubcommand),
}

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
struct Arguments {
    #[clap(subcommand)]
    subcmd: Command,
}

fn main() {
    if !arch::is_x86() {
        error!("py-perf only supports x86/x86_64 architectures for now.");
    }

    let env = Env::default().default_filter_or("info");
    env_logger::Builder::from_env(env)
        .format_timestamp_nanos()
        .init();
    debug!("debug mode enabled!");

    if let Err(err) = run() {
        error!("error: {}", err);
        for (i, suberror) in err.chain().enumerate() {
            if i > 0 {
                error!("cause: {}", suberror);
            }
        }
        std::process::exit(1);
    }
}

fn ctrlc_channel() -> Result<Receiver<()>, Error> {
    let (sender, receiver) = unbounded();
    ctrlc::set_handler(move || {
        trace!("signal handler is called");
        sender.send(()).expect("could not send signal on channel.");
    })?;

    Ok(receiver)
}

fn run() -> Result<()> {
    let args = Arguments::parse();
    match args.subcmd {
        Command::Info(_) => {
            if !Uid::current().is_root() {
                return Err(anyhow!(
                    "py-perf requires root to load and run BPF programs"
                ));
            }

            let info = info()?;
            println!("System info");
            println!("-----------");
            println!("Kernel release: {}", info.system.os_release);
            println!("DebugFS mounted: {}", info.system.debug_fs);
            println!();
        }

        Command::Record(record) => {
            if !Uid::current().is_root() {
                return Err(anyhow!(
                    "py-perf requires root to load and run BPF programs"
                ));
            }

            let mut py_perf = PyPerf::new(
                Duration::from_millis(u64::try_from(record.duration.unwrap().as_millis())?),
                record.frequency.unwrap(),
            )?;

            if record.pid == 0 {
                error!("at least one PID must be given");
                exit(1);
            }

            py_perf.record(record.pid)?;
            info!("py-perf is started!");
            let profile = py_perf.start(&ctrlc_channel().unwrap())?;
            info!("py-perf is stopped!");

            let now: DateTime<Utc> = Utc::now();
            let name_suffix = now.format("%m%d%Y_%Hh%Mm%Ss");

            let report = profile.report()?;
            match record.format.unwrap() {
                OutputType::Pprof => {
                    let path = format!("py-perf_{name_suffix}_profile.pb");
                    let f = File::create(&path).unwrap();
                    report.pprof(f)?
                }
                OutputType::Flamegraph => {
                    let path = format!("py-perf_{name_suffix}_flamegraph.svg");
                    let f = File::create(&path).unwrap();
                    report.flamegraph(f)?
                }
                OutputType::Folded => {
                    let path = format!("py-perf_{name_suffix}_folded.txt");
                    let f = File::create(&path).unwrap();
                    report.folded(f)?
                }
            };
            info!("done!");
        }
    }

    Ok(())
}

pub struct SystemInfo {
    pub os_release: String,
    pub debug_fs: bool,
}

pub struct Info {
    pub system: SystemInfo,
}

/// Returns information about the host system.
/// This function is used by the `info` subcommand.
/// It is also used by the `record` subcommand to check for root privileges.
/// # Errors
///
/// Returns an error if the kernel release cannot be determined.
pub fn info() -> Result<Info> {
    Ok(Info {
        system: SystemInfo {
            os_release: uname()?.release().to_string_lossy().to_string(),
            debug_fs: File::open("/sys/kernel/debug/").is_ok(),
        },
    })
}
