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
                std::mem::replace(l, Instruction::LinearLoop{ offset: 0, factors: increments });
            }
            // set cell at offset 0 to 0
        }
    }

    fn visit_read(&mut self, read: &'_ mut Instruction) {
    }

    fn visit_write(&mut self, write: &'_ mut Instruction) {
    }
}
























pub struct LinOptimizer {
    offset: i64,
    pub instructions: Vec<Instruction>
}

impl LinOptimizer {
    pub fn new() -> Self {
        LinOptimizer {
            offset: 0,
            instructions: Vec::new()
        }
    }
}

impl ir::MutVisitor for LinOptimizer {
    type Ret = Option<Instruction>;

    fn visit_instructions(&mut self, instrs: &mut Vec<Instruction>) {
        for inst in instrs {
            self.walk_instruction(inst);
        }
    }

    fn visit_add(&mut self, add: &mut Instruction) -> Self::Ret {
        if let Instruction::Add{ offset, value } = add {
            self.instructions.push(Instruction::Add{ offset: *offset + self.offset, value: *value });
        }
        None
    }

    fn visit_set(&mut self, set: &mut Instruction) -> Self::Ret {
        if let Instruction::Set{ offset, value } = set {
            self.instructions.push(Instruction::Set{ offset: *offset + self.offset, value: *value });
        }
        None
    }

    fn visit_linear_loop(&mut self, lloop: &mut Instruction) -> Self::Ret {
        self.instructions.push(std::mem::replace(lloop, Instruction::Nop));
        None
    }

    fn visit_move_ptr(&mut self, move_ptr: &mut Instruction) -> Self::Ret {
        if let Instruction::MovePtr(offset) = move_ptr {
            self.offset += *offset;
        }
        None
    }

    fn visit_loop(&mut self, l: &mut Instruction) -> Self::Ret {
        if let Instruction::Loop(instrs) = l {
            let mut increments: BTreeMap<i64, i64> = BTreeMap::new();
            let mut dirty = false;

            // pointer movement to be added in case this loop cannot be linearized
            let offset_before = self.offset;
            self.offset = 0;

            let mut swap: Vec<Instruction> = Vec::new();
            std::mem::swap(&mut self.instructions, &mut swap);

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
            if self.offset != 0 {
                self.instructions.push(Instruction::MovePtr(self.offset));
                self.offset = 0;
            }
            std::mem::swap(&mut self.instructions, &mut swap);

            if !dirty && increments.len() == 1 {
                self.offset = offset_before;
                if let Some(&v) = increments.get(&0) {
                    if v % 2 != 0 {
                        // cases like [-]
                        // also [---]
                        self.instructions.push(Instruction::Set{ offset: self.offset, value: 0 });
                    }
                }
            }
            else if !dirty && increments.get(&0) == Some(&-1) {
                self.offset = offset_before;
                self.instructions.push(Instruction::LinearLoop{ offset: self.offset, factors: increments });
            }
            else {
                if offset_before != 0 {
                    self.instructions.push(Instruction::MovePtr(offset_before));
                }
                self.instructions.push(Instruction::Loop(swap));
            }
            // set cell at offset 0 to 0
        }
        None
    }

    fn visit_read(&mut self, read: &mut Instruction) -> Self::Ret {
        if let Instruction::Read(offset) = read {
            self.instructions.push(Instruction::Read(*offset + self.offset));
        }
        None
    }

    fn visit_write(&mut self, write: &'_ mut Instruction) -> Self::Ret {
        if let Instruction::Write(offset) = write {
            self.instructions.push(Instruction::Write(*offset + self.offset));
        }
        None
    }
}
