use std::collections::BTreeMap;
use super::ir;

pub fn parse(code: &str) -> Result<Vec<ir::Instruction>, &str> {
    let mut ptr: i64 = 0;
    let mut add_map: BTreeMap<i64, i64> = BTreeMap::new();
    let mut instruction_stack: Vec<Vec<ir::Instruction>> = Vec::new();
    let mut instructions: Vec<ir::Instruction> = Vec::new();

    let implement = |add_map: &mut BTreeMap<i64, i64>, instructions: &mut Vec<ir::Instruction>, ptr: &mut i64| {
        for (&offset, &value) in add_map.iter() {
            instructions.push(ir::Instruction::Add{ value, offset });
        }
        add_map.clear();
        if *ptr != 0 {
            instructions.push(ir::Instruction::MovePtr(*ptr));
            *ptr = 0;
        }
    };

    for c in code.chars() {
        match c {
            '+' => {
                match add_map.get_mut(&ptr) {
                    Some(entry) => { *entry = entry.wrapping_add(1); },
                    None => { add_map.insert(ptr, 1); }
                }
            },
            '-' => {
                match add_map.get_mut(&ptr) {
                    Some(entry) => { *entry = entry.wrapping_sub(1); },
                    None => { add_map.insert(ptr, -1); }
                }
            },
            '>' => { ptr += 1; },
            '<' => { ptr -= 1; },
            '.' => {
                implement(&mut add_map, &mut instructions, &mut ptr);
                instructions.push(ir::Instruction::Write(ptr));
            },
            ',' => {
                implement(&mut add_map, &mut instructions, &mut ptr);
                instructions.push(ir::Instruction::Read(ptr));
            },
            '[' => {
                implement(&mut add_map, &mut instructions, &mut ptr);
                instruction_stack.push(instructions);
                instructions = Vec::new();
            },
            ']' => {
                implement(&mut add_map, &mut instructions, &mut ptr);
                let top = instruction_stack.pop();
                if let Some(mut inst) = top {
                    inst.push(ir::Instruction::Loop(instructions));
                    instructions = inst;
                }
                else {
                    // error, too many ']'
                    return Err("found ']' without matching '['");
                }
            },

            _ => {}
        }
    }

    if instruction_stack.len() > 0 {
        return Err("found '[' without matching ']'");
    }

    Ok(instructions)
}




