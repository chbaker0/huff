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

use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::stdin;
use std::io::stdout;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::SeekFrom;
use std::vec::Vec;

use bv::Bits;

use huff::build_tree;
use huff::codes_from_tree;
use huff::count_symbols;
use huff::decode;
use huff::encode;
use huff::SymbolCodes;

fn main() -> io::Result<()> {
    let args: Vec<_> = env::args().take(3).collect();
    if args.len() < 3 {
        let progname = if args.len() > 0 { &args[0] } else { "?" };
        println!("incorrect arguments");
        println!("usage: {} <input-filename> <output-filename>", progname);
        return Ok(());
    }

    let mut infile = File::open(&args[1])?;

    let symbol_counts = count_symbols(BufReader::new(&infile).bytes().map(Result::unwrap));

    let tree = build_tree(&symbol_counts);
    let codes = codes_from_tree(&tree);
    print_codes(&codes);

    let mut encoded = Vec::new();
    encoded.reserve(infile.metadata()?.len() as usize);

    infile.seek(SeekFrom::Start(0))?;
    encode(
        BufReader::new(infile).bytes().map(Result::unwrap),
        &mut encoded,
        &codes,
    );

    let outfile = File::create(&args[2])?;
    decode(encoded.iter().copied(), &mut BufWriter::new(outfile), &tree);

    Ok(())
}

fn print_codes(symbol_codes: &SymbolCodes) {
    for (sym, code) in (&symbol_codes.codes).iter().enumerate() {
        if code.bit_len() > 0 {
            println!("{}\t{}", sym as u8 as char, bits_to_string(code));
        }
    }
}

fn bits_to_string<B: Bits>(bits: B) -> String {
    let mut result = String::new();
    for i in 0..bits.bit_len() {
        result.push(if bits.get_bit(i) { '1' } else { '0' });
    }
    result
}
