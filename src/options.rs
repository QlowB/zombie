use std::str::FromStr;

#[derive(PartialEq, Clone)]
pub enum CellLayout {
    Trusting,
    Wrapping,
    Unbounded
}

#[derive(PartialEq, Clone)]
pub enum CellSize {
    Bits(usize),
    Modular(u64),
    Int
}


pub struct Options {
    pub cell_layout: CellLayout,
    pub memory_size: usize,
    pub cell_size: CellSize,
}


impl Default for Options {
    fn default() -> Self {
        Options {
            cell_layout: CellLayout::Trusting,
            memory_size: 0x10000,
            cell_size: CellSize::Bits(8),
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
        let integer = s.parse::<usize>();
        match integer {
            Ok(i) => Ok(CellSize::Bits(i)),
            Err(e) => match s {
                "8" => Ok(CellSize::Bits(8)),
                "16" => Ok(CellSize::Bits(16)),
                "32" => Ok(CellSize::Bits(16)),
                "int" => Ok(CellSize::Int),
                _ => Err("invalid cell size"),
            }
        }
    }
}
