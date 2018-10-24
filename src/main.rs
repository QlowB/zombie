use std::io::{self, Read};

pub mod ir;
pub mod parser;
pub mod interpret;

fn main() -> io::Result<()> {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;

    let insts = parser::parse(&buffer);
    if let Ok(insts) = insts {
        interpret::run(&insts);
    }
    else if let Err(msg) = insts {
        println!("error parsing: {}", msg);
    }

    Ok(())
}
