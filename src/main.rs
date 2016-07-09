///
/// Rust Hack Emulator
/// Created by Francois W. Nel on 9 Jul 2016.
///
/// TODO: Documentation
///
/// Usage:
///  $ hemu <path/to/rom.hack>
///

// TODO: define hardware
// TODO: implement hardware
// TODO: reset cpu
// TODO: instruction cycle
// TODO: keyboard input
// TODO: display output

use std::env;
use std::io::prelude::*;
use std::error::Error;
use std::fs::File;
use std::path::Path;

fn main() {
    let arguments: Vec<_> = env::args().collect();

    // Ensure that at least one file name is specified,
    //  only the first argument will be used, the rest are ignored.
    if arguments.len() < 2 {
        panic!(println!("\nUsage: {} <path/to/rom.hack>\n", arguments[0]))
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
    let mut rom = match File::open(&rom_path) {
        Err(why) => panic!("\nError: Failed to open {:?}: {}\n", rom_file_name, why.description()),
        Ok(rom) => rom,
    };

    // Create the rom buffer and read the file into it.
    let mut rom_buffer = String::new();

    match rom.read_to_string(&mut rom_buffer) {
        Err(why) => panic!("\nError: Failed to read {:?}: {}\n", rom_file_name, why.description()),
        Ok(_) => (),
    }

    // Output for debug.
    for instruction in rom_buffer.lines() {
        println!("{}", instruction);
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
