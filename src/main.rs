///
/// Rust Hack Emulator
/// Created by Francois W. Nel on 9 Jul 2016.
///
/// TODO: Documentation
///
/// Usage:
///  $ hemu <path/to/rom_file.hack>
///

// TODO: define hardware
// TODO: implement hardware
// TODO: reset cpu
// TODO: instruction cycle
// TODO: keyboard input
// TODO: display output

extern crate byteorder;

use byteorder::{ReadBytesExt, BigEndian};
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

fn main() {
    let arguments: Vec<_> = env::args().collect();

    // Ensure that at least one file name is specified,
    //  only the first argument will be used, the rest are ignored.
    if arguments.len() < 2 {
        panic!(println!("\nUsage: {} <path/to/rom_file.hack>\n", arguments[0]))
    }

    // Parse the path to get the file name and extension.
    let rom_path = Path::new(&arguments[1]);
    let rom_file_name = rom_path.file_name().unwrap();
    let rom_file_extension = rom_path.extension().unwrap();

    // Ensure that the file extension is ".hack".
    assert_eq!(rom_file_extension, "hack");

    // If the test passed, confirm success.
    println!("\nRunning: {:?}\n", rom_file_name);

    // Attempt to open the file.
    let mut rom_file = match File::open(&rom_path) {
        Err(why) => panic!("\nError: Failed to open {:?}: {}\n", rom_file_name, why.description()),
        Ok(rom_file) => rom_file,
    };

    // Create the rom_file buffer and read the file into it.
    let mut rom_buffer = String::new();

    match rom_file.read_to_string(&mut rom_buffer) {
        Err(why) => panic!("\nError: Failed to read {:?}: {}\n", rom_file_name, why.description()),
        Ok(_) => (),
    }

    // Parse instructions into rom.
    let mut rom: Vec<u16> = vec![];
    let mut line_number: usize = 0;

    for instruction in rom_buffer.lines() {
        // rom.push(match instruction.trim().parse() {
        //     Ok(num) => num,
        //     Err(why) => panic!("\nError: Failed to parse line {:?}: {}\n", line_number, why.description()),
        // });
        rom.push(&instruction.read_u16::<BigEndian>.unwrap());
        line_number += 1;
    }

    // Output for debug.
    for i in rom {
        println!("{:?}", i);
    }
}

struct Cpu {
    // parts
}

impl Cpu {
    // implementation
}

struct Memory {
    ram: [u16; 24577]
}

impl Memory {
    // implementation
}
