use std::cell::{RefCell};
use std::collections::BTreeMap;
use super::{ir};
use super::ir::Instruction;
use typed_arena::Arena;

pub struct DfgOptimizer<'a> {
    arena: Arena<DfgNode<'a>>,
    cell_states: BTreeMap<i64, &'a DfgNode<'a>>,
    cfg: Vec<DfInstr<'a>>,
}

pub enum DfgNode<'a> {
    Cell(i64),
    Const(i64),
    Add(&'a DfgNode<'a>, &'a DfgNode<'a>),
    Multiply(&'a DfgNode<'a>, &'a DfgNode<'a>),
    Read(),
}

pub enum DfInstr<'a> {
    Print(&'a RefCell<DfgNode<'a>>),
    WriteMem(i64, &'a DfgNode<'a>),
    Loop(i64, Vec<DfInstr<'a>>),
}


impl<'a> DfgOptimizer<'a> {

    fn new() -> Self {
        DfgOptimizer {
            arena: Arena::new(),
            cell_states: BTreeMap::new(),
            cfg: Vec::new()
        }
    }

    fn get_cell<'b: 'a>(&'b self, offset: i64) -> &'a DfgNode<'a> {
        if let Some(cell) = self.cell_states.get(&offset) {
            cell
        }
        else {
            let off: &'b mut DfgNode<'a> = self.arena.alloc(DfgNode::Cell(offset));
            off
        }
    }
/*}

impl<'a> ir::MutVisitor<'a> for DfgOptimizer<'a> {
    type Ret = ();
*/
    fn visit_instructions(&'a mut self, instr: &mut Vec<Instruction>) {
        for inst in instr {
            self.walk_instruction(inst);
        }
    }

    #[allow(unused_variables)]
    fn visit_add(&mut self, add: &mut Instruction) {
        if let Instruction::Add{ offset, value } = add {
            let s: &'a _ = self;
            let cell = s.get_cell(*offset);
            let adder = self.arena.alloc(DfgNode::Const(*value));
            let addition = self.arena.alloc(DfgNode::Add(cell, adder));
        }
    }

    #[allow(unused_variables)]
    fn visit_set(&'a self, set: &mut Instruction) {
        if let Instruction::Set{ offset, value } = set {
            let arena: &'a _ = &self.arena;
            let setter: &'a DfgNode<'a> = arena.alloc(DfgNode::Const(*value));
            self.cell_states.insert(13, setter);
        }
    }

    fn visit_linear_loop(&self, _lloop: &'_ mut Instruction) {
    }

    fn visit_move_ptr(&self, _move_ptr: &'_ mut Instruction) {
    }

    fn visit_loop(&self, l: &mut Instruction) {
        if let Instruction::Loop(instrs) = l {
            let mut increments: BTreeMap<i64, i64> = BTreeMap::new();
            let mut dirty = false;

            let mut optimizer = DfgOptimizer::new();

            for inst in instrs {
                optimizer.walk_instruction(inst);
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

    fn visit_read(&self, _read: &'_ mut Instruction) {
    }

    fn visit_write(&self, _write: &'_ mut Instruction) {
    }

    fn walk_instruction(&mut self, inst: &mut Instruction) {
        use self::Instruction::*;
        match inst {
            Nop => {},
            Add { offset: _, value: _ } => self.visit_add(inst),
            Set { offset: _, value: _ } => self.visit_set(inst),
            LinearLoop { offset: _, factors: _ } => self.visit_linear_loop(inst),
            MovePtr(_) => self.visit_move_ptr(inst),
            Loop(_) => self.visit_loop(inst),
            Read(_) => self.visit_read(inst),
            Write(_) => self.visit_write(inst),
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
