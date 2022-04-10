#![no_std]
#![no_main]

#![feature(abi_efiapi)]
#![feature(abi_x86_interrupt)]
#![feature(asm_sym)]
#![feature(array_chunks)]
#![feature(const_maybe_uninit_assume_init)]
#![feature(panic_info_message)]
#![feature(slice_ptr_len)]
#![feature(naked_functions)]

use uart_16550::SerialPort;

use elf::{Elf, self};
use cpu::{self, acpi};
use uefi::{self, Verify};
use bootinfo::Bootinfo;
use cereal;

use core::fmt::Write;
//use core::ptr;
use core::mem::MaybeUninit;

static KERNEL: &[u8] = include_bytes!(env!("SOVOS_KERNEL_PATH"));
static mut BOOTINFO: Bootinfo = Bootinfo::new();

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
extern "efiapi" fn efi_main(handle: uefi::ImageHandle, st: *mut uefi::SystemTable) -> uefi::RawStatus {
    cpu::disable_interrupts();

    let st = unsafe { &mut *st };
    let bootinfo = unsafe { &mut BOOTINFO };
    static mut BUF: [MaybeUninit<u64>; 1024] = unsafe { MaybeUninit::uninit().assume_init() };

    assert_eq!(st.verify(), Ok(()));

    let boot_services = unsafe { st.boot_services.unwrap().as_ref() };
    //assert_eq!(boot_services.verify(), Ok(()));

    let out = unsafe { st.con_out.unwrap().as_mut() };
    assert_eq!(out.print_utf8("TEST\n👍\n"), Ok(()));

    for cfg in st.config_slice() {
        brint!(out, "{:?}\n", cfg);

        use uefi::Guid;

        if cfg.guid == Guid::EFI_ACPI_20_TABLE {
            let rsdp: *const acpi::Rsdp = cfg.table as *const _;
            unsafe {
                let rsdp = &*rsdp;
                assert!(rsdp.verify_checksum());

                let xsdt: &acpi::Xsdt = acpi::Xsdt::from_raw(rsdp.xsdt);
                let sdt_iter = xsdt.other_sdts
                    .array_chunks::<8>()
                    .map(|x| usize::from_ne_bytes(*x) as *const acpi::SdtHeader);

                for sdt in sdt_iter {
                    let sig = &(*sdt).signature;
                    let sig = core::str::from_utf8_unchecked(sig);
                    let oid = &(*sdt).oem_id;
                    let oid = core::str::from_utf8_unchecked(oid);
                    brint!(out, "\tsignature: {:?} oem_id: {:?}\n", sig, oid);
                }
            }
        }
    }

    let (_, memmap) = boot_services.get_memory_map(unsafe { &mut BUF }).unwrap();

    let mut addr = 0;
    let mut len = 0;
    for map in memmap {
        use uefi::memory::Type;

        let mtyp = Type::from_int(map.typ);

        if mtyp == Some(Type::BootServicesCode)
            || mtyp == Some(Type::BootServicesData)
            || mtyp == Some(Type::Conventional)
        {
            if map.phys_start == addr + len {
                len += map.pages * 4096;
            } else {
                brint!(out, "\tFree memory addr={:016X} len={}kb\n", addr, len / 1024);
                addr = map.phys_start;
                len = map.pages * 4096;
            }
            continue;
        }

        brint!(out, "\t{:?}\n", map);
    }
    brint!(out, "\tFree memory addr={:016X} len={}kb\n", addr, len / 1024);

    prepare_kernel_elf(out);

    let cr4 = cpu::Cr4::get();
    let cr0 = cpu::Cr0::get();
    brint!(out, "CR4: {:?}\n", cr4);
    brint!(out, "CR0: {:?}\n", cr0);

    unsafe {
        brint!(out, "{:?}\n", cereal::identify_uart(0x3F8));
    }

    use cpu::segmentation::GDTR;
    let gdtr = GDTR::new(&bootinfo.gdt);
    unsafe { gdtr.apply(); }
    brint!(out, "GDT applied\n");

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
    brint!(out, "IDT applied\n");

    let (memkey, _) = boot_services.get_memory_map(unsafe { &mut BUF }).unwrap();
    let ok = unsafe { boot_services.exit_boot_services(handle, memkey) };
    assert_eq!(ok, Ok(()));

    todo!("Actually load the ELF");
}

fn prepare_kernel_elf(out: &mut uefi::SimpleTextOutput) {
    let kernel = KERNEL;
    brint!(out, "\nkernel ELF is placed at {:p}, size={}\n", kernel, core::mem::size_of_val(kernel));

    let kernelelf: Elf<elf::Amd64> = Elf::from_bytes(kernel).unwrap();
    let pheaders = kernelelf.program_headers().unwrap();
    let sheaders = kernelelf.section_headers().unwrap();

    brint!(out, "{:?} {:?} {:?} {:?} {:?} 0x{:X}\n",
        kernelelf.header().machine(),
        kernelelf.header().typ(),
        kernelelf.header().e_ident.osabi(),
        kernelelf.header().e_ident.class(),
        kernelelf.header().e_ident.data(),
        kernelelf.header().e_entry.unwrap(),
    );
    brint!(out, "Program headers: {:#?}\n", pheaders);
    brint!(out, "Section headers: {:#?}\n", sheaders);
}
