[![wakatime](https://wakatime.com/badge/user/c03c2c3a-0328-4e74-ba79-1ce0eb43a4f8/project/6de0edd3-d3d9-48b1-8f9e-e019fc7b42f1.svg)](https://wakatime.com/badge/user/c03c2c3a-0328-4e74-ba79-1ce0eb43a4f8/project/6de0edd3-d3d9-48b1-8f9e-e019fc7b42f1)

# py-perf

A Proof-of-concept low-overhead sampling CPU profiler written in Rust for Python implemented using eBPF.
It is heavily "influenced" by [rbperf](https://github.com/javierhonduco/rbperf) and [py-spy](https://github.com/benfred/py-spy).

> IT IS NOT READY FOR PRODUCTION USE AND IT IS NOT INTENDED TO BE A REPLACEMENT FOR EXISTING TOOLS.
> If you are looking for a production-ready tool, please check out [parca-agent](https://github.com/parca-dev/parca-agent) instead.

## Features

The main goals for `py-perf` are:

- On-CPU profiling support
- Low overhead
- Profiled processes don't have to be restarted or modified in any way

## Installation

The latest release is available [here](https://github.com/kakkoyun/py-perf/releases/latest).

## Usage

### CPU sampling

```shell
sudo py-perf record --pid `pidof python` cpu
```

Some debug information will be printed, and a flame graph called `py-perf_flame_$date` will be written to disk 🎉

## Supported Python versions

The currently supported Python (CPython) versions:

- **2.7**: 2.7.x
- **3.x**: 3.3.x, 3.5.x, 3.6.x, 3.7.x, 3.8.x, 3.9.x, 3.10.x, 3.11.x

## Supported kernels

Linux kernel 4.18 is the minimum required version but 5.x and greater is recommended.

## Building

To build `py-perf` you would need a modern Linux machine with:

- The Rust toolchain
- `clang` to compile the BPF code
- `elfutils` and `zlib` installed
- `make` and `pkg-config` to build libbpf

Once the dependencies are installed:

```shell
# As we are statically linking elfutils and zlib, we have to tell Rustc
# where are they located. On my Ubuntu system they are under
$ export RUSTFLAGS='-L /usr/lib/x86_64-linux-gnu'
$ cargo build [--release]
```

The built binary can be found under `target/(debug|release)/py-perf`.

## Developing and troubleshooting

Debug logs can be enabled with `RUST_LOG=debug`. The info subcommand, `py-perf info` shows the supported BPF features as well as other supported details.

## Stability

`py-perf` is in active development and the CLI and APIs might change any time.

## Bugs

If you encounter any bugs, feel free to open an issue on py-perf's [repo](https://github.com/kakkoyun/py-perf).

## Acknowledgments

`py-perf` wouldn't be possible without all the open-source projects that we benefit from, such as [Rust](https://github.com/rust-lang), [rbperf](https://github.com/javierhonduco/rbperf), [py-spy](https://github.com/benfred/py-spy) and all the superb crates we use in this project, Python, the BPF ecosystem, and many others!

## License

User-space code: Apache 2

Kernel-space code (eBPF profiler): GNU General Public License, version 2

#### TODO

- TODO(kakkoyun): Add sections from parca-agent!
- TODO(kakkoyun): Add reference to bcc, bcc/granulate and linux/tool examples from facebook.

## Features:

- Supports profiling Python processes running in Docker containers. Tested using official Python
  Docker images (`python:X.Y`).
- Supports glibc- and musl-based environments.
- Supports Python compiled in both PIE and non-PIE configurations.
- Supports Python running standalone and as a library (linked with `libpythonX.Y`).

## Limitations:

- Architecture: x86_64.
- Linux kernel version: oldest version tested is 4.14. Versions 4.11-4.14 may work. Required for
  `bpf_probe_read_str`.
- BCC version: using BCC nightly is recommended. v0.17 is known to work.
- Clang/LLVM: at least version 9.

## Overview

PyPerf uses Linux's perf events subsystem to gather stack samples of running Python interpreters at
a constant interval. Instead of capturing native execution stacks, PyPerf reads the information
stored by the Python interpreter regarding the current state of execution. Unlike many existing
tools however, the memory of the process is read from a kernel context. The advantages of this
approach are mainly reduced system overhead and no intervention with the program being profiled.
