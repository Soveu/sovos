#![no_std]
#![no_main]

#![feature(asm)]
#![feature(asm_sym)]
#![feature(bench_black_box)]
#![feature(naked_functions)]
#![feature(extern_types)]

#[panic_handler]
fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

static STR: [u8; 13] = *b"ayyyyyyyyyyyy";
static mut MUT: [u8; 1 << 18] = [b'a'; 1 << 18];
static mut ZEROED: [u8; 1 << 18] = [0u8; 1 << 18];

#[no_mangle]
#[naked]
pub unsafe extern "sysv64" fn _start() -> ! {
    asm!("
        # Check if there is an additional value from the interrupt
        test sp, 15
        jnz 1f

        pop rax

    1:
        # Pop the old ip
        pop rax

        # Pop the code segment
        pop rax

        # Pop flags
        pop rbx

        # Clear all the flags
        xor rbx, rbx
        push rbx

        # Push the previous code segment
        push rax

        # Well, x86 needs that special rip + SYMBOL notation for PIC
        # What it really means is just `lea rbx [SYMBOL]`, but in a position-
        # independent fashion
        lea rbx, [rip + {}]
        push rbx

        # This kernel should be jumped into via page fault
        iretq

        # This exists just for testing purposes
        lea rax, [rip + __KERNEL_BASE]
        lea rax, [rip + .text]
        ",
        sym kmain,
        options(noreturn),
    )
}

extern {
    type LinkerSymbol;
}

impl LinkerSymbol {
    fn as_ptr(&'static self) -> *const u8 {
        self as *const Self as *const u8
    }
}

extern "C" {
    static __KERNEL_BASE: LinkerSymbol;
}

extern "sysv64" fn kmain() -> ! {
    unsafe {
        let mut base = __KERNEL_BASE.as_ptr();
        asm!("mov rax, rax", inout("rax") base, options(nostack, nomem));

        core::hint::black_box(&STR);
        MUT[0] = b'b';
        core::hint::black_box(&mut MUT);
        core::hint::black_box(&mut ZEROED);
    }

    loop {
        unsafe {
            asm!("hlt", options(nostack, nomem));
        }
    }
}
