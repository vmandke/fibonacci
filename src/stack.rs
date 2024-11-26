use std::ptr;
use std::os::raw::c_void;


pub struct Stack {
    top: *mut usize,
    bottom: *mut usize,
}

const MAP_STACK: libc::c_int = 0;

impl Stack {

    pub unsafe fn new(size: usize) -> Stack {
        const NULL: *mut libc::c_void = 0 as *mut libc::c_void;
        const PROT: libc::c_int = libc::PROT_READ | libc::PROT_WRITE;
        const TYPE: libc::c_int = libc::MAP_PRIVATE | libc::MAP_ANON | MAP_STACK;

        let min_page_size = 16384 as usize;
        
        let bytes = usize::max(size * std::mem::size_of::<usize>(), min_page_size);

        let bytes = bytes << 1;

        let ptr = libc::mmap(NULL, bytes, PROT, TYPE, -1, 0);
        if ptr == libc::MAP_FAILED {
            panic!("Failed to allocate stack");
        }
        
        let top = (ptr as usize + bytes) as *mut usize;
        let bottom = ptr as *mut usize;
        let old_len = top as usize - bottom as usize;

        libc::mprotect(bottom as *mut c_void, min_page_size, libc::PROT_NONE);
        let bottom = (bottom as usize + min_page_size) as *mut c_void;


        let stk = Stack { top, bottom: bottom as *mut usize };
        let stk_len = stk.top as usize - stk.bottom as usize;
        
        debug_assert!(stk_len % min_page_size == 0 && stk_len != 0);
        
        // Confirm alignment before writing bytes
        assert!(stk.bottom as usize % std::mem::align_of::<usize>() == 0);
        
        // Write bytes safely
        ptr::write_bytes(stk.bottom as *mut usize, 0xEE, std::mem::size_of::<usize>());
        stk
    }

    pub fn get_offset(&self) -> *mut usize {
        unsafe { (self.top as *mut usize).offset(-1) }
    }

    pub fn end(&self) -> *mut usize {
        let offset = self.get_offset();
        unsafe { (self.top as *mut usize).offset(0 - *offset as isize) }
    }

    pub fn len(&self) -> usize {
        self.top as usize - self.bottom as usize
    }
}   