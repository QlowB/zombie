use std::collections::BTreeMap;

#[derive(Debug)]
pub enum Instruction {
    Nop,
    Add{ offset: i64, value: i64 },
    Set{ offset: i64, value: i64 },
    LinearLoop(BTreeMap<i64, i64>),
    MovePtr(i64),
    Loop(Vec<Instruction>),
    Read(i64),
    Write(i64)
}

impl Instruction {
    pub fn to_string(&self) -> String {
        use self::Instruction::*;
        match self {
            Nop => "Nop".to_string(),
            Add{ offset, value } => {
                if *offset == 0 {
                    if *value == 1 {
                        "Inc".to_string()
                    }
                    else if *value == -1 {
                        "Dec".to_string()
                    }
                    else {
                        format!("Add(@{}, {})", offset, value)
                    }
                } 
                else {
                    format!("Add(@{}, {})", offset, value)
                }
            },
            Set{ offset, value } => format!("Set(@{}, {})", offset, value),
            LinearLoop(map) => {
                "LinearLoop".to_string()
            },
            MovePtr(val) => format!("MovePtr({})", val),
            Loop(instrs) => {
                let mut ret = "[\n".to_string();
                for instr in instrs {
                    ret += &instr.to_string();
                    ret += "\n";
                }
                ret += "]\n";
                ret
            },
            Read(offset) => format!("Read(@{})", offset),
            Write(offset) => format!("Read(@{})", offset),
        }
    }
}

pub trait MutVisitor {
    type Ret: Default;

    fn visit_instructions(&mut self, instr: &mut Vec<Instruction>) {
        for inst in instr {
            self.walk_instruction(inst);
        }
    }

    fn visit_nop(&mut self, nop: &mut Instruction) -> Self::Ret {
        Self::Ret::default()
    }

    fn visit_add(&mut self, add: &mut Instruction) -> Self::Ret {
        Self::Ret::default()
    }

    fn visit_set(&mut self, add: &mut Instruction) -> Self::Ret {
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
            Nop => self.visit_nop(inst),
            Add {offset: _, value: _} => self.visit_add(inst),
            Set {offset: _, value: _} => self.visit_set(inst),
            LinearLoop (_) => self.visit_linear_loop(inst),
            MovePtr(_) => self.visit_move_ptr(inst),
            Loop(_) => self.visit_loop(inst),
            Read(_) => self.visit_read(inst),
            Write(_) => self.visit_write(inst),
        }
    }
}

pub trait ConstVisitor {
    type Ret: Default;

    fn visit_instructions(&mut self, instr: &Vec<Instruction>) {
        for inst in instr {
            self.walk_instruction(inst);
        }
    }

    fn visit_nop(&mut self, nop: &Instruction) -> Self::Ret {
        Self::Ret::default()
    }

    fn visit_add(&mut self, add: &Instruction) -> Self::Ret {
        Self::Ret::default()
    }

    fn visit_set(&mut self, add: &Instruction) -> Self::Ret {
        Self::Ret::default()
    }

    fn visit_linear_loop(&mut self, lloop: &Instruction) -> Self::Ret {
        Self::Ret::default()
    }

    fn visit_move_ptr(&mut self, move_ptr: &Instruction) -> Self::Ret {
        Self::Ret::default()
    }

    fn visit_loop(&mut self, l: &Instruction) -> Self::Ret {
        if let Instruction::Loop(instrs) = l {
            self.visit_instructions(instrs);
        }
        Self::Ret::default()
    }

    fn visit_read(&mut self, read: &Instruction) -> Self::Ret {
        Self::Ret::default()
    }
    
    fn visit_write(&mut self, write: &Instruction) -> Self::Ret {
        Self::Ret::default()
    }

    fn walk_instruction(&mut self, inst: &Instruction) -> Self::Ret {
        use self::Instruction::*;
        match inst {
            Nop => self.visit_nop(inst),
            Add {offset: _, value: _} => self.visit_add(inst),
            Set {offset: _, value: _} => self.visit_set(inst),
            LinearLoop (_) => self.visit_linear_loop(inst),
            MovePtr(_) => self.visit_move_ptr(inst),
            Loop(_) => self.visit_loop(inst),
            Read(_) => self.visit_read(inst),
            Write(_) => self.visit_write(inst),
        }
    }
}


