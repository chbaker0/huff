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

mod util;

use std::cmp;
use std::collections::BinaryHeap;
use std::io::prelude::*;

use bv::BitVec;
use bv::Bits;

use util::BitIter;
use util::Keyed;

/// Stores the probability (in the form of a raw count) of each possible input
/// symbol. In this case, a symbol is any byte.
#[derive(Clone, Copy)]
pub struct SymbolCounts {
    pub counts: [u32; 256],
}

/// Stores the bit strings for each symbol (in this case, any byte);
#[derive(Clone, Copy)]
pub struct SymbolCodes {
    pub codes: [SymbolCode; 256],
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SymbolCode {
    packed_bits: [u8; 32],
    length: u8,
}

/// Encode the input with a precomputed Huffman code
pub fn encode<I: IntoIterator<Item = u8>, W: Write>(
    input: I,
    output: &mut W,
    symbol_codes: &SymbolCodes,
) {
    let codes = symbol_codes.codes;
    let mut accumulator = BitVec::<u8>::new();

    for byte in input {
        let code = codes[byte as usize];
        for i in 0..code.bit_len() {
            accumulator.push(code.get_bit(i));
        }

        let full_bytes = accumulator.bit_len() / 8;
        for i in 0..full_bytes {
            let block = accumulator.get_block(i as usize);
            output.write(std::slice::from_ref(&block)).unwrap();
        }

        let remainder_len = accumulator.bit_len() - full_bytes * 8;
        if remainder_len > 0 {
            let remainder = accumulator.get_block(full_bytes as usize);
            accumulator.clear();
            for i in 0..remainder_len {
                accumulator.push(remainder.get_bit(i));
            }
        } else {
            accumulator.clear();
        }
    }
}

pub fn decode<I: IntoIterator<Item = u8>, W: Write>(input: I, output: &mut W, tree: &HuffNode) {
    let mut iter = BitIter::new(input.into_iter());
    while let Some(symbol) = decode_symbol(&mut iter, tree) {
        output.write(std::slice::from_ref(&symbol)).unwrap();
    }
}

fn decode_symbol<I: Iterator<Item = bool>>(bits: &mut I, tree: &HuffNode) -> Option<u8> {
    match tree {
        HuffNode::Leaf(l) => Some(l.symbol),
        HuffNode::Parent(p) => {
            if bits.next()? {
                decode_symbol(bits, &p.one)
            } else {
                decode_symbol(bits, &p.zero)
            }
        }
    }
}

pub fn codes_from_tree(tree: &HuffNode) -> SymbolCodes {
    let mut codes = [Default::default(); 256];

    if let HuffNode::Leaf(_) = tree {
        panic!("must be passed a non-leaf node");
    }

    let mut cur_bits = BitVec::new();
    codes_from_tree_impl(tree, &mut codes, &mut cur_bits);
    assert_eq!(cur_bits.len(), 0);

    SymbolCodes { codes: codes }
}

fn codes_from_tree_impl(tree: &HuffNode, codes: &mut [SymbolCode; 256], cur_bits: &mut BitVec<u8>) {
    match tree {
        HuffNode::Parent(p) => {
            cur_bits.push(false);
            codes_from_tree_impl(&p.zero, codes, cur_bits);
            cur_bits.pop();
            cur_bits.push(true);
            codes_from_tree_impl(&p.one, codes, cur_bits);
            cur_bits.pop();
        }
        HuffNode::Leaf(l) => {
            codes[l.symbol as usize] = SymbolCode::from_bits(cur_bits);
        }
    }
}

impl SymbolCode {
    pub fn from_bits<B: Bits<Block = u8>>(bits: B) -> SymbolCode {
        assert!(bits.bit_len() > 0);
        assert!(bits.bit_len() <= 255);

        let mut packed_bits = [0; 32];
        for i in 0..bits.block_len() {
            packed_bits[i] = bits.get_block(i);
        }

        SymbolCode {
            packed_bits: packed_bits,
            length: bits.bit_len() as u8,
        }
    }
}

impl Bits for SymbolCode {
    type Block = u8;

    fn bit_len(&self) -> u64 {
        self.length as u64
    }

    fn get_block(&self, position: usize) -> Self::Block {
        self.packed_bits[position]
    }
}

pub fn build_tree(symbol_counts: &SymbolCounts) -> HuffNode {
    let counts = symbol_counts.counts;

    let mut node_queue = BinaryHeap::new();
    for symbol in 0..256 {
        if counts[symbol] > 0 {
            node_queue.push(cmp::Reverse(Keyed {
                key: counts[symbol],
                value: HuffNode::Leaf(HuffLeaf {
                    symbol: symbol as u8,
                }),
            }));
        }
    }

    assert!(node_queue.len() > 1);

    while node_queue.len() > 1 {
        let cmp::Reverse(first) = node_queue.pop().unwrap();
        let cmp::Reverse(second) = node_queue.pop().unwrap();

        let new_weight = first.key + second.key;
        let new_node = HuffNode::Parent(HuffParent {
            zero: Box::new(first.value),
            one: Box::new(second.value),
        });

        node_queue.push(cmp::Reverse(Keyed {
            key: new_weight,
            value: new_node,
        }));
    }

    node_queue.pop().unwrap().0.value
}

#[derive(Clone, Debug)]
pub struct HuffParent {
    zero: Box<HuffNode>,
    one: Box<HuffNode>,
}

#[derive(Clone, Copy, Debug)]
pub struct HuffLeaf {
    symbol: u8,
}

#[derive(Clone, Debug)]
pub enum HuffNode {
    Parent(HuffParent),
    Leaf(HuffLeaf),
}

pub fn count_symbols<I: IntoIterator<Item = u8>>(input: I) -> SymbolCounts {
    let mut counts = [0; 256];
    for b in input {
        counts[b as usize] += 1;
    }

    SymbolCounts { counts: counts }
}
