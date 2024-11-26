use std::{mem::transmute, sync::{Arc, Mutex}, ptr};

mod registers;
use registers::{initialize_call_frame, gen_init, Registers, swap_registers, RegistersContext};
use stack::Stack;
mod stack;


pub fn pause() {
    let gen_reg_context = RegistersContext::gen_context();
    let main_reg_context = RegistersContext::main_context();
    let main_register = unsafe { &mut *main_reg_context.regs };
    let gen_register = unsafe { &mut *gen_reg_context.regs };
    unsafe { swap_registers(gen_register as *mut Registers, main_register as *mut Registers) };
}


pub fn resume() {
    let gen_reg_context = RegistersContext::gen_context();
    let main_reg_context = RegistersContext::main_context();
    let main_register = unsafe { &mut *main_reg_context.regs };
    let gen_register = unsafe { &mut *gen_reg_context.regs };
    unsafe { swap_registers(main_register as *mut Registers, gen_register as *mut Registers) };
}



fn main() {
    if !cfg!(target_os = "macos") {
        panic!("This code is only supported on MacOS");
    }
    if !cfg!(target_arch = "aarch64") {
        panic!("This code is only supported aarch64");
    }

    let gen_stack = unsafe { Stack::new(1024) };

    // init the stack 
    let offset = gen_stack.get_offset();
    unsafe { *offset = 1 };

    fn fibo() {
        unsafe {
            let mut a = 0;
            let mut b = 1;
            loop {
                println!("Generated elem ::: {}", a);
                let tmp = a;
                a = b;
                b = tmp + b;
                // Switch to the main context
                pause();
            }
        };
    }


    let gen_reg_context = RegistersContext::gen_context();
    let gen_register = unsafe { &mut *gen_reg_context.regs };
    initialize_call_frame(gen_register, gen_init, 0, fibo as *mut usize, &gen_stack);
    // Switch to the generator context
    resume();


    for _ in 0..10 {
        println!("Trying to resume the coroutine");
        // Switch to the generator context
        resume();
    }
}
