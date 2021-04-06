#[repr(C)]
pub struct Registers {
    rax: u64,
    rbx: u64,
    rcx: u64,
    rdx: u64,
    rsi: u64,
    rdi: u64,
    rsp: u64,
    rip: u64,

    r: [u64; 8],
}

impl Registers {
    fn dump(&mut self) {
        unsafe {
            asm!{
            );
        }
    }
}
