use std::collections::BTreeMap;
use super::{ir};
use super::ir::Instruction;
use typed_arena::Arena;

pub struct DfgOptimizer<'a> {
    arena: &'a Arena<DfgNode<'a>>,
    pub cell_states: BTreeMap<i64, &'a DfgNode<'a>>,
    pub cfg: Vec<DfInstr<'a>>,
}

pub struct BasicBlock<'a> {
    arena: &'a Arena<DfgNode<'a>>,
    pub cell_states: BTreeMap<i64, &'a DfgNode<'a>>,
    pub cfg: Vec<DfInstr<'a>>,
}

pub enum DfgNode<'a> {
    Cell(i64),
    Const(i64),
    Add(&'a DfgNode<'a>, &'a DfgNode<'a>),
    Multiply(&'a DfgNode<'a>, &'a DfgNode<'a>),
    Read(),
}

pub enum DfInstr<'a> {
    Print(&'a DfgNode<'a>),
    WriteMem(i64, &'a DfgNode<'a>),
    MovePtr(i64),
    Loop(i64, Vec<DfInstr<'a>>),
}


impl<'a> DfgOptimizer<'a> {

    fn new(arena: &'a Arena<DfgNode<'a>>) -> Self {
        DfgOptimizer {
            arena: arena,
            cell_states: BTreeMap::new(),
            cfg: Vec::new()
        }
    }

    fn get_cell(&self, offset: i64) -> &'a DfgNode<'a> {
        if let Some(cell) = self.cell_states.get(&offset) {
            cell
        }
        else {
            self.arena.alloc(DfgNode::Cell(offset))
        }
    }

    fn set_cell(&mut self, offset: i64, cell: &'a DfgNode<'a>) {
        self.cell_states.insert(offset, cell);
    }
}

impl<'a> ir::MutVisitor for DfgOptimizer<'a> {
    type Ret = ();

    fn visit_instructions(&mut self, instr: &mut Vec<Instruction>) {
        for inst in instr {
            self.walk_instruction(inst);
        }
    }

    #[allow(unused_variables)]
    fn visit_add(&mut self, add: &mut Instruction) {
        if let Instruction::Add{ offset, value } = add {
            let cell = self.get_cell(*offset);
            let adder = self.arena.alloc(DfgNode::Const(*value));
            let addition = self.arena.alloc(DfgNode::Add(cell, adder));
            self.set_cell(*offset, addition);
        }
    }

    #[allow(unused_variables)]
    fn visit_set(&mut self, set: &mut Instruction) {
        if let Instruction::Set{ offset, value } = set {
            let setter: &'a DfgNode<'a> = self.arena.alloc(DfgNode::Const(*value));
            self.cell_states.insert(*offset, setter);
        }
    }

    fn visit_linear_loop(&mut self, lloop: &mut Instruction) {
        if let Instruction::LinearLoop{ offset, factors } = lloop {
            let multiplier = self.get_cell(*offset);
            for (off, fact) in factors {
                if *fact == 1 {
                    self.set_cell(*offset + *off, multiplier);
                }
                else {
                    let factor2 = self.arena.alloc(DfgNode::Const(*fact));
                    let prod = self.arena.alloc(DfgNode::Multiply(multiplier, factor2));
                    self.set_cell(*offset + *off, prod);
                }
            }
            self.set_cell(*offset, self.arena.alloc(DfgNode::Const(0)));
        }
    }

    fn visit_move_ptr(&mut self, move_ptr: &'_ mut Instruction) {
        if let Instruction::MovePtr(val) = move_ptr {
            self.cfg.push(DfInstr::MovePtr(*val));

            let mut new_map = BTreeMap::new();
            for (off, v) in &self.cell_states {
                new_map.insert(*off - *val, *v);
            }
            std::mem::swap(&mut self.cell_states, &mut new_map);
        }
    }

    fn visit_loop(&mut self, l: &mut Instruction) {
        if let Instruction::Loop(instrs) = l {
            let arena = Arena::new();
            let mut optimizer = DfgOptimizer::new(&arena);
            optimizer.visit_instructions(instrs);
            self.cfg.push(DfInstr::Loop(0, optimizer.cfg));
            self.cell_states.clear();
            self.set_cell(0, self.arena.alloc(DfgNode::Const(0)));
        }
    }

    fn visit_read(&mut self, read: &'_ mut Instruction) {
        if let Instruction::Read(off) = read {
            self.set_cell(*off, self.arena.alloc(DfgNode::Read()));
        }
    }

    fn visit_write(&mut self, write: &'_ mut Instruction) {
        if let Instruction::Write(off) = write {
            self.cfg.push(DfInstr::Print(self.get_cell(*off)));
        }
    }
}




struct MemoryState {
    cellStates: BTreeMap<i64, CellState>,
    default_cell: CellState
}

enum CellState {
    Unknown,
    Const(i64),
    OtherCell(usize),
    Sum(usize, usize),
    Prod(usize, usize),
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
            // also copy the instruction list (essentially push the optimizer state on a stack
            // for the loop)
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
