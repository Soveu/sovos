#![no_std]
#![no_main]

#![feature(asm)]
#![feature(bench_black_box)]

#[panic_handler]
fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

static STR: &[u8] = b"ayyyyyyyyyyyy";
static mut MUT: [u8; 1 << 18] = [b'a'; 1 << 18];
static mut ZEROED: [u8; 1 << 18] = [0u8; 1 << 18];

#[no_mangle]
pub extern "C" fn _start() -> ! {
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
