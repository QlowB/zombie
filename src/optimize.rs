
use std::collections::BTreeMap;
use super::ir;
use super::ir::Instruction;
use typed_arena::Arena;

pub struct DfgOptimizer<'a> {
    dfg: DataflowGraph<'a>,
    tape_state: TapeState<'a>,
}

struct DataflowGraph<'a> {
    nodes: Arena<DfgNode<'a>>
}

enum DfgNode<'a> {
    Offset(i64),
    ConstAdd(&'a Cell<DfgNode<'a>>),
    Const(i64),
    AddMultiplied(&'a Cell<DfgNode<'a>>, i64, &'a Cell<DfgNode<'a>>),
}

struct TapeState<'a> {
    pub cell_states: BTreeMap<i64, DfgNode<'a>>
}

pub struct Optimizer {
}

impl TapeState<'a> {
    fn add(&'a mut self, offset: i64, value: i64) {
        if let Some(cell) = cell_state {
            let new_cell = match cell {
                CellState::Value(val) => CellState::Value(*val + value),
                CellState::Added(val) => CellState::Added(*val + value)
            };
            std::mem::replace(cell, new_cell);
        }
        else {
            self.cell_states.insert(offset, CellState::Added(value));
        }
    }

    fn set(&mut self, offset: i64, value: i64) {
        let cell_state = self.cell_states.get_mut(&offset);
        if let Some(cell) = cell_state {
            std::mem::replace(cell, CellState::Value(value));
        }
        else {
            self.cell_states.insert(offset, CellState::Value(value));
        }
    }

    fn get(&self, offset: i64) -> Option<&CellState> {
        self.cell_states.get(&offset)
    }
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
