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

static STR: [u8; 12] = *b"Hello World!";
static mut MUT: [u8; 11] = *b"This is MUT";

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

extern "sysv64" fn kmain() -> ! {
    unsafe {
        asm!("xchg rax, rax", in("rax") &STR, options(nostack, nomem));
        MUT[0] = b'b';
        asm!("xchg rax, rax", in("rax") &MUT, options(nostack, nomem));
    }

    loop {
        unsafe {
            asm!("hlt", options(nostack, nomem));
        }
    }
}
