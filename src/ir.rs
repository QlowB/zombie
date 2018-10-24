
use std::collections::BTreeMap;

pub enum Instruction {
    Add{ offset: i64, value: i64 },
    LinearLoop{ offset: i64, increment: i64, factors: BTreeMap<i64, i64> },
    MovePtr(i64),
    Loop(Vec<Instruction>),
    Read(i64),
    Write(i64)
}





