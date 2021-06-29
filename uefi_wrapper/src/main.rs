#![no_std]
#![no_main]

#![feature(abi_efiapi)]
#![feature(abi_x86_interrupt)]
#![feature(asm)]
#![feature(const_maybe_uninit_assume_init)]
#![feature(panic_info_message)]
#![feature(slice_ptr_len)]
#![feature(naked_functions)]

use uart_16550::SerialPort;

use elf::{Elf, self};
use cpu::{self, acpi};
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
extern "efiapi" fn efi_main(handle: uefi::ImageHandle, st: *const uefi::SystemTable) -> uefi::RawStatus {
    cpu::disable_interrupts();

    let st = unsafe { &*st };
    let bootinfo = unsafe { &mut BOOTINFO };
    let mut out = unsafe { SerialPort::new(0x3F8) };
    static mut buf: [MaybeUninit<u64>; 1024] = unsafe { MaybeUninit::uninit().assume_init() };
    out.init();

    assert_eq!(st.verify(), Ok(()));

    let boot_services = unsafe { &*st.boot_services.get() };
    //assert_eq!(boot_services.verify(), Ok(()));

    for cfg in st.config_slice() {
        use uefi::Guid;

        if cfg.guid == Guid::EFI_ACPI_20_TABLE {
            let rsdp: *const acpi::Rsdp = cfg.table as *const _;
            unsafe {
                let rsdp = &*rsdp;
                assert!(rsdp.verify_checksum());

                let xsdt: &acpi::Xsdt = acpi::Xsdt::from_raw(rsdp.xsdt);
                for sdt in &xsdt.other_sdts {
                    let sdt: *const acpi::SdtHeader = core::ptr::read_unaligned(sdt);
                    let sig = &(*sdt).signature;
                    let sig = core::str::from_utf8_unchecked(sig);
                    let oid = &(*sdt).oem_id;
                    let oid = core::str::from_utf8_unchecked(oid);
                    brint!(out, "\tsignature: {:?} oem_id: {:?}\n", sig, oid);
                }
            }
        }
        brint!(out, "{:?}\n", cfg);
    }

    let (memkey, memmap) = boot_services.get_memory_map(unsafe { &mut buf }).unwrap();
    let ok = unsafe { boot_services.exit_boot_services(handle, memkey) };
    assert_eq!(ok, Ok(()));

    for map in memmap {
        use uefi::memory::Type;

        let mtyp = Type::from_int(map.typ);

        if mtyp == Some(Type::BootServicesCode) || mtyp == Some(Type::BootServicesData) {
            continue;
        }

        brint!(out, "\t{:?}\n", map);
    }

    let cr4 = cpu::Cr4::get();
    let cr0 = cpu::Cr0::get();
    brint!(out, "CR4: {:?}\n", cr4);
    brint!(out, "CR0: {:?}\n", cr0);

    use cpu::segmentation::GDTR;
    let gdtr = GDTR::new(&bootinfo.gdt);
    unsafe { gdtr.apply(); }

    use cpu::interrupt;
    extern "sysv64" fn _dummy_handler(ii: &mut interrupt::Stack) {
        let mut out = unsafe { SerialPort::new(0x3F8) };
        brint!(out, "\nHANDLER\n\n{:?}\n", ii);
        loop { cpu::halt() };
    }
    let dummy_handler = interrupt::make_handler!(_dummy_handler);
    let idt_flags = interrupt::Flags::new_interrupt()
        .disable_interrupts()
        .set_present();
    let idt_entry = interrupt::Entry::with_handler_and_flags(dummy_handler, idt_flags);
    bootinfo.idt = [idt_entry; 256];
    let idtr = interrupt::TableRegister::new(&bootinfo.idt);
    unsafe { idtr.apply(); }

    prepare_kernel_elf(&mut out);

    loop { cpu::halt() };
}

fn prepare_kernel_elf(out: &mut SerialPort) {
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
}

