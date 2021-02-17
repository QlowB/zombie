use super::super::{ir, formatter};

use ir::Instruction;
use formatter::Formatter;

pub fn transpile(instrs: &Vec<ir::Instruction>) -> String {
    let mut formatter = Formatter::new();

    formatter.add_line("import sys");
    formatter.add_line("mem = [0] * 0x10000");
    formatter.add_line("ptr = 0");

    generate(&mut formatter, instrs);

    formatter.get_code()
}


fn generate(formatter: &mut Formatter, instrs: &Vec<Instruction>) {
    for instr in instrs {
        match instr {
            Instruction::Nop => {},
            Instruction::Add{ offset, value } => {
                formatter.add_line(&format!("mem[(ptr + {}) & 0xFFFF] = (mem[(ptr + {}) & 0xFFFF] + {}) & 0xFF", offset, offset, value));
            },
            Instruction::Set{ offset, value } => {
                formatter.add_line(&format!("mem[(ptr + {}) & 0xFFFF] = {}", offset, value));
            },
            Instruction::LinearLoop{ offset, factors } => {
                for (off, factor) in factors {
                    formatter.add_line(&format!("mem[(ptr + {}) & 0xFFFF] = (mem[(ptr + {}) & 0xFFFF] + {} * mem[(ptr + {}) & 0xFFFF]) & 0xFF",
                                                offset + off, offset + off, factor, offset));
                }
                formatter.add_line(&format!("mem[(ptr + {}) & 0xFFFF] = 0", offset));
            },
            Instruction::MovePtr(offset) => {
                formatter.add_line(&format!("ptr += {}", offset));
            },
            Instruction::Loop(instructions) => {
                formatter.add_line("while mem[ptr & 0xFFFF] != 0:");
                formatter.indent();
                generate(formatter, instructions);
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