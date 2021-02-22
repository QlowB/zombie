use std::i64;
use super::{ir, optimize};
use std::io::{Write, Read};
use std::mem;
use super::ir::{ConstVisitor, Instruction};
//use mmap::{MemoryMap, MapOption};

/*#[cfg(target_os = "windows")]
use kernel32::VirtualAlloc;
#[cfg(target_os = "windows")]
use winapi::um::winnt::{MEM_COMMIT, PAGE_EXECUTE_READWRITE};*/

use dynasmrt::{DynasmApi, DynasmLabelApi};

pub enum Storation {
    Register(&'static str),
    Stack(i64),
}

pub fn compile_cfg<'a>(cfg: Vec<optimize::DfInstr<'a>>) -> Box<fn (*mut u8) -> ()> {
    Box::new(|_| {})
}


pub fn compile(instrs: &Vec<ir::Instruction>) -> Vec<u8> {
    let mut cg = CodeGenerator::new();
    cg.initialize();

    let entry = cg.buffer.offset();

    cg.visit_instructions(instrs);
    let _committed = cg.buffer.commit();
    cg.finalize();
    let buf = cg.buffer.finalize().unwrap();

    //let ret = buf.to_vec();
    //println!("{:02x?}", ret);

    let function: extern "C" fn(memory: *mut u8) -> bool = unsafe {
        //mem::transmute(cg.get_callable())
        mem::transmute(buf.ptr(entry))
    };

    //let mut data: Vec<u8> = Vec::with_capacity(100000);
    unsafe {
        //function(&mut *data.into_boxed_slice().as_mut_ptr() as *mut u8);
        let layout = std::alloc::Layout::new::<[u8; 0x10000]>();
        let mem = std::alloc::alloc_zeroed(layout);
        function(mem.offset(0x08000));
        std::alloc::dealloc(mem, layout);
    }

    //cg.into_vec()
    //ret
    vec![]
}

pub struct CodeGenerator {
    pub buffer: dynasmrt::x64::Assembler
}

impl CodeGenerator {
    pub fn new() -> Self {
        CodeGenerator {
            buffer: dynasmrt::x64::Assembler::new().unwrap()
        }
    }

    pub fn initialize(&mut self) {
        dynasm!(self.buffer
            ; xor rsi, rsi
        );
    }

    pub fn finalize(&mut self) {
        dynasm!(self.buffer
            ; ret
        );
    }

    /*#[cfg(target_os = "windows")]
    pub fn get_callable(self) -> *const u8 {
        let data = self.buffer.finalize().unwrap().to_vec();
        println!("asm buffer of size {}", data.len());
        //let ex = unsafe { VirtualAlloc(0 as _, data.len(), MEM_COMMIT, PAGE_EXECUTE_READWRITE) };
        let ex = unsafe {
            MemoryMap::new(
                data.len(),
                &[
                    MapOption::MapAddr(0 as *mut u8),
                    MapOption::MapOffset(0),
                    MapOption::MapFd(-1),
                    MapOption::MapReadable,
                    MapOption::MapWritable,
                    MapOption::MapExecutable,
                    MapOption::MapNonStandardFlags(libc::MAP_ANON),
                    MapOption::MapNonStandardFlags(libc::MAP_PRIVATE),
                ]
            )
        };
        unsafe {
            std::ptr::copy_nonoverlapping(data.as_ptr(), ex as _, data.len());
        }
        ex as _
    }

    #[cfg(not(target_os = "windows"))]
    pub fn get_callable(self) {
    }*/
}

impl ir::ConstVisitor for CodeGenerator {
    type Ret = ();

    fn visit_nop(&mut self, _nop: &Instruction) {
    }

    fn visit_add(&mut self, add: &'_ Instruction) {
        if let Instruction::Add{ offset, value } = add {
            dynasm!(self.buffer
                ; add BYTE [rdi + *offset as i32], *value as i8
            );
        }
    }

    fn visit_set(&mut self, set: &'_ Instruction) {
        if let Instruction::Set{ offset, value } = set {
            dynasm!(self.buffer
                ; mov BYTE [rdi + *offset as i32], *value as i8
            );
        }
    }

