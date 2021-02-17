
use super::super::{ir, formatter, optimize};

use ir::Instruction;
use ir::ConstVisitor;
use formatter::Formatter;
use optimize::{DfInstr, DfgNode};

struct CTranspiler {
    pub code_buf: Formatter
}

fn eval(dn: &DfgNode) -> String {
    match dn {
        DfgNode::Const(c) => {
            format!("{}", c)
        },
        DfgNode::Cell(off) => {
            format!("mem[OFF({})]", off)
        },
        DfgNode::Add(a, b) => {
            format!("{} + {}", eval(a), eval(b))
        },
        DfgNode::Multiply(a, b) => {
            format!("({}) * ({})", eval(a), eval(b))
        },
        DfgNode::Read() => {
            format!("getchar()")
        }
    }
}

pub fn transpile_dfg(dfg: &optimize::DfgOptimizer) -> String {
    let mut formatter = Formatter::new();
    formatter.add_line(r#"#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <inttypes.h>

#define OFF(X) ((uint16_t)(ptr + (uint16_t) (X)))

int main() {
    uint8_t* mem = (uint8_t*) calloc(0x10000, 1);
    uint16_t ptr = 0;"#);
    formatter.indent();
    generate_dfg(&dfg.cfg, &mut formatter);
    formatter.unindent();
    formatter.add_line("}");

    formatter.get_code()
}

fn generate_dfg(cfg: &Vec<DfInstr>, formatter: &mut Formatter) {
    let mut memoffs: Vec<(i64, u64)> = Vec::new();
    let mut tmp_counter: u64 = 0;
    let offset_creator = |o: i64| {
        if o < 0 {
            format!("minus_{}", -o)
        }
        else {
            format!("{}", o)
        }
    };
    for stmt in cfg {
        match stmt {
            DfInstr::MovePtr(off) => {
                formatter.add_line(&format!("ptr += {};", off));
            },
            DfInstr::WriteMem(off, val) => {
                formatter.add_line(&format!("uint8_t tmp_{} = {};", tmp_counter, eval(val)));
                memoffs.push((*off, tmp_counter));
                tmp_counter += 1;
            },
            DfInstr::Print(val) => {
                formatter.add_line(&format!("putchar({});", eval(val)));
            },
            DfInstr::Loop(_val, instrs) => {
                for (off, tmp) in &memoffs {
                    formatter.add_line(&format!("mem[OFF({})] = tmp_{};", *off, *tmp));
                }
                memoffs.clear();
                formatter.add_line("while(mem[OFF(0)]) {");
                formatter.indent();
                generate_dfg(&instrs, formatter);
                formatter.unindent();
                formatter.add_line("}");
            },
        }
    }
    for (off, tmp) in memoffs {
        formatter.add_line(&format!("mem[OFF({})] = tmp_{};", off, tmp));
    }
}


pub fn transpile(instrs: &Vec<ir::Instruction>) -> String {
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
#include <string.h>
#include <inttypes.h>

#define OFF(X) ((uint16_t)(ptr + (uint16_t) (X)))

int main() {
    uint8_t* mem = (uint8_t*) malloc(0x10000 * sizeof(uint8_t));
    memset(mem, 0, 0x10000 * sizeof(uint8_t));
    uint16_t ptr = 0;"#);
        transpiler.code_buf.indent();
        transpiler
    }
}

impl CTranspiler {
    pub fn finalize(&mut self) {
        self.code_buf.add_line("free(mem);");
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
            self.code_buf.add_line(&format!("mem[OFF({})] += {};", offset, value));
        }
    }

    fn visit_set(&mut self, set: &'_ Instruction) {
        if let Instruction::Set{ offset, value } = set {
            self.code_buf.add_line(&format!("mem[OFF({})] = {};", offset, value));
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
                    self.code_buf.add_line(&format!("mem[OFF({})] += mem[OFF({})];", glob_offset + offset, glob_offset));
                }
                else if factor == -1 {
                    self.code_buf.add_line(&format!("mem[OFF({})] -= mem[OFF({})];", glob_offset + offset, glob_offset));
                }
                else {
                    self.code_buf.add_line(&format!("mem[OFF({})] += {} * mem[OFF({})];", glob_offset + offset, factor, glob_offset));
                }
            }
            self.code_buf.add_line(&format!("mem[OFF({})] = 0;", glob_offset));
        }
    }

    fn visit_move_ptr(&mut self, mp: &Instruction) {
        if let Instruction::MovePtr(offset) = mp {
            self.code_buf.add_line(&format!("ptr = OFF({});", offset));
        }
    }

    fn visit_loop(&mut self, l: &Instruction) {
        if let Instruction::Loop(insts) = l {
            if insts.len() == 1 {
                if let Instruction::MovePtr(1) = insts[0] {
                    //self.code_buf.add_line("printf(\"strlen(%s, %d)\\n\", buffer, strlen(buffer));");
                    self.code_buf.add_line("ptr = OFF(strlen(&mem[ptr]));");
                    return;
                }
            }
            self.code_buf.add_line("while(mem[OFF(0)]) {");
            self.code_buf.indent();
            self.visit_instructions(insts);
            self.code_buf.unindent();
            self.code_buf.add_line("}");
        }
    }
    
    fn visit_read(&mut self, r: &Instruction) {
        if let Instruction::Read(offset) = r {
            self.code_buf.add_line(&format!("mem[OFF({})] = getchar();", offset));
        }
    }

    fn visit_write(&mut self, w: &Instruction) {
        if let Instruction::Write(offset) = w {
            self.code_buf.add_line(&format!("putchar(mem[OFF({})]);", offset));
        }
    }
}