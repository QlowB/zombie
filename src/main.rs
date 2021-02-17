//#![feature(plugin)]
//#![plugin(dynasm)]
#![feature(proc_macro_hygiene)]
#[macro_use]
extern crate dynasm;

extern crate winapi;
extern crate typed_arena;

use std::io::{self, Read};
use std::fs::File;
use clap::{Arg, App};

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
    let matches = App::new("Zombie")
        .version("0.1.0")
        .author("Nicolas Winkler <nicolas.winkler@gmx.ch>")
        .about("Brainfuck interpreter and transpiler")
        .arg(Arg::with_name("input")
                 .index(1)
                 .takes_value(true)
                 .help("Input file"))
        .arg(Arg::with_name("transpile")
                 .long("transpile")
                 .short("t")
                 .takes_value(true)
                 .help("Transpile to language"))
        .get_matches();
    let mut buffer = String::new();

    if let Some(input) = matches.value_of("input") {
        File::open(&input)?.read_to_string(&mut buffer)?;
    }
    else {
        io::stdin().read_to_string(&mut buffer)?;
    }

    let insts = parser::parse(&buffer);
    if let Ok(mut insts) = insts {
        let mut lin_loop_optimizer = optimize::LinOptimizer::new();
        lin_loop_optimizer.visit_instructions(&mut insts);
        let _ = std::mem::replace(&mut insts, lin_loop_optimizer.instructions);
        
        //for ref inst in &insts {
            //println!("{}\n", inst.to_string());
        //}
        //println!("{}", trans::java::transpile(&insts));
        //let c = trans::c::transpile_dfg(&dfg);
        //println!("{}", c);
        //println!("{}", trans::java::transpile(&insts));
        
        match matches.value_of("transpile") {
            Some(lang) => {
                let arena = Arena::new();
                let dfg = optimize::create_dfg(&mut insts, &arena);
                let code = if lang == "c" {
                    //trans::c::transpile_dfg(&dfg)
                    trans::c::transpile(&insts)
                } else if lang == "java" {
                    trans::java::transpile(&insts)
                } else if lang == "python" {
                    trans::python::transpile(&insts)
                } else if lang == "zombie_ir" {
                    trans::zombie_ir::transpile(&insts)
                } else {
                    "".to_owned()
                };
                println!("{}", code);
            },
            None => {
                let _code = compile::compile(&insts);
            }
        }
    }
    else if let Err(msg) = insts {
        println!("error parsing: {}", msg);
    }

    Ok(())
}
