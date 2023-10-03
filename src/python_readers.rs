// Copyright (c) 2023 The rbperf authors
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

// TODO(kakkoyun): Check py-spy!
// TODO(kakkoyun): Do we really need this?
// TODO(kakkoyun): Can we use plain instead?

pub const unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::std::slice::from_raw_parts((p as *const T).cast::<u8>(), ::std::mem::size_of::<T>())
}
