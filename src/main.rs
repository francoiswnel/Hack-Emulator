///
/// Hack Emulator
/// Created by Francois W. Nel on 9 Jul 2016.
///
/// TODO: Documentation
///
/// Usage:
///  $ hemu <path/to/rom_file.hack>
///

// TODO: define hardware
// TODO: implement hardware
// TODO: instruction cycle
// TODO: keyboard input
// TODO: display output

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
    let mut rom = Rom {rom_map: Vec::new(), buffer_line_number: 0};
    rom.convert_buffer(&rom_buffer);

    let mut cpu = Cpu {register_a: 0, register_d: 0, register_m: 0};

    let mut program_counter = ProgramCounter {register_pc: 0};

    loop {
        // TODO: Find safer way to break loop. Maybe try macro, modify get_instruction to return result.
        if rom.get_instruction(program_counter.get_pc()) == 0 {
            break;
        }

        println!("{:?}", rom.get_instruction(program_counter.get_pc()));
        program_counter.increment();
    }

    // Output for debug.
    // rom.print();
    // let char_str: String = rom.get_instruction_string(1);
    // for c in char_str.chars() {
    //     print!("{}", c);
    // }
}

struct Rom {
    rom_map: Vec<u16>,
    buffer_line_number: u16
}

impl Rom {
    pub fn convert_buffer(&mut self, buffer: &String) {
        for instruction in buffer.lines() {
            self.rom_map.push(match u16::from_str_radix(&instruction.trim(), 2) {
                Ok(num) => num,
                Err(why) => panic!("\nError: Failed to parse line {:?}: {}\n", self.buffer_line_number, why.description()),
            });
            self.buffer_line_number += 1;
        }
    }

    pub fn get_instruction(&self, address: u16) -> Option<u16> {
        // TODO: Return Some(u16) if possible, otherwise None.
        return self.rom_map[address as usize];
    }

    pub fn get_instruction_string(&self, address: u16) -> String {
        return format!("{:0>16b}", self.rom_map[address as usize]);
    }

    // Only for debug, delete for release
    pub fn print(&self) {
        for instruction in &self.rom_map {
            println!("{:?}", instruction);
            let instruction_string_binary = format!("{:0>16b}", instruction);
            println!("{}", instruction_string_binary);
        }
    }
}

struct Cpu {
    register_a: u16,
    register_d: u16,
    register_m: u16
}

impl Cpu {
    pub fn set_register_a(&mut self, value: u16) {
        self.register_a = value;
    }

    pub fn set_register_d(&mut self, value: u16) {
        self.register_d = value;
    }

    pub fn set_register_m(&mut self, value: u16) {
        self.register_m = value;
    }

    pub fn get_register_a(&self) -> u16 {
        return self.register_a;
    }

    pub fn get_register_d(&self) -> u16 {
        return self.register_d;
    }

    pub fn get_register_m(&self) -> u16 {
        return self.register_m;
    }
}

struct Memory {
    ram_map: [u16; 16384],
    display_map: [u16; 8192],
    keyboard_map: u16
}

impl Memory {
    pub fn set_memory(&mut self, value: u16, address: u16) {
        if address < 16384 {
            self.ram_map[address as usize] = value;
        }
        else if address < 24576 {
            self.display_map[(address - 16384) as usize] = value;
        }
    }

    pub fn get_memory(&self, address: u16) -> u16 {
        if address < 16384 {
            return self.ram_map[address as usize];
        }
        else if address < 24576 {
            return self.display_map[(address - 16384) as usize];
        }
        else {
            return self.keyboard_map;
        }
    }
}

struct ProgramCounter {
    register_pc: u16
}

impl ProgramCounter {
    pub fn get_pc(&self) -> u16 {
        return self.register_pc;
    }

    pub fn increment(&mut self) {
        self.register_pc += 1;
    }

    pub fn load(&mut self, address: u16) {
        self.register_pc = address;
    }

    pub fn reset(&mut self) {
        self.register_pc = 0;
    }
}
