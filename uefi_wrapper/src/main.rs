#![no_std]
#![no_main]

#![allow(unused_parens)]
#![feature(asm_const)]
#![feature(exposed_provenance)]
#![feature(strict_provenance)]
#![feature(panic_info_message)]
#![feature(naked_functions)]

use uefi;
use fb;
use bootinfo::*;
use core::sync::atomic::{Ordering, AtomicPtr};
use core::fmt::Write;
use core::ptr::NonNull;
use arrayvec;

const PRESENT: u64 = (1 << 0);
const WRITABLE: u64 = (1 << 1);
const NONCACHABLE: u64 = (1 << 4);
const NX: u64 = (1 << 63);
const VIRT_OFFSET: u64 = 0xFFFF_8000_0000_0000;
const PTR_MASK: usize  = 0x0000_FFFF_FFFF_F000;
const BOOTINFO_SIZE_PAGES: u64 = (core::mem::size_of::<Bootinfo>() / 4096) as u64;
static STUFF_PTR: AtomicPtr<Bootinfo> = AtomicPtr::new(core::ptr::null_mut());

macro_rules! brint {
    ($($arg:tt)*) => {{
        let _ = write!($($arg)*);
    }}
}

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    let ptr = STUFF_PTR.load(Ordering::SeqCst);
    let fb = unsafe { &mut *core::ptr::addr_of_mut!((*ptr).fb) };

    if let Some(loc) = info.location() {
        let _ = write!(fb, "Panic at {}:{}\n", loc.file(), loc.line());
    } else {
        let _ = fb.write_str("Panic at unknown location\n");
    }

    // let _ = write!(fb, "Message: '{}'", info.message());

    loop {
        cpu::halt();
    }
}

fn setup_framebuffer(boot_services: &uefi::BootServices, bootinfo: &mut Bootinfo) -> uefi::RawStatus {
    let gop = boot_services.locate_protocol(uefi::Guid::EFI_GRAPHICS_OUTPUT_PROTOCOL);
    let gop = match gop {
        Ok(Some(x)) => unsafe { x.cast::<uefi::protocols::gop::GraphicsOutput>().as_mut() },
        _ => return uefi::RawStatus::from_error(uefi::Error::CompromisedData),
    };

    let gop_mode = unsafe { &*gop.mode };
    let gop_info = gop_mode.info.unwrap();
    
    let out = &mut bootinfo.fb;
    *out = fb::Framebuffer {
        base: gop_mode.framebuffer_base as *mut u8,
        memsize: gop_mode.framebuffer_size,
        scanline_width: gop_info.pixels_per_scanline as usize,
        max_x: (gop_info.horizontal_res as usize / fb::FONT_X) as u16,
        max_y: (gop_info.vertical_res as usize / fb::FONT_Y) as u16,
        cursor_x: 0,
        cursor_y: 0,
        mode: fb::Mode::Scroll,
    };
    out.cursor_y = out.max_y - 1;

    brint!(out, "Current mode = {}\n", gop_mode.mode);
    let (mode_index, new_mode) = (0..gop_mode.max_mode)
        .map(|i| (i, gop.query_mode(i).unwrap()))
        .max_by_key(|(_, m)| m.horizontal_res * m.vertical_res)
        .unwrap();

    if gop_mode.mode == mode_index {
        brint!(out, "Current graphics mode is optimal\n");
        return uefi::RawStatus::ok();
    }

    let (x, y) = (new_mode.horizontal_res, new_mode.vertical_res);
    brint!(out, "Switching mode to {}x{}\n", new_mode.horizontal_res, new_mode.vertical_res);
    gop.set_mode(mode_index).unwrap();
    let gop_mode = unsafe { &*gop.mode };
    let gop_info = gop_mode.info.unwrap();

    *out = fb::Framebuffer {
        base: gop_mode.framebuffer_base as *mut u8,
        memsize: gop_mode.framebuffer_size,
        scanline_width: gop_info.pixels_per_scanline as usize,
        max_x: (gop_info.horizontal_res as usize / fb::FONT_X) as u16,
        max_y: (gop_info.vertical_res as usize / fb::FONT_Y) as u16,
        cursor_x: 0,
        cursor_y: 0,
        mode: fb::Mode::Scroll,
    };
    out.cursor_y = out.max_y - 1;

    brint!(out, "Finished switching to {}x{}\n", x, y);
    // for i in (0..gop_mode.max_mode) {
    //     brint!(out, "{}: {:?}\n", i, gop.query_mode(i).unwrap());
    // }

    return uefi::RawStatus::ok();
}

