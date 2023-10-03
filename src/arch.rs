// Copyright (c) 2023 The rbperf authors
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[must_use]
pub const fn is_x86() -> bool {
    true
}

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
pub const fn is_x86() -> bool {
    false
}
