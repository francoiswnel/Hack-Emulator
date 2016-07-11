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
        register_pc: 0,
        output_alu_previous: 0,
    };

    let mut memory = Memory {
        ram: [0; 0b1 << 14],
        screen: [0; 0b1 << 13],
        keyboard: 0,
    };

    let mut input_m: u16 = 0;
    let mut output_m: u16 = 0;
    let mut address_m: u16 = 0;
    let mut pc: u16 = 0;
    let mut write_m: bool = false;
    let reset: bool = false;

    // Pretty print cpu before loop
    println!("\n{:#?}\n", cpu);

    // Main execution cycle loop,
    //  runs until a None instruction is fetched.
    loop {
        // Rom
        let instruction: &u16 = match rom.get(pc as usize) {
            Some(instruction) => instruction,
            None => break,
        };

        let instruction: u16 = *instruction;

        // Cpu
        let output_cpu = cpu.cpu(instruction, input_m, reset);
        output_m = output_cpu.0;
        address_m = output_cpu.1;
        pc = output_cpu.2;
        write_m = output_cpu.3;

        // Memory
        input_m = memory.memory(output_m, address_m, write_m);
    }

    // Pretty print cpu after loop
    println!("\n{:#?}\n", cpu);
}

#[derive(Debug)]
struct Cpu {
    register_a: u16,
    register_d: u16,
    register_pc: u16,
    output_alu_previous: u16,
}

impl Cpu {
    pub fn cpu(&mut self, instruction: u16, input_m: u16, reset: bool) -> (u16, u16, u16, bool) {
        let mut flags = ((instruction >> 6) as u8) << 2;    // [zx, nx, zy, ny, f, no, zr, ng]
        let a_instruction: bool = instruction & 0b1 << 15 == 0;
        let c_instruction: bool = !a_instruction;

        let write_a: bool = a_instruction || (c_instruction && instruction & 0b1 << 5 != 0);
        let write_d: bool = c_instruction && instruction & 0b1 << 4 != 0;
        let write_m: bool = c_instruction && instruction & 0b1 << 3 != 0;

        let input_a: u16 = mux(instruction, self.output_alu_previous, a_instruction);
        let output_a: u16 = self.register_a(input_a, write_a);
        let self_output_alu_previous = self.output_alu_previous;
        let input_alu_d: u16 = self.register_d(self_output_alu_previous, write_d);
        let input_alu_am: u16 = mux(output_a, input_m, instruction & 0b1 << 12 != 0);
        let output_m: u16 = self.alu(input_alu_d, input_alu_am, flags);

        let ng: bool = flags & 0b1 != 0;
        let zr: bool = flags & 0b1 != 0;
        let pl: bool = !ng && !zr;
        let jump: bool = (c_instruction && instruction & 0b1 << 2 != 0 && ng) ||
                         (c_instruction && instruction & 0b1 << 1 != 0 && zr) ||
                         (c_instruction && instruction & 0b1 != 0 && pl);
        let pc = self.pc(output_a, jump, !jump, reset);

        return (output_m, output_a, pc, write_m);
    }

    fn alu(&mut self, mut x: u16, mut y: u16, mut flags: u8) -> u16 {
        let mut output: u16 = 0;

        if flags & 0b1 << 7 != 0 {
            x = 0;
        }

        if flags & 0b1 << 6 != 0 {
            x = !x;
        }

        if flags & 0b1 << 5 != 0 {
            y = 0;
        }

        if flags & 0b1 << 4 != 0 {
            y = !y;
        }

        if flags & 0b1 << 3 != 0 {
            output = x + y;
        } else {
            output = x & y;
        }

        if flags & 0b1 << 2 != 0 {
            output = !output;
        }

        if output == 0 {
            flags = flags | 0b1 << 1;
        }

        if output < 0 {
            flags = flags | 0b1;
        }

        return output;
    }

    fn register_a(&mut self, input: u16, load: bool) -> u16 {
        if load {
            self.register_a = input;
        }

        return input;
    }

    fn register_d(&mut self, input: u16, load: bool) -> u16 {
        if load {
            self.register_d = input;
        }

        return input;
    }

    fn pc(&mut self, input: u16, load: bool, increment: bool, reset: bool) -> u16 {
        if reset {
            self.register_pc = 0;
        } else if load {
            self.register_pc = input;
        } else if increment {
            self.register_pc += 1;
        }

        return self.register_pc;
    }
}

fn mux(input_a: u16, input_b: u16, select: bool) -> u16 {
    if select {
        return input_b;
    }

    return input_a;
}

struct Memory {
    ram: [u16; 0b1 << 14],
    screen: [u16; 0b1 << 13],
    keyboard: u16,
}

impl Memory {
    pub fn memory(&mut self, input: u16, address: u16, load: bool) -> u16 {
        if load {
            self.set(input, address);
        }

        return self.get(address);
    }

    fn set(&mut self, input: u16, address: u16) {
        if address < 0b1 << 14 {
            self.ram[address as usize] = input;
        } else if address < 0b11 << 14 {
            self.screen[(address - 0b1 << 14) as usize] = input;
        }
    }

    fn get(&self, address: u16) -> u16 {
        if address < 16384 {
            return self.ram[address as usize];
        } else if address < 0b11 << 14 {
            return self.screen[(address - 0b1 << 14) as usize];
        } else {
            return self.keyboard;
        }
    }
}
