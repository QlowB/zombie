use super::super::{ir, formatter};

use ir::Instruction;
use formatter::Formatter;

pub fn transpile(instrs: &Vec<ir::Instruction>) -> String {
    let mut formatter = Formatter::new();
    generate(&mut formatter, instrs);
    formatter.get_code()
}


fn generate(formatter: &mut Formatter, instrs: &Vec<Instruction>) {
    for instr in instrs {
        match instr {
            Instruction::Nop => {},
            Instruction::Add{ offset, value } => {
                formatter.add_line(&format!("@{} += {}", offset, value));
            },
            Instruction::Set{ offset, value } => {
                formatter.add_line(&format!("@{} = {})", offset, value));
            },
            Instruction::LinearLoop{ offset, factors } => {
                for (off, factor) in factors {
                    formatter.add_line(&format!("@{} = {} * @{}", offset + off, factor, offset));
                }
                formatter.add_line(&format!("@{} = 0 // End LL", offset));
            },
            Instruction::MovePtr(offset) => {
                formatter.add_line(&format!("ptr += {}", offset));
            },
            Instruction::Loop(instructions) => {
                formatter.add_line("Loop {");
                formatter.indent();
                generate(formatter, instructions);
                formatter.unindent();
                formatter.add_line("}");
            },
            Instruction::Read(offset) => {
                formatter.add_line(&format!("Read(@{})", offset));
            },
            Instruction::Write(offset) => {
                formatter.add_line(&format!("Write(@{})", offset));
            }
        }
    }
}