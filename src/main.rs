#![feature(plugin)]
#![plugin(dynasm)]

//#[macro_use]
//extern crate dynasm;

use std::io::{self, Read};
use std::fs::File;
use std::env;

pub mod ir;
pub mod parser;
pub mod interpret;
pub mod optimize;
pub mod compile;

use crate::ir::MutVisitor;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut buffer = String::new();

    if args.len() > 1 {
        File::open(&args[1])?.read_to_string(&mut buffer)?;
    }
    else {
        io::stdin().read_to_string(&mut buffer)?;
    }

    let insts = parser::parse(&buffer);
    if let Ok(mut insts) = insts {
        let mut lin_loop_optimizer = optimize::LoopLinearizer;
        //println!("code: {:#?}", insts);
        lin_loop_optimizer.visit_instructions(&mut insts);
        //println!("code: {:#?}", insts);

        //interpret::run(&insts);
        let code = compile::compile(insts);
        
        //println!("{:?}", code.into_iter().map(|x| format!("{:x}", x)).collect::<Vec<String>>());
    }
    else if let Err(msg) = insts {
        println!("error parsing: {}", msg);
    }

    Ok(())
}
