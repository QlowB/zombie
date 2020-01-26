
use super::{ir, optimize};

use ir::Instruction;
use ir::ConstVisitor;

struct Transpiler {
    pub code: String
}


pub fn transpile_c(instrs: &Vec<ir::Instruction>) -> String {
    let mut transpiler = Transpiler::default();
    transpiler.visit_instructions(instrs);
    transpiler.finalize();
    return transpiler.code;
}


impl Default for Transpiler {
    fn default() -> Self {
        let mut transpiler = Transpiler{ code: "".to_string() };

        transpiler.code += r#"#include <stdio.h>
#include <stdlib.h>
int main() {
    unsigned char* buffer = calloc(2000000000, 1);
    buffer += 1000000000;
"#;
        transpiler
    }
}

impl Transpiler {
    pub fn finalize(&mut self) {
        self.code += "}\n";
    }
}


impl ir::ConstVisitor for Transpiler {
    type Ret = ();

    fn visit_nop(&mut self, nop: &Instruction) {
        self.code += "\n";
    }

    fn visit_add(&mut self, add: &'_ Instruction) {
        if let Instruction::Add{ offset, value } = add {
            self.code += &format!("buffer[{}] += {};\n", offset, value);
        }
    }

    fn visit_set(&mut self, set: &'_ Instruction) {
        if let Instruction::Set{ offset, value } = set {
            self.code += &format!("buffer[{}] = {};\n", offset, value);
        }
    }

    fn visit_linear_loop(&mut self, l: &Instruction) {
        if let Instruction::LinearLoop{ offset: glob_offset, factors } = l {
            for (&offset, &factor) in factors {
                if offset == 0 {
                    continue;
                }

                if factor == 0 {
                }
                else if factor == 1 {
                    self.code += &format!("buffer[{}] += buffer[0];\n", glob_offset + offset);
                }
                else if factor == -1 {
                    self.code += &format!("buffer[{}] -= buffer[0];\n", glob_offset + offset);
                }
                else {
                    self.code += &format!("buffer[{}] += {} * buffer[0];\n", glob_offset + offset, factor);
                }
            }
            self.code += "buffer[0] = 0;\n";
        }
    }

    fn visit_move_ptr(&mut self, mp: &'_ Instruction) {
        if let Instruction::MovePtr(offset) = mp {
            self.code += &format!("buffer += {};\n", offset);
        }
    }

    fn visit_loop(&mut self, l: &Instruction) {
        if let Instruction::Loop(insts) = l {
            self.code += "while(buffer[0]) {\n";
            self.visit_instructions(insts);
            self.code += "}\n";
        }
    }
    
    fn visit_read(&mut self, r: &Instruction) {
        if let Instruction::Read(offset) = r {
            self.code += &format!("buffer[{}] = getchar();\n", offset);
        }
    }

    fn visit_write(&mut self, w: &Instruction) {
        if let Instruction::Write(offset) = w {
            self.code += &format!("putchar(buffer[{}]);\n", offset);
        }
    }
}