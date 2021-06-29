#![no_std]
#![no_main]

#![feature(asm)]
#![feature(bench_black_box)]
#![feature(naked_functions)]

#[panic_handler]
fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

static STR: &[u8] = b"ayyyyyyyyyyyy";
static mut MUT: [u8; 1 << 18] = [b'a'; 1 << 18];
static mut ZEROED: [u8; 1 << 18] = [0u8; 1 << 18];

#[no_mangle]
#[naked]
pub unsafe extern "sysv64" fn _start() -> ! {
    asm!("
        test sp, 15
        jnz no_error_code

        pop rax

    no_error_code:
        pop rax // pop the old ip
        pop rax // pop the code segment
        pop rbx // pop flags

        xor rbx, rbx
        push rbx
        push rax
        push qword ptr {}

        iretq",
        sym kmain,
        options(noreturn),
    )
}

extern "sysv64" fn kmain() -> ! {
    unsafe {
        core::hint::black_box(STR);
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
