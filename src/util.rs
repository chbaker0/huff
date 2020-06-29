// Copyright 2020 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::cmp;
use std::iter::Iterator;

use bv::Bits;

/// `Keyed` stores a key-value pair and forwards comparisons to the key.
///
/// `Keyed` is a helper for containers that use comparisons on elements. For key
/// type `K` and value type `T`, `Keyed<K, T>` stores a key and value. It
/// forwards all comparison operations to `key`. This allows, for example, a
/// `std::collections::BinaryHeap` to use a user-provided priority key instead
/// of the value that is actually being stored.
#[derive(Clone, Copy, Debug)]
pub struct Keyed<K, T> {
    pub key: K,
    pub value: T,
}

impl<K: PartialEq, T> PartialEq for Keyed<K, T> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl<K: Eq, T> Eq for Keyed<K, T> {}

impl<K: Ord, T> Ord for Keyed<K, T> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.key.cmp(&other.key)
    }
}

impl<K: PartialOrd, T> PartialOrd for Keyed<K, T> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.key.partial_cmp(&other.key)
    }
}

pub struct BitIter<Inner> {
    inner: Inner,
    cur: u8,
    bit_index: u8,
}

impl<I: Iterator<Item = u8>> BitIter<I> {
    pub fn new(inner: I) -> BitIter<I> {
        BitIter {
            inner: inner,
            cur: 0,
            bit_index: 8,
        }
    }
}

impl<I: Iterator<Item = u8>> Iterator for BitIter<I> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bit_index == 8 {
            self.cur = self.inner.next()?;
            self.bit_index = 0;
        }

        self.bit_index += 1;
        Some(self.cur.get_bit(self.bit_index as u64 - 1))
    }
}
