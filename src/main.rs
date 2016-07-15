///
/// Hack Emulator
/// Created by Francois W. Nel on 9 Jul 2016.
///
/// TODO: Documentation
///
/// Usage:
///  $ hemu <path/to/rom_file.hack>
///

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

    // Initialise the processor and memory.
    let mut cpu = Cpu {
        register_a: 0,
        register_d: 0,
        register_pc: 0,
        input_alu_d: 0,
        input_alu_am: 0,
        output_alu: 0,
        input_a: 0,
        output_a: 0,
        flags: 0,
        write_a: false,
        write_d: false,
        write_m: false,
        jump: false,
        instruction_type: false,
    };

    let mut memory = Memory {
        ram: [0; 0b1 << 14],
        screen: [0; 0b1 << 13],
        keyboard: 0,
    };

    // Initialise interconnect and loop control variables.
    let mut instruction: u16 = 0;
    let mut instruction_previous: u16 = 0;
    let mut instruction_previous_previous: u16 = 0;
    let mut infinite_loop_counter: u16 = 0;

    // Interconnect: output_alu, address_m, write_m, pc
    let mut cpu_memory_interconnect: (u16, u16, bool, u16) = (0, 0, false, 0);
    let mut memory_cpu_interconnect: u16 = 0;
    let reset: bool = false;

    // Main execution loop:
    //  Runs until a None instruction is fetched
    //  or an infinite loop is detected.
    loop {
        // Fetch instruction from Rom.
        instruction = *(match rom.get(cpu_memory_interconnect.3 as usize) {
            Some(instruction) => instruction,
            None => break,
        });

        // Detect infinite loop.
        if instruction_previous_previous == instruction {
            if infinite_loop_counter > 1 {
                break;
            }
            infinite_loop_counter += 1;
        }

        // Process instruction using Cpu.
        cpu_memory_interconnect = cpu.cpu(instruction, memory_cpu_interconnect, reset);

        // Store output in Memory.
        memory_cpu_interconnect = memory.memory(
            cpu_memory_interconnect.0,
            cpu_memory_interconnect.1,
            cpu_memory_interconnect.2,
        );

        // Pretty print cpu state for debug.
        println!("Instruction: {:0>16b}\n\n{:#?}\n", instruction, cpu);

        // Update variables for infinte loop detection.
        instruction_previous_previous = instruction_previous;
        instruction_previous = instruction;
    }
}

#[derive(Debug)]
struct Cpu {
    register_a: u16,
    register_d: u16,
    register_pc: u16,
    input_alu_d: u16,
    input_alu_am: u16,
    output_alu: u16,
    input_a: u16,
    output_a: u16,
    flags: u8,
    write_a: bool,
    write_d: bool,
    write_m: bool,
    jump: bool,
    instruction_type: bool,
}

impl Cpu {
    pub fn cpu(&mut self, instruction: u16, input_m: u16, reset: bool) -> (u16, u16, bool, u16) {
        // Get control flags from instruction: [zx, nx, zy, ny, f, no, zr, ng].
        self.flags = ((instruction >> 6) as u8) << 2;

        // Get instruction type: true = c_instruction, false = a_instruction.
        self.instruction_type = instruction & 0b1 << 15 != 0;

        // Set write flags.
        self.write_a = !self.instruction_type || (self.instruction_type && instruction & 0b1 << 5 != 0);
        self.write_d = self.instruction_type && instruction & 0b1 << 4 != 0;
        self.write_m = self.instruction_type && instruction & 0b1 << 3 != 0;

        // Get Alu output.
        self.input_a = mux(self.output_alu, instruction, !self.instruction_type);
        self.output_a = self.register_a();
        self.input_alu_am = mux(self.output_a, input_m, instruction & 0b1 << 12 != 0);
        self.alu();
        self.input_alu_d = self.register_d();

        // Set jump flag.
        let ng: bool = self.flags & 0b1 != 0;
        let zr: bool = self.flags & 0b1 != 0;
        let pl: bool = !ng && !zr;
        self.jump = (self.instruction_type && instruction & 0b1 << 2 != 0 && ng) ||
            (self.instruction_type && instruction & 0b1 << 1 != 0 && zr) ||
            (self.instruction_type && instruction & 0b1 != 0 && pl);

        // Increment program counter or jump to instruction.
        self.pc(reset);

        (self.output_alu, self.output_a, self.write_m, self.register_pc)
    }

    fn alu(&mut self) {
        let mut x: u16 = self.input_alu_d;
        let mut y: u16 = self.input_alu_am;

        // If zx flag is set.
        if self.flags & 0b1 << 7 != 0 {
            x = 0;
        }

        // If nx flag is set.
        if self.flags & 0b1 << 6 != 0 {
            x = !x;
        }

        // If zy flag is set.
        if self.flags & 0b1 << 5 != 0 {
            y = 0;
        }

        // If ny flag is set.
        if self.flags & 0b1 << 4 != 0 {
            y = !y;
        }

        // Determine function based on f flag.
        if self.flags & 0b1 << 3 != 0 {
            self.output_alu = x + y;
        } else {
            self.output_alu = x & y;
        }

        // If no flag is set.
        if self.flags & 0b1 << 2 != 0 {
            self.output_alu = !self.output_alu;
        }

        // Set zr and ng flags.
        if self.output_alu == 0 {
            self.flags = self.flags | 0b1 << 1;
        } else if self.output_alu & 0b1 << 15 != 0 {
            self.flags = self.flags | 0b1;
        }
    }

    fn register_a(&mut self) -> u16 {
        if self.write_a {
            self.register_a = self.input_a;
        }

        self.register_a
    }

    fn register_d(&mut self) -> u16 {
        if self.write_d {
            self.register_d = self.output_alu;
        }

        self.register_d
    }

    fn pc(&mut self, reset: bool) {
        if reset {
            self.register_pc = 0;
        } else if self.jump {
            self.register_pc = self.output_a;
        } else if !self.jump {
            self.register_pc += 1;
        }
    }
}

fn mux(input_a: u16, input_b: u16, select: bool) -> u16 {
    if select {
        return input_b;
    }

    input_a
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

        self.get(address)
    }

    fn set(&mut self, input: u16, address: u16) {
        if address < 0b1 << 14 {
            self.ram[address as usize] = input;
        } else if address < 0b11 << 14 {
            self.screen[(address - 0b1 << 14) as usize] = input;
        }
    }

    fn get(&self, address: u16) -> u16 {
        if address < 0b1 << 14 {
            return self.ram[address as usize];
        } else if address < 0b11 << 14 {
            return self.screen[(address - 0b1 << 14) as usize];
        } else {
            return self.keyboard;
        }
    }
}
