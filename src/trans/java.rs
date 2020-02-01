use super::super::{ir, formatter};

use ir::Instruction;
use formatter::Formatter;

pub fn transpile(instrs: &Vec<ir::Instruction>) -> String {
    let mut formatter = Formatter::new();

    formatter.add_line("class Brainfuck {");
    formatter.indent();
    formatter.add_line("public static void main(String[] args) {");
    formatter.indent();
    formatter.add_line("byte[] mem = new byte[0x10000];");
    formatter.add_line("int ptr = 0;");
    formatter.add_line("");

    generate(&mut formatter, instrs);

    formatter.unindent();
    formatter.add_line("}");
    formatter.unindent();
    formatter.add_line("}");

    formatter.get_code()
}


fn generate(formatter: &mut Formatter, instrs: &Vec<Instruction>) {
    for instr in instrs {
        match instr {
            Instruction::Nop => {},
            Instruction::Add{ offset, value } => {
                formatter.add_line(&format!("mem[(ptr + {}) & 0xFFFF] += {};", offset, value));
            },
            Instruction::Set{ offset, value } => {
                formatter.add_line(&format!("mem[(ptr + {}) & 0xFFFF] = {};", offset, value));
            },
            Instruction::LinearLoop{ offset, factors } => {
                for (off, factor) in factors {
                    formatter.add_line(&format!("mem[(ptr + {}) & 0xFFFF] += {} * mem[(ptr + {}) & 0xFFFF];", offset + off, factor, offset));
                }
                formatter.add_line(&format!("mem[(ptr + {}) & 0xFFFF] = 0;", offset));
            },
            Instruction::MovePtr(offset) => {
                formatter.add_line(&format!("ptr += {};", offset));
            },
            Instruction::Loop(instructions) => {
                formatter.add_line("while(mem[ptr & 0xFFFF] != 0) {");
                formatter.indent();
                generate(formatter, instructions);
                formatter.unindent();
                formatter.add_line("}");
            },
            Instruction::Read(offset) => {
                formatter.add_line(&format!("mem[(ptr + {}) & 0xFFFF] = (byte) System.in.read();", offset));
            },
            Instruction::Write(offset) => {
                formatter.add_line(&format!("System.out.write(mem[(ptr + {}) & 0xFFFF]);", offset));
                formatter.add_line(&format!("System.out.flush();"));
            }
        }
    }
}