fn allocate_pages(boot_services: &uefi::BootServices, pages: u32) -> (*mut u8, uefi::RawStatus) {
    const ALLOCATE_ANY_PAGES: usize = 0;
    let mut memory: *mut u8 = core::ptr::null_mut();
    let result = (boot_services.allocate_pages)(
        ALLOCATE_ANY_PAGES,
        uefi::memory::Type::LoaderData,
        pages,
        &mut memory,
    );
    return (memory, result);
}

fn base_setup(boot_services: &uefi::BootServices) -> Result<&'static mut Bootinfo, uefi::RawStatus> {
    // First, we need to allocate some memory for global state (framebuffer, memory information..)
    let (bootinfo_ptr, result) = allocate_pages(boot_services, BOOTINFO_SIZE_PAGES as u32);

    if !result.is_ok() {
        // TODO: print on uefi console?
        return Err(result);
    }

    let bootinfo_ptr = bootinfo_ptr.cast::<Bootinfo>();
    unsafe { bootinfo_ptr.write_bytes(0u8, 1) };
    STUFF_PTR.store(bootinfo_ptr, Ordering::SeqCst);
    let bootinfo = unsafe { &mut *bootinfo_ptr };

    let result = setup_framebuffer(boot_services, bootinfo);
    if !result.is_ok() {
        return Err(result);
    }

    return Ok(bootinfo);
}

#[no_mangle]
extern "efiapi" fn efi_main(handle: uefi::ImageHandle, st: *mut uefi::SystemTable) -> uefi::RawStatus {
    if st.is_null() {
        return uefi::RawStatus::from_error(uefi::Error::HttpError);
    }
    let boot_services = match unsafe { (*st).boot_services } {
        Some(p) => unsafe { p.as_ref() },
        None => return uefi::RawStatus::from_error(uefi::Error::IpAddressConflict),
    };

    let bootinfo = match base_setup(boot_services) {
        Ok(b) => b,
        Err(status) => return status,
    };

    let (memkey, memmap) = boot_services.get_memory_map(&mut bootinfo.buf).unwrap();
    const TYPICAL_MEMORY: uefi::memory::Attributes = uefi::memory::Attributes::new()
        .set_noncacheable()
        .set_write_combine()
        .set_write_through()
        .set_write_back();

    for map in memmap {
        use uefi::memory::Type;
        let mtyp = Type::from_int(map.typ);

        // brint!(bootinfo.fb, "{:?}\n", map);
        bootinfo.uefi_meminfo.push(*map);

        if map.phys_start == 0 {
            continue; // We don't quite want to allocate null pointer later, but we should do something with this
        }

        if mtyp == Some(Type::Conventional) {
            assert_eq!(map.attributes, TYPICAL_MEMORY);
            bootinfo.free_memory.push(FreeMemory{ phys_start: map.phys_start, pages: map.pages });
        }
    }

    let ok = unsafe { boot_services.exit_boot_services(handle, memkey) };
    assert_eq!(ok, Ok(()));
    brint!(bootinfo.fb, "Exit boot services\n");

    bootinfo.uefi_systable = unsafe { Some(&*st) };
    post_boot_services(bootinfo);
}

type FreeMemoryVec = arrayvec::ArrayVec<bootinfo::FreeMemory, 32>;

// TODO: zero out stuff
fn post_allocate_page(free_memory: &mut FreeMemoryVec, pages: u64) -> NonNull<u8> {
    for i in 0..free_memory.len() {
        let mem = &mut free_memory[i];
        if mem.pages > pages {
            let result = core::ptr::with_exposed_provenance_mut(mem.phys_start as usize);
            mem.phys_start += (pages * 4096);
            mem.pages -= pages;
            return NonNull::new(result).unwrap();
        } else if mem.pages == pages {
            let result = core::ptr::with_exposed_provenance_mut(free_memory.remove(i).phys_start as usize);
            return NonNull::new(result).unwrap();
        }
    }

    panic!("Out of memory, requested {} pages", pages);
}

