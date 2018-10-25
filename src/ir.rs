use std::collections::BTreeMap;

#[derive(Debug)]
pub enum Instruction {
    Add{ offset: i64, value: i64 },
    LinearLoop(BTreeMap<i64, i64>),
    MovePtr(i64),
    Loop(Vec<Instruction>),
    Read(i64),
    Write(i64)
}



pub trait MutVisitor {
    type Ret: Default;

    fn visit_instructions(&mut self, instr: &mut Vec<Instruction>) {
        for inst in instr {
            self.walk_instruction(inst);
        }
    }

    fn visit_add(&mut self, add: &mut Instruction) -> Self::Ret {
        Self::Ret::default()
    }

    fn visit_linear_loop(&mut self, lloop: &mut Instruction) -> Self::Ret {
        Self::Ret::default()
    }

    fn visit_move_ptr(&mut self, move_ptr: &mut Instruction) -> Self::Ret {
        Self::Ret::default()
    }

    fn visit_loop(&mut self, l: &mut Instruction) -> Self::Ret {
        if let Instruction::Loop(instrs) = l {
            self.visit_instructions(instrs);
        }
        Self::Ret::default()
    }

    fn visit_read(&mut self, read: &mut Instruction) -> Self::Ret {
        Self::Ret::default()
    }
    
    fn visit_write(&mut self, write: &mut Instruction) -> Self::Ret {
        Self::Ret::default()
    }

    fn walk_instruction(&mut self, inst: &mut Instruction) -> Self::Ret {
        use self::Instruction::*;
        match inst {
            Add {offset: _, value: _} => self.visit_add(inst),
            LinearLoop (_) => self.visit_linear_loop(inst),
            MovePtr(_) => self.visit_move_ptr(inst),
            Loop(_) => self.visit_loop(inst),
            Read(_) => self.visit_read(inst),
            Write(_) => self.visit_write(inst),
        }
    }
}


