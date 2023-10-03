use log::info;
use std::fmt;

use anyhow::{Context, Result};

// use py_spy::python_interpreters::{InterpreterState, ThreadState};
use py_spy::python_process_info::{
    get_interpreter_address, get_python_version, get_threadstate_address, PythonProcessInfo,
};
use py_spy::version::Version;
use remoteprocess::{Pid, Process};

pub struct ProcessInfo {
    pub pid: Pid,
    pub process: Process,

    pub version: Version,
    pub version_string: String,

    pub python_info: PythonProcessInfo,
    pub interpreter_address: u64,
    pub thread_state_address: u64,
}

impl fmt::Display for ProcessInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "pid: {}", self.pid)?;
        writeln!(f, "python version: \n\t{:?}", self.version)?;
        writeln!(f, "\tstring: {}", self.version_string)?;

        writeln!(f, "python interpreter info:")?;
        writeln!(
            f,
            "\tfilename: {}",
            self.python_info.python_filename.display()
        )?;
        writeln!(f, "\tdockerized: {}", self.python_info.dockerized)?;
        writeln!(f, "\tinterpreter address: 0x{:x}", self.interpreter_address)?;
        writeln!(
            f,
            "\tthreadstate address: 0x{:x}",
            self.thread_state_address
        )?;

        Ok(())
    }
}

impl ProcessInfo {
    pub fn new(pid: Pid) -> Result<Self, anyhow::Error> {
        let process =
            Process::new(pid).context("failed to open process: check if it is running.")?;

        let python_info = PythonProcessInfo::new(&process)?;

        let version = get_python_version(&python_info, &process)?;
        info!("python version {} detected", version);

        let interpreter_address = get_interpreter_address(&python_info, &process, &version)?;
        info!("found interpreter at 0x{:016x}", interpreter_address);

        let thread_state_address = get_threadstate_address(&python_info, &version, false)?;
        info!("found thread state at 0x{:016x}", thread_state_address);

        let version_string = format!("python{}.{}", version.major, version.minor);

        Ok(Self {
            pid,
            process,
            version,
            version_string,
            python_info,
            interpreter_address: interpreter_address as u64,
            thread_state_address: thread_state_address as u64,
        })
    }

    pub fn children(self) -> Result<Vec<Self>, anyhow::Error> {
        let mut children = Vec::new();

        for (child_pid, _) in self.process.child_processes()? {
            children.push(Self::new(child_pid)?);
        }

        Ok(children)
    }
}
