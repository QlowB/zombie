


pub mod c;
pub mod java;
pub mod python;
pub mod zombie_ir;



fn hex_bitmask(bits: usize) -> String {
    let fs = bits / 4;
    let leftover = bits % 4;

    match leftover {
        1 => "1",
        2 => "3",
        3 => "7",
        _ => ""
    }.to_owned() + &"F".repeat(fs)
}
