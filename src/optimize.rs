
use std::collections::BTreeMap;
use super::ir;
use super::ir::Instruction;


pub struct LoopLinearizer;

impl ir::MutVisitor for LoopLinearizer {
    type Ret = ();

    fn visit_instructions(&mut self, instr: &'_ mut Vec<Instruction>) {
        for inst in instr {
            self.walk_instruction(inst);
        }
    }

    fn visit_add(&mut self, add: &'_ mut Instruction) {
    }

    fn visit_linear_loop(&mut self, lloop: &'_ mut Instruction) {
    }

    fn visit_move_ptr(&mut self, move_ptr: &'_ mut Instruction) {
    }

    fn visit_loop(&mut self, l: &mut Instruction) {
        if let Instruction::Loop(instrs) = l {
            let mut increments: BTreeMap<i64, i64> = BTreeMap::new();
            let mut dirty = false;
            for inst in instrs {
                self.walk_instruction(inst);
                if !dirty {
                    use super::ir::Instruction::*;
                    match inst {
                        Add { offset, value } => {
                            match increments.get_mut(offset) {
                                Some(v) => *v += *value,
                                None => { increments.insert(*offset, *value); },
                            }
                        },
                        _ => {
                            dirty = true;
                        }
                    }
                }
            }

            if !dirty && increments.get(&0) == Some(&-1) {
                std::mem::replace(l, Instruction::LinearLoop(increments));
            }
        }
    }

    fn visit_read(&mut self, read: &'_ mut Instruction) {
    }

    fn visit_write(&mut self, write: &'_ mut Instruction) {
    }
}
