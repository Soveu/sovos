#![feature(abi_x86_interrupt)]
#![feature(naked_functions)]
#![feature(asm)]

use cpu::interrupt;

#[test]
fn interrupt_handler_macro_check() {
    extern "sysv64" fn example(_: &mut interrupt::Stack) {}
    //extern "win64" fn example(_: &mut interrupt::Stack) {}
    let _ih: extern "x86-interrupt" fn() = interrupt::make_handler!(example);
}

#[test]
fn stack_size_check() {
    assert!(core::mem::size_of::<interrupt::Stack>() % 16 == 0);
}
