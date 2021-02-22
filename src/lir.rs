

pub struct Lir {
    values: Vec<Value>
}

pub enum Value {
    Cell(i64),
    Const(i64),
    Operation(Operation),
}

enum Operation {
    Add(usize, usize),
    Sub(usize, usize),
    Mul(usize, usize)
}

