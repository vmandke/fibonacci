use crate::stack::Stack;
use std::mem::transmute;
use std::cell::Cell;
use std::ptr;

std::arch::global_asm!(include_str!("asm_aarch64_aapcs_macho.S"));




//#[link(name = "asm", kind = "static")]
extern "C" {
    pub fn bootstrap_green_task();
    pub fn swap_registers(out_regs: *mut Registers, in_regs: *const Registers);
}


#[repr(C)]
#[derive(Debug, Clone)]
pub struct Registers {
    gpr: [usize; 32],
}

impl Registers {
    pub fn new() -> Registers {
        Registers { gpr: [0; 32] }
    }

    #[inline]
    #[cfg(test)]
    pub fn load(to_context: &Registers) {
        let mut cur = Registers::new();
        let regs: &Registers = &to_context;

        unsafe { swap_registers(&mut cur, regs) }
    }
}

thread_local! {
    // each thread has it's own generator context stack
    static GEN_REGISTER_P: Cell<*mut Registers> = const { Cell::new(ptr::null_mut()) };
    static MAIN_REGISTER_P: Cell<*mut Registers> = const { Cell::new(ptr::null_mut()) };
}

pub struct RegistersContext {
    pub(crate) regs: *mut Registers,
}

impl RegistersContext {
    pub fn main_context() -> RegistersContext {
        let mut root: *mut Registers = MAIN_REGISTER_P.get();
        if root.is_null() {
            root = {
                let mut root = Box::new(Registers::new());
                Box::leak(root)
            };
            MAIN_REGISTER_P.set(root);
        }
        RegistersContext { regs: root }
    }

    pub fn gen_context() -> RegistersContext {
        let mut root: *mut Registers = GEN_REGISTER_P.get();
        if root.is_null() {
            root = {
                let mut root = Box::new(Registers::new());
                Box::leak(root)
            };
            GEN_REGISTER_P.set(root);
        }
        RegistersContext { regs: root }
    }
}



pub type InitFn = extern "C" fn(usize, *mut usize) -> !;


pub extern "C" fn gen_init(a1: usize, a2: *mut usize) -> ! {
    println!("gen_init: a1: {}, a2: {:?}", a1, a2);
    let func: fn() = unsafe { transmute(a2) };
    func();
    unreachable!("Should never comeback");
}

#[inline]
fn align_down(sp: *mut usize) -> *mut usize {
    let sp = (sp as usize) & !(16 - 1);
    sp as *mut usize
}


pub fn initialize_call_frame(
    regs: &mut Registers,
    fptr: InitFn,
    arg: usize,
    arg2: *mut usize,
    stack: &Stack,
) {
    // Callee-saved registers start at x19
    const X19: usize = 19 - 19;
    const X20: usize = 20 - 19;
    const X21: usize = 21 - 19;

    const FP: usize = 29 - 19;
    const LR: usize = 30 - 19;
    const SP: usize = 31 - 19;

    let sp = align_down(stack.end());

    // These registers are frobbed by bootstrap_green_task into the right
    // location so we can invoke the "real init function", `fptr`.
    regs.gpr[X19] = arg;
    regs.gpr[X20] = arg2 as usize;
    regs.gpr[X21] = fptr as usize;

    // Aarch64 current stack frame pointer
    regs.gpr[FP] = sp as usize;

    regs.gpr[LR] = bootstrap_green_task as usize;

    // setup the init stack
    // this is prepared for the swap context
    regs.gpr[SP] = sp as usize;
}


#[cfg(test)]
mod test {
    use std::mem::transmute;
    use crate::{registers::{initialize_call_frame, swap_registers, Registers}, stack};

    fn init_fn_impl(arg: usize, f: *mut usize) -> ! {
        let func: fn() = unsafe { transmute(f) };
        func();
        
        let ctx: &Registers = unsafe { transmute(arg) };
        Registers::load(ctx);

        unreachable!("Should never comeback");
    }
    #[cfg(target_arch = "aarch64")]
    extern "C" fn init_fn(arg: usize, f: *mut usize) -> ! {
        init_fn_impl(arg, f)
    }


    #[test]
    fn test_swap_context() {
        static mut VAL: bool = false;

        fn callback() {
            unsafe { VAL = true };
        }

        let stk = unsafe { stack::Stack::new(1024) };
        let mut cur_regs = Registers::new();
        let mut arg_reg = Registers::new();

        // init the stack 
        let offset = stk.get_offset();
        unsafe { *offset = 1 };
        let arg_ptr = &arg_reg as *const _ as usize;
        initialize_call_frame(&mut cur_regs, init_fn, arg_ptr, callback as *mut usize, &stk);
        unsafe { swap_registers(&mut arg_reg as *mut Registers, &mut cur_regs as *mut Registers) };
        unsafe {
            assert!(VAL);
        }
    }
}