fn setup_dummy_instruction_page(bootinfo: &mut Bootinfo) -> NonNull<u8> {
    const INFINITE_LOOP: u16 = 0xFEEB;
    const UD2: u16 = 0x0B0F;
    let mut instr = post_allocate_page(&mut bootinfo.free_memory, 1).cast::<[u16; 2048]>();
    let instr_ref = unsafe { instr.as_mut() };
    instr_ref.fill(UD2);
    instr_ref[0] = INFINITE_LOOP;
    return instr.cast::<u8>();
}

fn _map_memory_page(
    free_memory: &mut FreeMemoryVec,
    mut paging: NonNull<[*mut u8; 512]>,
    phys_addr: u64,
    flags: u64,
    level: u8,
    offset: u64,
) {
    let virt = phys_addr + offset;
    // let virt = phys_addr;
    let idx = (virt >> (12 + level * 9)) & 0x1FF;
    let idx = idx as usize;

    let paging = unsafe { paging.as_mut() };

    if let Some(lower_level) = level.checked_sub(1) {
        if paging[idx].is_null() {
            let mut p = post_allocate_page(free_memory, 1).cast::<[*mut u8; 512]>();
            unsafe { p.as_mut().fill(core::ptr::null_mut()); }
            paging[idx] = p.as_ptr().map_addr(|x| x | 0b11).cast(); // present | readwrite
        }

        let lower = NonNull::new(paging[idx].map_addr(|p| p & PTR_MASK)).unwrap();
        return _map_memory_page(free_memory, lower.cast(), phys_addr, flags, lower_level, offset);
    }

    if !paging[idx].is_null() { return; }
    let entry = phys_addr | flags;
    paging[idx] = core::ptr::null_mut::<u8>().with_addr(entry as usize);
}

fn map_memory_page(
    free_memory: &mut FreeMemoryVec,
    paging: NonNull<[*mut u8; 512]>,
    phys_addr: u64,
    flags: u64,
) {
    assert!(phys_addr & 0xFFF == 0);
    _map_memory_page(free_memory, paging, phys_addr, flags, 3, VIRT_OFFSET);
    // _map_memory_page(free_memory, paging, phys_addr, flags, 3, 0);
}

fn uefi_type_to_flags(typ: uefi::memory::Type) -> u64 {
    use uefi::memory::Type;
    return match typ {
        Type::Reserved
        | Type::Unusable => 0,
        Type::Mmio
        | Type::MmioPortSpace => PRESENT | WRITABLE | NX | NONCACHABLE,
        Type::LoaderData
        | Type::BootServicesCode
        | Type::BootServicesData
        | Type::RuntimeServicesData
        | Type::Persistent
        | Type::Conventional => PRESENT | WRITABLE | NX,
        Type::LoaderCode
        | Type::RuntimeServicesCode => PRESENT,
        Type::AcpiReclaim
        | Type::AcpiNVS
        | Type::PalCode => PRESENT | NX,
    };
}

fn map_whole_memory(bootinfo: &mut Bootinfo, paging: NonNull<[*mut u8; 512]>) {
    use uefi::memory::Type;
    for mem in bootinfo.uefi_meminfo.iter() {
        let flags = uefi_type_to_flags(Type::from_int(mem.typ).unwrap());
        if flags == 0 {
            continue;
        }
        for offset in 0..mem.pages {
            let phys = mem.phys_start + offset * 4096;
            assert!(phys & 0xFFF == 0);
            map_memory_page(&mut bootinfo.free_memory, paging, phys, flags);
        }
    }
}

fn ref_to_addr<T: 'static>(r: *const T) -> u64 {
    r.addr() as u64
}

