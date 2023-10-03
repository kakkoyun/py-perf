// Copyright (c) 2023 The rbperf authors
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use std::os::raw::{c_int, c_ulong};

use anyhow::{anyhow, Result};
use errno::errno;
use libc::{self, pid_t};

use perf_event_open_sys as sys;
use perf_event_open_sys::bindings::{perf_event_attr, PERF_FLAG_FD_CLOEXEC};

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
unsafe fn perf_event_open(
    attrs: *mut perf_event_attr,
    pid: pid_t,
    cpu: c_int,
    group_fd: c_int,
    flags: c_ulong,
) -> c_int {
    sys::perf_event_open(attrs, pid, cpu, group_fd, flags) as c_int
}

/// # Safety
pub unsafe fn setup(cpu: i32, frequency: u64, pid: Option<i32>) -> Result<c_int> {
    let mut attrs = perf_event_open_sys::bindings::perf_event_attr {
        size: u32::try_from(std::mem::size_of::<sys::bindings::perf_event_attr>())?,
        type_: sys::bindings::PERF_TYPE_SOFTWARE,
        config: u64::from(sys::bindings::PERF_COUNT_SW_CPU_CLOCK),
        ..Default::default()
    };
    let sample_period = u64::pow(10, 9) / frequency;
    attrs.__bindgen_anon_1.sample_period = sample_period;
    attrs.__bindgen_anon_1.sample_freq = frequency;
    attrs.set_disabled(1);

    let pid = pid.unwrap_or(-1);

    let fd = perf_event_open(
        &mut attrs,
        pid,                             /* pid */
        cpu,                             /* cpu */
        -1,                              /* group_fd */
        u64::from(PERF_FLAG_FD_CLOEXEC), /* flags */
    );

    if fd < 0 {
        return Err(anyhow!("setup_perf_event failed with errno {}", errno()));
    }

    Ok(fd)
}
