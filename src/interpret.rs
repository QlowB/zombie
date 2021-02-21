use super::ir::Instruction;
use super::options;
use super::options::Options;
use std::io::Read;
use std::io::Write;
use std::io;
use std::ops::{Add, Mul, Sub};
use std::num::Wrapping;


trait FromNum {
    fn from(n: i64) -> Self;
}
trait CellWrite {
    fn write<S: Write>(&self, s: &mut S);
}
trait CellRead {
    fn read<R: Read>(&mut self, r: &mut R);
}

impl FromNum for Wrapping<u8> {
    fn from(n: i64) -> Self { Wrapping(n as u8) }
}
impl CellWrite for Wrapping<u8> {
    fn write<S: Write>(&self, s: &mut S) {
        s.write(&[self.0]).unwrap();
        s.flush().unwrap();
    }
}
impl CellRead for Wrapping<u8> {
    fn read<R: Read>(&mut self, r: &mut R) {
        let mut bytes: [u8; 1] = [0];
        if r.read(&mut bytes).is_ok() {
            *self = Wrapping(bytes[0]);
        }
        else {
            *self = Wrapping(0);
        }
    }
}


impl FromNum for Wrapping<u16> {
    fn from(n: i64) -> Self { Wrapping(n as u16) }
}
impl CellWrite for Wrapping<u16> {
    fn write<S: Write>(&self, s: &mut S) {
        write!(s, "{}", self.0);
        s.flush().unwrap();
    }
}
impl CellRead for Wrapping<u16> {
    fn read<R: Read>(&mut self, r: &mut R) {
        let mut bytes: [u8; 1] = [0];
        if r.read(&mut bytes).is_ok() {
            *self = Wrapping(bytes[0] as _);
        }
        else {
            *self = Wrapping(0);
        }
    }
}

impl FromNum for i64 {
    fn from(n: i64) -> Self { n as _ }
}
impl CellWrite for i64 {
    fn write<S: Write>(&self, s: &mut S) {
        s.write(&[*self as u8]).unwrap();
        s.flush().unwrap();
    }
}
impl CellRead for i64 {
    fn read<R: Read>(&mut self, r: &mut R) {
        let mut bytes: [u8; 1] = [0];
        if r.read(&mut bytes).is_ok() {
            *self = bytes[0] as _;
        }
        else {
            *self = 0;
        }
    }
}







struct Data<T> {
    memory: Vec<T>,
    ptr: i64,
}

pub fn run(instructions: &Vec<Instruction>, opts: &Options) {
    if opts.cell_size == options::CellSize::Bits(8) {
        let mut data = Data::<Wrapping<u8>> {
            memory: vec![Wrapping(0); opts.memory_size as usize],
            ptr: 0,
        };
        run_with_funcs(instructions, &mut data, &|a, b| a + b, &|a, b| a * b);
    }
    else if opts.cell_size == options::CellSize::Bits(16) {
        let mut data = Data::<Wrapping<u16>> {
            memory: vec![Wrapping(0); opts.memory_size as usize],
            ptr: 0,
        };
        run_with_funcs(instructions, &mut data, &|a, b| a + b, &|a, b| a * b);
    }
    else {
        match opts.cell_size {
            options::CellSize::Modular(n) => {
                let n = n as i64;
                let mut data = Data::<i64> {
                    memory: vec![0; opts.memory_size as usize],
                    ptr: 0,
                };
                run_with_funcs(instructions, &mut data, &|a, b| (a + b) % n, &|a, b| (a * b) % n);
            },
            _ => {}
        }
    }
}


fn run_instrs<T>(instructions: &Vec<Instruction>, data: &mut Data<T>)
where
T: Add<Output=T> + Sub<Output=T> + Mul<Output=T>,
T: Copy + Eq + CellWrite + CellRead + FromNum
{
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
                cell.read(&mut io::stdin());
            },
            Instruction::Write(offset) => {
                let cell = data.memory[(data.ptr + offset) as usize % len];
                cell.write(&mut io::stdout());
            },
            Instruction::LinearLoop{ offset: glob_offset, factors } => {
                //assert_eq!(factors.get(&0), Some(&-1));
                let multiplicator = data.memory[((data.ptr + glob_offset) as usize) % len];
                for (offset, value) in factors {
                    let cell = &mut data.memory[(data.ptr + offset + glob_offset) as usize % len];
                    *cell = *cell + (multiplicator * T::from(*value));
                }
                data.memory[((data.ptr + glob_offset) as usize) % len] = T::from(0);
            },
        }
    }
}


fn run_with_funcs<T>(instructions: &Vec<Instruction>,
                 data: &mut Data<T>,
                 add: &dyn Fn(T, T) -> T,
                 mul: &dyn Fn(T, T) -> T)
where
T: Copy + Eq + CellWrite + CellRead + FromNum
{
    let len = data.memory.len();
    for inst in instructions {
        match inst {
            Instruction::Nop => {},
            Instruction::Add{ offset, value } => {
                let cell = &mut data.memory[(data.ptr + offset) as usize % len];
                *cell = add(*cell, T::from(*value));
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
                    run_with_funcs(instrs, data, add, mul);
                }
            },
            Instruction::Read(offset) => {
                let cell = &mut data.memory[(data.ptr + offset) as usize % len];
                cell.read(&mut io::stdin());
            },
            Instruction::Write(offset) => {
                let cell = data.memory[(data.ptr + offset) as usize % len];
                cell.write(&mut io::stdout());
            },
            Instruction::LinearLoop{ offset: glob_offset, factors } => {
                //assert_eq!(factors.get(&0), Some(&-1));
                let multiplicator = data.memory[((data.ptr + glob_offset) as usize) % len];
                for (offset, value) in factors {
                    let cell = &mut data.memory[(data.ptr + offset + glob_offset) as usize % len];
                    *cell = add(*cell, mul(multiplicator, T::from(*value)));
                }
                data.memory[((data.ptr + glob_offset) as usize) % len] = T::from(0);
            },
        }
    }
}


