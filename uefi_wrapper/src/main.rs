#![no_std]
#![no_main]

#![feature(abi_efiapi)]
#![feature(asm)]
#![feature(panic_info_message)]
#![feature(slice_ptr_len)]
#![feature(naked_functions)]

use uart_16550::SerialPort;

use elf::{Elf, self};
use cpu;
use bootinfo::Bootinfo;
use uefi::{self, Verify};

use core::fmt::Write;
//use core::ptr;
use core::mem::MaybeUninit;

#[repr(align(2097152))]
struct PageAligned<T: ?Sized>(T);

static KERNEL: &PageAligned<[u8]> = &PageAligned(*include_bytes!(env!("SOVOS_KERNEL_PATH")));
static mut BOOTINFO: Bootinfo = Bootinfo::new();
const KERNEL_VIRT_ADDR: u64 = 0xffff_ffff_c000_0000;

//static mut BUF: [MaybeUninit<uefi::MemoryDescriptor>; 192] = [MaybeUninit::uninit(); 192];

macro_rules! brint {
    ($($arg:tt)*) => {{
        let _ = write!($($arg)*);
    }}
}

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    let mut out = unsafe { SerialPort::new(0x3F8) };

    brint!(out, "\n\n!!! PANIK !!!\n");
    if let Some(location) = info.location() {
        brint!(out, "file: {:?}, line: {}\n", location.file(), location.line());
    }
    if let Some(msg) = info.message() {
        let _ = out.write_fmt(*msg);
        let _ = out.write_char('\n');
    }

    loop {
        cpu::halt();
    }
}

#[no_mangle]
extern "efiapi" fn efi_main(_handle: uefi::Handle, st: *mut uefi::SystemTable) -> uefi::RawStatus {
    cpu::disable_interrupts();

    let st = unsafe { &mut *st };
    let bootinfo = unsafe { &mut BOOTINFO };
    let mut out = unsafe { SerialPort::new(0x3F8) };
    out.init();

    assert_eq!(st.verify(), Ok(()));
    brint!(out, "Vendor: {:?}\n", st.vendor());

    let boot_services = unsafe { &*st.boot_services };
    //assert_eq!(boot_services.verify(), Ok(()));

    let mut buf: [MaybeUninit<u64>; 1024] = unsafe { MaybeUninit::uninit().assume_init() };
    let (_memkey, memmap) = boot_services.get_memory_map(&mut buf).unwrap();

    for map in memmap {
        brint!(out, "\t{:?}\n", map);
    }

    let cr4 = cpu::Cr4::get();
    let cr0 = cpu::Cr0::get();
    brint!(out, "CR4: {:?}\n", cr4);
    brint!(out, "CR0: {:?}\n", cr0);

    /*
    brint!(out, "\nptr = {:p}, len = {}\n", loader_code as *const u8, loader_code.len());
    brint!(out, "entries = {}\n", bootinfo.uefi_meminfo.len());
    */

    let kernel = &KERNEL.0;
    brint!(out, "kernel: {:p}, size={}\n", kernel, core::mem::size_of_val(kernel));
    //brint!(out, "bootinfo: {:p}, size={}\n", bootptr, core::mem::size_of::<Bootinfo>());

    let kernelelf: Elf<elf::Amd64> = Elf::from_bytes(&KERNEL.0).unwrap();
    let pheaders = kernelelf.program_headers().unwrap();

    brint!(out, "\n{:?} {:?}\n", kernelelf.header().machine(), kernelelf.header().e_ident.os_abi());
    assert_eq!(pheaders[0].p_vaddr, KERNEL_VIRT_ADDR);

    let (text, pheaders) = pheaders.split_first().unwrap();
    assert!(text.is_executable());
    assert!(!text.is_writable());
    assert_eq!(text.p_align, 1 << 21);
    assert_eq!(text.p_vaddr, KERNEL_VIRT_ADDR);

    let (rodata, pheaders) = pheaders.split_first().unwrap();
    assert!(!rodata.is_executable());
    assert!(!rodata.is_writable());
    assert_eq!(rodata.p_align, 1 << 21);

    let (data_bss, pheaders) = pheaders.split_first().unwrap();
    assert!(!data_bss.is_executable());
    assert!(data_bss.is_writable());
    assert_eq!(data_bss.p_align, 1 << 21);

    brint!(out, "Remaining headers: {:#?}\n", pheaders);

    use cpu::segmentation::GDTR;
    let gdtr = GDTR::new(&bootinfo.gdt);
    unsafe { apply_gdtr(&gdtr); }

    loop { cpu::halt() };
}

#[naked]
pub unsafe extern "win64" fn apply_gdtr(_gdtr: &cpu::segmentation::GDTR) {
    asm!("
        lgdt [rcx]

        mov ax, 16
        mov ds, ax
        mov ss, ax

        mov ax, 0
        mov es, ax
        mov fs, ax
        mov gs, ax

        pop rax
        push qword ptr 8
        push rax

        retfq
        ",
        options(noreturn),
    )
}

