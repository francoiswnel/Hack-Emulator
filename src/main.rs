///
/// Hack Emulator
/// Created by Francois W. Nel on 9 Jul 2016.
///
/// TODO: Documentation
///
/// Usage:
///  $ hemu <path/to/rom_file.hack>
///

// TODO: instruction cycle
// TODO: keyboard input
// TODO: display output

use std::env;
use std::error::Error;
use std::fmt;
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
        Err(why) => {
            panic!("\nError: Failed to open {:?}: {}\n",
                   rom_file_name,
                   why.description())
        }
        Ok(rom_file) => rom_file,
    };

    // Create the rom_file buffer and read the file into it.
    let mut rom_buffer = String::new();

    match rom_file.read_to_string(&mut rom_buffer) {
        Err(why) => {
            panic!("\nError: Failed to read {:?}: {}\n",
                   rom_file_name,
                   why.description())
        }
        Ok(_) => (),
    }

    // Parse instructions into rom.
    let mut rom: Vec<u16> = vec![];
    let mut buffer_line_number: u16 = 0;

    for instruction in rom_buffer.lines() {
        rom.push(match u16::from_str_radix(&instruction.trim(), 2) {
            Ok(num) => num,
            Err(why) => {
                panic!("\nError: Failed to parse line {:?}: {}\n",
                       buffer_line_number,
                       why.description())
            }
        });
        buffer_line_number += 1;
    }

    // Initialise the processor
    let mut cpu = Cpu {
        register_a: 0,
        register_d: 0,
        register_m: Memory {
            ram_map: [0; 0b1 << 14],
            display_map: [0; 0b1 << 13],
            keyboard_map: 0,
        },
        pc: ProgramCounter { register_pc: 0 },
        flags: 0,
    };

    // Pretty print cpu before loop
    println!("\n{:#?}\n", cpu);

    // Main execution cycle loop,
    //  runs until a None instruction is fetched.
    loop {
        let current_instruction: &u16 = match rom.get(cpu.pc.get() as usize) {
            Some(current_instruction) => current_instruction,
            None => break,
        };

        println!("{:?}", current_instruction);
        let instruction_string_binary = format!("{:0>16b}", current_instruction);
        println!("{}", instruction_string_binary);
        cpu.pc.increment();
    }

    // Pretty print cpu after loop
    println!("\n{:#?}\n", cpu);

    // Output for debug.
    // let char_str: String = rom.get_instruction_string(1);
    // for c in char_str.chars() {
    //     print!("{}", c);
    // }
}

struct Cpu {
    register_a: u16,
    register_d: u16,
    register_m: Memory,
    pc: ProgramCounter,
    flags: u8, // [zx, nx, zy, ny, f, no, zr, ng]
}

impl Cpu {
    pub fn cpu(&mut self, instruction: u16, inM: u16) -> u16 {
        if instruction < 0b1 << 15 {
            self.register_a = instruction;
        } else {
        }
        0
    }

    fn alu(&mut self, mut x: u16, mut y: u16) -> u16 {
        let mut out: u16 = 0;

        if self.flags & 0b1 << 7 != 0 {
            x = 0;
        }

        if self.flags & 0b1 << 6 != 0 {
            x = !x;
        }

        if self.flags & 0b1 << 5 != 0 {
            y = 0;
        }

        if self.flags & 0b1 << 4 != 0 {
            y = !y;
        }

        if self.flags & 0b1 << 3 != 0 {
            out = x + y;
        } else {
            out = x & y;
        }

        if self.flags & 0b1 << 2 != 0 {
            out = !out;
        }

        if out == 0 {
            self.flags = self.flags | 0b1 << 1;
        }

        if out < 0 {
            self.flags = self.flags | 0b1;
        }

        return out;
    }
}

impl fmt::Debug for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "Cpu {{\n\tregister_a:\t{:0>16b}\n\tregister_d:\t{:0>16b}\n\tregister_m:\t{:\
                0>16b}\n\tregister_pc:\t{:0>16b}\n\tflags:\t\t{:0>8b}\n}}",
               self.register_a,
               self.register_d,
               self.register_m.get(self.register_a),
               self.pc.get(),
               self.flags)
    }
}

struct Memory {
    ram_map: [u16; 0b1 << 14],
    display_map: [u16; 0b1 << 13],
    keyboard_map: u16,
}

impl Memory {
    pub fn set(&mut self, value: u16, address: u16) {
        if address < 0b1 << 14 {
            self.ram_map[address as usize] = value;
        } else if address < 0b11 << 14 {
            self.display_map[(address - 0b1 << 14) as usize] = value;
        }
    }

    pub fn get(&self, address: u16) -> u16 {
        if address < 16384 {
            return self.ram_map[address as usize];
        } else if address < 0b11 << 14 {
            return self.display_map[(address - 0b1 << 14) as usize];
        } else {
            return self.keyboard_map;
        }
    }
}

struct ProgramCounter {
    register_pc: u16,
}

impl ProgramCounter {
    pub fn get(&self) -> u16 {
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
