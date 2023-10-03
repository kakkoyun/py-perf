#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]
#![warn(clippy::perf)]
pub mod arch;
pub mod bindings;
pub mod py_perf;
pub mod python_versions;

mod bpf;
mod perf_event;
mod process_info;
mod profile;
mod python_readers;