fn post_boot_services(bootinfo: &'static mut Bootinfo) -> ! {
    let mut paging = post_allocate_page(&mut bootinfo.free_memory, 1).cast::<[*mut u8; 512]>();
    unsafe { paging.as_mut().fill(core::ptr::null_mut()); }

    let instr = setup_dummy_instruction_page(bootinfo);
    let instr_addr = instr.addr().get() as u64;
    map_memory_page(&mut bootinfo.free_memory, paging, instr_addr, PRESENT);

    let bootinfo_addr = ref_to_addr(bootinfo);
    for offset in 0..BOOTINFO_SIZE_PAGES {
        let phys = bootinfo_addr + offset * 4096;
        map_memory_page(&mut bootinfo.free_memory, paging, phys, PRESENT | WRITABLE | NX);
    }

    let fb_addr = ref_to_addr(bootinfo.fb.base);
    let fb_memsize = bootinfo.fb.memsize as u64;
    assert!(fb_memsize % 4096 == 0);
    brint!(bootinfo.fb, "fb_addr={:X}\n", fb_addr);
    let fb_pagesize = fb_memsize / 4096;
    for offset in 0..fb_pagesize {
        let phys = fb_addr + offset * 4096;
        map_memory_page(&mut bootinfo.free_memory, paging, phys, PRESENT | WRITABLE | NX);
    }

    brint!(bootinfo.fb, "Mapping memory\n");
    map_whole_memory(bootinfo, paging);
    brint!(bootinfo.fb, "Setting up IDT and GDT\n");
    setup_gdt(bootinfo);
    setup_idt(bootinfo, instr_addr);

    let cr3 = cpu::Cr3(paging.addr().get() as u64);
    let cpu::interrupt::TableRegister { limit, base } = cpu::interrupt::TableRegister::read();
    brint!(bootinfo.fb, "IDTR limit={:x} base={:p}\n", limit, base);
    let new_stack_ptr = &bootinfo as *const _ as usize as u64;
    let new_stack_ptr = new_stack_ptr + VIRT_OFFSET + 4096;
    unsafe {
        core::arch::asm!(
            "mov rsp, {new_stack}",
            "mov cr3, {cr3}",
            "ud2",
            new_stack = in(reg) new_stack_ptr,
            cr3 = in(reg) cr3.0,
            options(nostack, noreturn),
        );
    }
}

fn setup_gdt(bootinfo: &mut Bootinfo) {
    bootinfo.gdt = cpu::segmentation::GlobalDescriptorTable::new();
    unsafe { cpu::segmentation::Gdtr::new(&bootinfo.gdt).apply() };

    let limit = (core::mem::size_of::<cpu::segmentation::GlobalDescriptorTable>() - 1) as u16;
    let base: *const cpu::segmentation::GlobalDescriptorTable = &bootinfo.gdt;
    let base = base.map_addr(|p| p + VIRT_OFFSET as usize);
    let gdtr = cpu::segmentation::Gdtr { limit, base };
    unsafe {
        core::arch::asm!("lgdt [{}]", in(reg) &gdtr, options(nostack, readonly));
    }

    // let cpu::segmentation::Gdtr { limit, base } = cpu::segmentation::Gdtr::read();
    // brint!(bootinfo.fb, "GDTR limit={:x} base={:p}", limit, base);
}

fn setup_idt(bootinfo: &mut Bootinfo, entry_after_jump: u64) {
    bootinfo.idt.fill(cpu::interrupt::Entry::new());
    // let addr = entry_after_jump + VIRT_OFFSET;
    let addr = entry_after_jump + VIRT_OFFSET;
    let flags = cpu::interrupt::Flags::new_interrupt().set_present();
    bootinfo.idt.fill(cpu::interrupt::Entry::with_handler_and_flags(addr, flags));

    // unsafe { cpu::interrupt::TableRegister::new(&bootinfo.idt).apply(); }

    let bootinfo_asdf: *const cpu::interrupt::Table = &bootinfo.idt;
    let bootinfo_asdf = bootinfo_asdf.map_addr(|p| (p + VIRT_OFFSET as usize));
    let idtr = cpu::interrupt::TableRegister { limit: 16 * 256 - 1, base: bootinfo_asdf };
    brint!(bootinfo.fb, "Applying IDTR with limit={:x} base={:p}\n", 16 * 256 - 1, bootinfo_asdf);
    unsafe { idtr.apply(); }

    // let cpu::interrupt::TableRegister { limit, base } = cpu::interrupt::TableRegister::read();
    // brint!(bootinfo.fb, "IDTR limit={:x} base={:p}\n", limit, base);
}
