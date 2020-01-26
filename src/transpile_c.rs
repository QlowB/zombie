
use super::{ir, optimize, formatter};

use ir::Instruction;
use ir::ConstVisitor;
use formatter::Formatter;

struct CTranspiler {
    pub code_buf: Formatter
}


pub fn transpile_c(instrs: &Vec<ir::Instruction>) -> String {
    let mut transpiler = CTranspiler::default();
    transpiler.visit_instructions(instrs);
    transpiler.finalize();
    return transpiler.code_buf.get_code();
}


impl Default for CTranspiler {
    fn default() -> Self {
        let mut transpiler = CTranspiler{ code_buf: Formatter::new() };

        transpiler.code_buf.add_line(r#"#include <stdio.h>
#include <stdlib.h>


int main() {
    unsigned char* restrict buffer = (unsigned char*) calloc(2000000000, 1);
    buffer += 1000000000;"#);
        transpiler.code_buf.indent();
        transpiler
    }
}

impl CTranspiler {
    pub fn finalize(&mut self) {
        self.code_buf.unindent();
        self.code_buf.add_line("}");
    }
}


impl ir::ConstVisitor for CTranspiler {
    type Ret = ();

    fn visit_nop(&mut self, nop: &Instruction) {
        self.code_buf.add_line("");
    }

    fn visit_add(&mut self, add: &'_ Instruction) {
        if let Instruction::Add{ offset, value } = add {
            self.code_buf.add_line(&format!("buffer[{}] += {};", offset, value));
        }
    }

    fn visit_set(&mut self, set: &'_ Instruction) {
        if let Instruction::Set{ offset, value } = set {
            self.code_buf.add_line(&format!("buffer[{}] = {};", offset, value));
        }
    }

    fn visit_linear_loop(&mut self, l: &Instruction) {
        if let Instruction::LinearLoop(factors) = l {
            for (&offset, &factor) in factors {
                if offset == 0 {
                    continue;
                }

                if factor == 0 {
                }
                else if factor == 1 {
                    self.code_buf.add_line(&format!("buffer[{}] += buffer[0];", offset));
                }
                else if factor == -1 {
                    self.code_buf.add_line(&format!("buffer[{}] -= buffer[0];", offset));
                }
                else {
                    self.code_buf.add_line(&format!("buffer[{}] += {} * buffer[0];", offset, factor));
                }
            }
            self.code_buf.add_line("buffer[0] = 0;");
        }
    }

    fn visit_move_ptr(&mut self, mp: &'_ Instruction) {
        if let Instruction::MovePtr(offset) = mp {
            self.code_buf.add_line(&format!("buffer += {};", offset));
        }
    }

    fn visit_loop(&mut self, l: &Instruction) {
        if let Instruction::Loop(insts) = l {
            self.code_buf.add_line("while(buffer[0]) {");
            self.code_buf.indent();
            self.visit_instructions(insts);
            self.code_buf.unindent();
            self.code_buf.add_line("}");
        }
    }
    
    fn visit_read(&mut self, r: &Instruction) {
        if let Instruction::Read(offset) = r {
            self.code_buf.add_line(&format!("buffer[{}] = getchar();", offset));
        }
    }

    fn visit_write(&mut self, w: &Instruction) {
        if let Instruction::Write(offset) = w {
            self.code_buf.add_line(&format!("putchar(buffer[{}]);", offset));
        }
    }
}