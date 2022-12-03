// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
// Modified from the source, https://github.com/rust-lang/rustc-hash.
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::ops::BitXor;

#[cfg(target_pointer_width = "32")]
const K: usize = 0x9e3779b9;
#[cfg(target_pointer_width = "64")]
const K: usize = 0x517cc1b727220a95;

#[derive(Default)]
pub struct FxHasher {
    hash: usize,
}
impl FxHasher {
    #[inline]
    pub fn add_to_hash(&mut self, i: usize) {
        self.hash = self.hash.rotate_left(5).bitxor(i).wrapping_mul(K);
    }

    #[inline]
    pub fn finish(&self) -> u64 {
        self.hash as u64
    }
}