    fn visit_linear_loop(&mut self, l: &Instruction) {
        if let Instruction::LinearLoop{ offset: glob_offset, factors } = l {
            if factors.len() > 0 {
                dynasm!(self.buffer
                    ; movzx rcx, BYTE [rdi + *glob_offset as i32]
                );
            }
            for (&offset, &factor) in factors {
                if offset == 0 {
                    continue;
                }

                let absoff = offset + glob_offset;

                if factor == 0 {
                }
                else if factor == 1 {
                    dynasm!(self.buffer
                        ; add BYTE [rdi + absoff as i32], cl
                    );
                }
                else if factor == -1 {
                    dynasm!(self.buffer
                        ; sub BYTE [rdi + absoff as i32], cl
                    );
                }
                else if factor == 3 {
                    dynasm!(self.buffer
                        ; lea ebx, [rcx + rcx * 2]
                        ; add BYTE [rdi + absoff as i32], bl
                    );
                }
                else if factor == 5 {
                    dynasm!(self.buffer
                        ; lea ebx, [rcx + rcx * 4]
                        ; add BYTE [rdi + absoff as i32], bl
                    );
                }
                else if factor == 9 {
                    dynasm!(self.buffer
                        ; lea ebx, [rcx + rcx * 8]
                        ; add BYTE [rdi + absoff as i32], bl
                    );
                }
                else if factor.count_ones() == 1 {
                    dynasm!(self.buffer
                        ; mov bl, cl
                        ; shl bl, factor.trailing_zeros() as i8
                        ; add BYTE [rdi + absoff as i32], bl
                    );
                }
                else if (-factor).count_ones() == 1 {
                    dynasm!(self.buffer
                        ; mov bl, cl
                        ; shl bl, factor.trailing_zeros() as i8
                        ; sub BYTE [rdi + absoff as i32], bl
                    );
                }
                else {
                    dynasm!(self.buffer
                        ; mov al, factor as i8
                        ; mul cl
                        ; add BYTE [rdi + absoff as i32], al
                    );
                }
            }
            dynasm!(self.buffer
                ; mov BYTE [rdi + *glob_offset as i32], 0
            );
        }
    }

    fn visit_move_ptr(&mut self, mp: &Instruction) {
        if let Instruction::MovePtr(offset) = mp {
            dynasm!(self.buffer
                ; lea rdi, [rdi + *offset as i32]
            );
        }
    }

    fn visit_loop(&mut self, l: &Instruction) {
        if let Instruction::Loop(insts) = l {
            let begin = self.buffer.new_dynamic_label();
            let end = self.buffer.new_dynamic_label();
            dynasm!(self.buffer
                ; cmp BYTE [rdi], 0
                ; jz => end
                ; => begin
            );
            self.visit_instructions(insts);
            dynasm!(self.buffer
                ; cmp BYTE [rdi], 0
                ; jnz => begin
                ; => end
            );
        }
    }
    
    fn visit_read(&mut self, r: &Instruction) {
        if let Instruction::Read(offset) = r {
            dynasm!(self.buffer
                ; push rdi
                ; push rsi
                ; sub rsp, 24
                ; mov rax, QWORD readbyte as _
                ; call rax
                ; add rsp, 24
                ; pop rsi
                ; pop rdi
                ; mov BYTE [rdi + *offset as i32], al
            );
        }
    }

    fn visit_write(&mut self, w: &Instruction) {
        if let Instruction::Write(offset) = w {
            dynasm!(self.buffer
                ; push rdi
                ; push rsi
                ; sub rsp, 24
                ; mov dil, BYTE [rdi + *offset as i32]
                ; mov rax, QWORD putbyte as _
                ; call rax
                ; add rsp, 24
                ; pop rsi
                ; pop rdi
            );
        }
    }
}

extern "C" fn putbyte(chr: u8) {
    //print!("{:?}", chr as char);
    std::io::stdout().write(&[chr]).unwrap();
    std::io::stdout().flush().unwrap();
}

extern "C" fn readbyte() -> u8 {
    //let mut byte: u8 = 0;
    //std::io::stdin().read(&mut [byte]).unwrap();
    std::io::stdin().bytes().next().unwrap_or(Ok(0)).unwrap_or(0)
}

