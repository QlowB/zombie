use super::ir::Instruction;
use std::io::Read;
use std::io::Write;
use std::io;
use std::ops::{Add, Mul, Sub};
use std::num::Wrapping;

struct Data<T: Add + Mul + Sub + Eq + From<i64>> {
    memory: Vec<T>,
    ptr: i64,
}

pub fn run(instructions: &Vec<Instruction>) {
    let len = 1024;
    let mut data = Data<Wrapping<u8>> {
        memory: vec![0; len],
        ptr: 0,
    };

    run_instrs(instructions, &mut data);
}

fn run_instrs<T: Add + Mul + Sub + Eq + From<i64>>(instructions: &Vec<Instruction>, data: &mut Data<T>) {
    let len = data.memory.len();
    for inst in instructions {
        match inst {
            Instruction::Nop => {},
            Instruction::Add{ offset, value } => {
                let cell = &mut data.memory[(data.ptr + offset) as usize % len];
                *cell = *cell + T::from(*value);
            },
            Instruction::Set{ offset, value } => {
                let cell = &mut data.memory[(data.ptr + offset) as usize % len];
                *cell = T::from(*value);
            },
            Instruction::MovePtr(offset) => {
                data.ptr = data.ptr.wrapping_add(*offset);
            },
            Instruction::Loop(instrs) => {
                while data.memory[data.ptr as usize % len] != T::from(0) {
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
            },
            Instruction::LinearLoop{ offset: glob_offset, factors } => {
                assert_eq!(factors.get(&0), Some(&-1));
                let multiplicator = data.memory[((data.ptr + glob_offset) as usize) % len];
                for (offset, value) in factors {
                    let cell = &mut data.memory[(data.ptr + offset + glob_offset) as usize % len];
                    *cell = cell + (multiplicator * (*value as _));
                }
            },
        }
    }
}


