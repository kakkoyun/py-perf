use std::fmt::Write;
use std::fs;
use std::time::{Duration, SystemTime};
use std::{collections::HashMap, path::Path};

use anyhow::Result;
use pprof::protos;
use pprof::protos::Message;
use pprof::timer::ReportTiming;
use pprof::{Frames, Symbol};

pub struct Report {
    pub data: HashMap<Frames, isize>,
    pub timing: ReportTiming,
}

impl Report {
    pub fn pprof<W>(&self, mut writer: W) -> Result<()>
    where
        W: std::io::Write,
    {
        let report = pprof::Report {
            data: self.data.clone(),
            timing: self.timing.clone(),
        };
        let profile: protos::Profile = report.pprof().unwrap();

        let mut content = Vec::new();
        profile.write_to_vec(&mut content).unwrap();
        writer.write_all(&content).unwrap();
        println!("report: {:?}", &report);
        Ok(())
    }

    pub fn flamegraph<W>(&self, writer: W) -> Result<()>
    where
        W: std::io::Write,
    {
        let report = pprof::Report {
            data: self.data.clone(),
            timing: self.timing.clone(),
        };
        report.flamegraph(writer).unwrap();
        println!("report: {:?}", &report);
        Ok(())
    }

    pub fn folded<W>(&self, mut writer: W) -> Result<()>
    where
        W: std::io::Write,
    {
        let report = pprof::Report {
            data: self.data.clone(),
            timing: self.timing.clone(),
        };

        let lines: Vec<String> = self
            .data
            .iter()
            .map(|(key, value)| {
                let mut line = key.thread_name_or_id();
                line.push(';');

                for frame in key.frames.iter().rev() {
                    for symbol in frame.iter().rev() {
                        write!(&mut line, "{};", symbol).unwrap();
                    }
                }

                line.pop().unwrap_or_default();
                write!(&mut line, " {}", value).unwrap();

                line
            })
            .collect();
        if !lines.is_empty() {
            writer.write_all(lines.join("\n").as_bytes()).unwrap();
        }
        println!("report: {:?}", &report);
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct Profile {
    pub start_time: Option<SystemTime>,

    duration: Duration,
    frequency: u64,

    // frames: Vec<Frames>,
    thread_id_to_frames: HashMap<u64, Frames>,
    // thread_id_to_name: HashMap<u64, String>,
    data: HashMap<Frames, isize>,
    // From rbperf:
    // #[serde(skip)]
    // symbol_id_map: HashMap<String, u32>,
    // symbols: Vec<String>,
    // samples: Vec<Sample>,
}

impl Profile {
    pub fn new(duration: Duration, frequency: u64) -> Self {
        Self {
            start_time: None,
            duration,
            frequency,
            thread_id_to_frames: HashMap::new(),
            data: HashMap::new(),
        }
    }

    pub fn add_sample(
        &mut self,
        thread_id: u64,
        timestamp: SystemTime,
        sample: Vec<Symbol>,
        weight: isize,
    ) {
        let frames = self
            .thread_id_to_frames
            .entry(thread_id)
            .or_insert_with(|| Frames {
                frames: vec![sample],
                thread_name: get_thread_name(thread_id),
                thread_id,
                sample_timestamp: timestamp,
            });
        *self.data.entry(frames.clone()).or_insert(weight) += weight;
    }

    pub fn report(&self) -> Result<Report> {
        Ok(Report {
            data: self.data.clone(),
            timing: ReportTiming {
                frequency: i32::try_from(self.frequency)?,
                start_time: self.start_time.unwrap(),
                duration: self.duration,
            },
        })
    }
}

fn get_thread_name(tid: u64) -> String {
    let path_str = format!("/proc/{tid}/comm");
    let path = Path::new(&path_str);

    match fs::read_to_string(path) {
        Ok(name) => name.trim().to_string(),
        Err(_) => format!("Thread {tid}"),
    }
}
