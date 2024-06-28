use core::arch::asm;

#[inline(always)]
pub unsafe fn ud2() -> ! {
    asm!("ud2", options(nomem, nostack, noreturn));
}

#[inline(always)]
pub fn nop() {
    unsafe {
        asm!("nop", options(nomem, nostack, preserves_flags));
    }
}

#[inline(always)]
pub fn get_sp() -> usize {
    let out: usize;

    unsafe {
        asm!(
            "mov {}, rsp",
            out(reg) out,
            options(nomem, nostack, preserves_flags)
        );
    }

    return out;
}

#[inline(always)]
pub fn get_ip() -> usize {
    let out: usize;
    unsafe {
        asm!(
            "mov {}, rip",
            out(reg) out,
            options(nomem, nostack, preserves_flags)
        );
    }
    return out;
}

/// SAFETY: Intel says this instruction is UB when input is equal to 0
#[inline(always)]
pub unsafe fn bsf(mut x: u64) -> u8 {
    unsafe {
        asm!(
            "bsf {0}, {0}",
            "jnz 1f",
            "mov al, 64",
            "1:",
            inout(reg) x,
        );
    }

    return x as u8;
}

/// SAFETY: port must be valid
#[inline(always)]
pub unsafe fn inb(port: u16) -> u8 {
    let result: u8;

    asm!(
        "in al, dx",
        out("al") result,
        in("dx") port,
        options(nostack, nomem),
    );

    return result;
}

/// SAFETY: port and value must be valid
#[inline(always)]
pub unsafe fn outb(port: u16, value: u8) {
    asm!(
        "out dx, al",
        in("dx") port,
        in("al") value,
        options(nostack, nomem),
    );
}

/// SAFETY: port must be valid for write,
/// pointer and size must be valid for read.
/// Also, be aware, that these instructions might be too fast
/// for the device under that port.
#[inline(always)]
pub unsafe fn rep_outsb(port: u16, ptr: *const u8, sz: usize) {
    asm!(
        "rep outs dx, byte ptr [rsi]",
        in("dx") port,
        in("rsi") ptr,
        in("rcx") sz,
        options(nostack, readonly),
    );
}

/// SAFETY: port must be valid for read,
/// pointer and size must be valid for write.
/// Also, be aware, that these instructions might be too fast
/// for the device under that port.
#[inline(always)]
pub unsafe fn rep_insb(port: u16, ptr: *mut u8, sz: usize) {
    asm!(
        "rep ins byte ptr [rdi], dx",
        in("dx") port,
        in("rdi") ptr,
        in("rcx") sz,
        options(nostack),
    );
}
