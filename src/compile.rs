use std::i64;
use super::ir;
use std::io::{Write, Read};
use std::mem;
use super::ir::{MutVisitor, Instruction};

#[cfg(target_os = "windows")]
use winapi::um::memoryapi::VirtualAlloc;
#[cfg(target_os = "windows")]
use winapi::um::winnt::{MEM_COMMIT, PAGE_EXECUTE_READWRITE};

use dynasmrt::{DynasmApi, DynasmLabelApi};

pub fn compile(mut instrs: Vec<ir::Instruction>) -> Vec<u8> {
    let mut cg = CodeGenerator::new();
    cg.initialize();

    let entry = cg.buffer.offset();

    cg.visit_instructions(&mut instrs);
    cg.buffer.commit();
    cg.finalize();
    let buf = cg.buffer.finalize().unwrap();

    //let ret = buf.to_vec();
    //println!("{:02x?}", ret);

    let function: extern "C" fn(memory: *mut u8) -> bool = unsafe {
        //mem::transmute(cg.get_callable())
        mem::transmute(buf.ptr(entry))
    };

    let mut data: Vec<u8> = Vec::with_capacity(100000);
    unsafe {
        //function(&mut *data.into_boxed_slice().as_mut_ptr() as *mut u8);
        function(std::alloc::alloc_zeroed(std::alloc::Layout::new::<[u8; 1000]>()));
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

    #[cfg(target_os = "windows")]
    pub fn get_callable(self) -> *const u8 {
        let data = self.buffer.finalize().unwrap().to_vec();
        println!("asm buffer of size {}", data.len());
        let ex = unsafe { VirtualAlloc(0 as _, data.len(), MEM_COMMIT, PAGE_EXECUTE_READWRITE) };
        unsafe {
            std::ptr::copy_nonoverlapping(data.as_ptr(), ex as _, data.len());
        }
        ex as _
    }

    #[cfg(not(target_os = "windows"))]
    pub fn get_callable(self) {
    }
}

impl ir::MutVisitor for CodeGenerator {
    type Ret = ();

    fn visit_nop(&mut self, nop: &mut Instruction) {
    }

    fn visit_add(&mut self, add: &'_ mut Instruction) {
        if let Instruction::Add{ offset, value } = add {
            dynasm!(self.buffer
                ; add BYTE [rdi + rsi + *offset as i32], *value as i8
            );
        }
    }

    fn visit_set(&mut self, set: &'_ mut Instruction) {
        if let Instruction::Set{ offset, value } = set {
            dynasm!(self.buffer
                ; mov BYTE [rdi + rsi + *offset as i32], *value as i8
            );
        }
    }

    fn visit_linear_loop(&mut self, l: &mut Instruction) {
        if let Instruction::LinearLoop(factors) = l {
            if factors.len() > 1 ||
                factors.len() >= 1 && !factors.contains_key(&0) {
                dynasm!(self.buffer
                    ; mov cl, BYTE [rdi + rsi]
                );
            }
            for (&offset, &mut factor) in factors {
                if offset == 0 {
                    continue;
                }

                if factor == 0 {
                }
                else if factor == 1 {
                    dynasm!(self.buffer
                        ; add BYTE [rdi + rsi + offset as i32], cl
                    );
                }
                else if factor == -1 {
                    dynasm!(self.buffer
                        ; sub BYTE [rdi + rsi + offset as i32], cl
                    );
                }
                else if factor.count_ones() == 1 {
                    dynasm!(self.buffer
                        ; mov bl, cl
                        ; shl bl, factor.trailing_zeros() as i8
                        ; add BYTE [rdi + rsi + offset as i32], bl
                    );
                }
                else if (-factor).count_ones() == 1 {
                    dynasm!(self.buffer
                        ; mov bl, cl
                        ; shl bl, factor.trailing_zeros() as i8
                        ; sub BYTE [rdi + rsi + offset as i32], bl
                    );
                }
                else {
                    dynasm!(self.buffer
                        ; mov al, factor as i8
                        ; mul cl
                        ; add BYTE [rdi + rsi + offset as i32], al
                    );
                }
            }
            dynasm!(self.buffer
                ; mov BYTE [rdi + rsi], 0
            );
        }
    }

    fn visit_move_ptr(&mut self, mp: &'_ mut Instruction) {
        if let Instruction::MovePtr(offset) = mp {
            dynasm!(self.buffer
                ; add rsi, DWORD *offset as i32
            );
        }
    }

    fn visit_loop(&mut self, l: &mut Instruction) {
        if let Instruction::Loop(insts) = l {
            let begin = self.buffer.new_dynamic_label();
            let end = self.buffer.new_dynamic_label();
            dynasm!(self.buffer
                ; mov al, BYTE [rdi + rsi]
                ; test al, al
                ; jz => end
                ; => begin
            );
            self.visit_instructions(insts);
            dynasm!(self.buffer
                ; mov al, BYTE [rdi + rsi]
                ; test al, al
                ; jnz => begin
                ; => end
            );
        }
    }
    
    fn visit_read(&mut self, r: &mut Instruction) {
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
                ; mov BYTE [rdi + rsi + *offset as i32], al
            );
        }
    }

    fn visit_write(&mut self, w: &mut Instruction) {
        if let Instruction::Write(offset) = w {
            dynasm!(self.buffer
                ; push rdi
                ; push rsi
                ; sub rsp, 24
                ; mov dil, BYTE [rdi + rsi + *offset as i32]
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
    std::io::stdout().write(&[chr]);
}

extern "C" fn readbyte() -> u8 {
    let mut byte: u8 = 0;
    //std::io::stdin().read(&mut [byte]).unwrap();
    std::io::stdin().bytes().next().unwrap_or(Ok(0)).unwrap_or(0)
}

