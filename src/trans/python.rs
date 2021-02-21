use super::super::{ir, formatter, options};
use super::hex_bitmask;

use ir::Instruction;
use formatter::Formatter;
use options::*;

pub fn transpile(opts: &Options, instrs: &Vec<ir::Instruction>) -> String {
    let mut formatter = Formatter::new();

    formatter.add_line("import sys");
    formatter.add_line("mem = [0] * 0x10000");
    formatter.add_line("ptr = 0");

    generate(&mut formatter, instrs, opts);

    formatter.get_code()
}


fn generate(formatter: &mut Formatter, instrs: &Vec<Instruction>, opts: &Options) {

    let cell_mask = match opts.cell_size {
        CellSize::Bits(n) => {
            "0x".to_owned() + &hex_bitmask(n)
        },
        CellSize::Modular(n) => n.to_string(),
        CellSize::Int => "-1".to_owned()
    };
    for instr in instrs {
        match instr {
            Instruction::Nop => {},
            Instruction::Add{ offset, value } => {
                formatter.add_line(&format!("mem[(ptr + {}) & 0xFFFF] = (mem[(ptr + {}) & 0xFFFF] + {}) & {}", offset, offset, value, cell_mask));
            },
            Instruction::Set{ offset, value } => {
                formatter.add_line(&format!("mem[(ptr + {}) & 0xFFFF] = {} & {}", offset, value, cell_mask));
            },
            Instruction::LinearLoop{ offset, factors } => {
                for (off, factor) in factors {
                    formatter.add_line(&format!("mem[(ptr + {}) & 0xFFFF] = (mem[(ptr + {}) & 0xFFFF] + {} * mem[(ptr + {}) & 0xFFFF]) & {}",
                                                offset + off, offset + off, factor, offset, cell_mask));
                }
                formatter.add_line(&format!("mem[(ptr + {}) & 0xFFFF] = 0", offset));
            },
            Instruction::MovePtr(offset) => {
                formatter.add_line(&format!("ptr += {}", offset));
            },
            Instruction::Loop(instructions) => {
                formatter.add_line("while mem[ptr & 0xFFFF] != 0:");
                formatter.indent();
                generate(formatter, instructions, opts);
                formatter.unindent();
            },
            Instruction::Read(offset) => {
                formatter.add_line(&format!("mem[(ptr + {}) & 0xFFFF] = sys.stdin.buffer.read(1)", offset));
            },
            Instruction::Write(offset) => {
                formatter.add_line(&format!("sys.stdout.buffer.write(mem[(ptr + {}) & 0xFFFF].to_bytes(1, 'little'))", offset));
                formatter.add_line("sys.stdout.buffer.flush()");
            }
        }
    }
}