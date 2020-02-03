//#![feature(plugin)]
//#![plugin(dynasm)]
#![feature(proc_macro_hygiene)]
#[macro_use]
extern crate dynasm;

extern crate winapi;
extern crate typed_arena;

use std::io::{self, Read};
use std::fs::File;
use std::env;

pub mod ir;
pub mod parser;
pub mod interpret;
pub mod optimize;
pub mod compile;
pub mod formatter;
pub mod trans;


use crate::ir::MutVisitor;
use typed_arena::Arena;

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
        let mut lin_loop_optimizer = optimize::LinOptimizer::new();
        lin_loop_optimizer.visit_instructions(&mut insts);
        std::mem::replace(&mut insts, lin_loop_optimizer.instructions);
        
        //for ref inst in &insts {
            //println!("{}\n", inst.to_string());
        //}
        //println!("{}", trans::java::transpile(&insts));
        let arena = Arena::new();
        let dfg = optimize::create_dfg(&mut insts, &arena);
        let c = trans::c::transpile_dfg(&dfg);
        println!("{}", c);
        //let _code = compile::compile(&insts);
    }
    else if let Err(msg) = insts {
        println!("error parsing: {}", msg);
    }

    Ok(())
}
