use super::ir::Instruction;
use std::io::Read;
use std::io::Write;
use std::io;

struct Data {
    memory: Vec<u8>,
    ptr: i64,
}

pub fn run(instructions: &Vec<Instruction>) {
    let len = 1024;
    let mut data = Data {
        memory: vec![0; len],
        ptr: 0,
    };

    run_instrs(instructions, &mut data);
}

fn run_instrs(instructions: &Vec<Instruction>, data: &mut Data) {
    let len = data.memory.len();
    for inst in instructions {
        match inst {
            Instruction::Add{ offset, value } => {
                let cell = &mut data.memory[(data.ptr + offset) as usize % len];
                *cell = cell.wrapping_add(*value as u8);
            },
            Instruction::MovePtr(offset) => {
                data.ptr = data.ptr.wrapping_add(*offset);
            },
            Instruction::Loop(instrs) => {
                while data.memory[data.ptr as usize % len] != 0 {
                    run_instrs(instrs, data);
                }
            },
            Instruction::Read(offset) => {
                let cell = &mut data.memory[(data.ptr + offset) as usize % len];
                *cell = io::stdin().bytes().next().unwrap_or(Ok(0)).unwrap_or_default();
            },
            Instruction::Write(offset) => {
                let cell = data.memory[(data.ptr + offset) as usize % len];
                io::stdout().write(&[cell]).unwrap();
                io::stdout().flush().unwrap();
            }
            _ => {}
        }
    }
}


