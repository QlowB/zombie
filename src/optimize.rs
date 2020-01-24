use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use super::{ir, compile};
use super::ir::Instruction;
use typed_arena::Arena;

pub struct DfgOptimizer<'a> {
    dfg: DataflowGraph<'a>,
    cell_states: BTreeMap<i64, DfgNode<'a>>,
    cfg: Vec<DfInstr<'a>>,
}

struct DataflowGraph<'a> {
    arena: Arena<DfgNode<'a>>,
}

pub struct DfgNode<'a> {
    storation: Option<compile::Storation>,
    kind: DfgNodeKind<'a>,
}

pub enum DfgNodeKind<'a> {
    Offset(i64),
    ConstAdd(&'a DfgNode<'a>),
    Const(i64),
    AddMultiplied(&'a DfgNode<'a>, i64, &'a DfgNode<'a>),
    Read(),
}

pub enum DfInstr<'a> {
    Print(&'a RefCell<DfgNode<'a>>),
    WriteMem(i64, &'a DfgNode<'a>),
    Loop(i64, Vec<DfInstr<'a>>),
}

impl<'a> DfgNode<'a> {
    pub fn new(kind: DfgNodeKind<'a>) -> Self {
        DfgNode {
            storation: None,
            kind: kind
        }
    }
}

impl<'a> DfgOptimizer<'a> {
}

impl<'a> ir::MutVisitor for DfgOptimizer<'a> {
    type Ret = ();

    fn visit_instructions(&mut self, instr: &mut Vec<Instruction>) {
        for inst in instr {
            self.walk_instruction(inst);
        }
    }

    fn visit_add(&mut self, add: &mut Instruction) {
        if let Instruction::Add{ offset, value } = add {
            let arena = &self.dfg.arena;
            let load = RefCell::new(arena.alloc(DfgNode::new(DfgNodeKind::Offset(*offset))));
            //let addition: &'a _ = arena.alloc(DfgNode::new(DfgNodeKind::ConstAdd(load)));
            //self.cfg.push(DfInstr::WriteMem(*offset, addition));
        }
    }

    fn visit_set(&mut self, set: &mut Instruction) {
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
            // set cell at offset 0 to 0
        }
    }

    fn visit_read(&mut self, read: &'_ mut Instruction) {
    }

    fn visit_write(&mut self, write: &'_ mut Instruction) {
    }
}
























pub struct Optimizer {
}

impl Optimizer {
    pub fn new() -> Self {
        Optimizer {
        }
    }
}

impl ir::MutVisitor for Optimizer {
    type Ret = ();

    fn visit_instructions(&mut self, instr: &mut Vec<Instruction>) {
        for inst in instr {
            self.walk_instruction(inst);
        }
    }

    fn visit_add(&mut self, add: &mut Instruction) {
    }

    fn visit_set(&mut self, set: &mut Instruction) {
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

            if !dirty && increments.len() == 1 {
                if let Some(&v) = increments.get(&0) {
                    if v % 2 != 0 {
                        // cases like [-]
                        // also [---]
                        std::mem::replace(l, Instruction::Set{ offset: 0, value: 0 });
                    }
                }
            }
            else if !dirty && increments.get(&0) == Some(&-1) {
                std::mem::replace(l, Instruction::LinearLoop(increments));
            }

            // set cell at offset 0 to 0
        }
    }

    fn visit_read(&mut self, read: &'_ mut Instruction) {
    }

    fn visit_write(&mut self, write: &'_ mut Instruction) {
    }
}
