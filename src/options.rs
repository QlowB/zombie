use std::str::FromStr;

pub enum CellLayout {
    Trusting,
    Wrapping
}


pub enum CellSize {
    Int8,
    Int16,
    Int32,
    Int
}


pub struct Options {
    pub cell_layout: CellLayout,
    pub memory_size: i64,
    pub cell_size: CellSize,
}


impl Default for Options {
    fn default() -> Self {
        Options {
            cell_layout: CellLayout::Trusting,
            memory_size: 0x10000,
            cell_size: CellSize::Int8,
        }
    }
}

impl FromStr for CellLayout {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "trusting" => Ok(CellLayout::Trusting),
            "wrapping" => Ok(CellLayout::Wrapping),
            _ => Err("invalid cell layout"),
        }
    }
}

impl FromStr for CellSize {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "8" => Ok(CellSize::Int8),
            "16" => Ok(CellSize::Int16),
            "32" => Ok(CellSize::Int32),
            "int" => Ok(CellSize::Int),
            _ => Err("invalid cell size"),
        }
    }
}
