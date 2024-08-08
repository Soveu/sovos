#![no_std]
#![no_main]

#![allow(unused_parens)]
#![feature(asm_const)]
#![feature(exposed_provenance)]
#![feature(strict_provenance)]
#![feature(naked_functions)]

use uefi;
use fb;
use bootinfo::*;
use core::sync::atomic::{Ordering, AtomicPtr};
use core::fmt::Write;
use core::ptr::NonNull;
use arrayvec;

#[repr(align(16))]
struct AlignedTo16<T: ?Sized>(T);
static KERNEL: &AlignedTo16<[u8]> = &AlignedTo16(*include_bytes!(env!("SOVOS_KERNEL_PATH")));

const UPPER_HALF:     u64 = 0xFFFF_8000_0000_0000;
const QUARTER:        u64 = 0x0000_4000_0000_0000;
const THREE_QUARTERS: u64 = UPPER_HALF + QUARTER;
const GIGAPAGE_SIZE:  u64 = 1 << 20;
const MEGAPAGE_SIZE:  u64 = 2 << 10;

const PRESENT: u64 = (1 << 0);
const WRITABLE: u64 = (1 << 1);
const NONCACHABLE: u64 = (1 << 4);
const NX: u64 = (1 << 63);
const VIRT_OFFSET: u64 = THREE_QUARTERS;
const PTR_MASK: usize  = 0x0000_FFFF_FFFF_F000;
const BOOTINFO_SIZE_PAGES: u64 = (core::mem::size_of::<Bootinfo>() / 4096) as u64;

static STUFF_PTR: AtomicPtr<Bootinfo> = AtomicPtr::new(core::ptr::null_mut());

macro_rules! brint {
    ($($arg:tt)*) => {{
        let _ = write!($($arg)*);
    }}
}

struct Size(u64);
impl core::fmt::Debug for Size {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        const KILO: u64 = 1 << 10;
        const MEGA: u64 = 1 << 20;
        const GIGA: u64 = 1 << 30;
        const TERA: u64 = 1 << 40;

        if self.0 > TERA * 8 {
            return write!(f, "{}TiB", self.0 >> 40);
        }
        if self.0 > GIGA * 8 {
            return write!(f, "{}GiB", self.0 >> 30);
        }
        if self.0 > MEGA * 8 {
            return write!(f, "{}MiB", self.0 >> 20);
        }
        if self.0 > KILO * 8 {
            return write!(f, "{}KiB", self.0 >> 10);
        }
        return write!(f, "{}B", self.0);
    }
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

    let _ = write!(fb, "Message: '{}'", info.message());

    loop {
        cpu::halt();
    }
}

fn setup_framebuffer(boot_services: &mut uefi::BootServices, bootinfo: &mut Bootinfo) -> uefi::RawStatus {
    let gop = boot_services.locate_protocol_mut::<uefi::protocols::gop::GraphicsOutput>();
    let Ok(gop) = gop else {
        return uefi::RawStatus::from_error(uefi::Error::CompromisedData);
    };

    let gop_mode = gop.mode();
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
        mode: fb::Mode::Overwrite,
    };
    // out.cursor_y = out.max_y - 1;

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

    // This is the only place where we need &mut GraphicsOutput and
    // because of that we're dragging &mut BootServices all the way here.
    gop.set_mode(mode_index).unwrap();

    let gop_mode = gop.mode();
    let gop_info = gop_mode.info.unwrap();

    *out = fb::Framebuffer {
        base: gop_mode.framebuffer_base as *mut u8,
        memsize: gop_mode.framebuffer_size,
        scanline_width: gop_info.pixels_per_scanline as usize,
        max_x: (gop_info.horizontal_res as usize / fb::FONT_X) as u16,
        max_y: (gop_info.vertical_res as usize / fb::FONT_Y) as u16,
        cursor_x: 0,
        cursor_y: 0,
        mode: fb::Mode::Overwrite,
    };
    // out.cursor_y = out.max_y - 1;

    brint!(out, "Finished switching to {}x{}\n", x, y);
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

fn base_setup(boot_services: &mut uefi::BootServices) -> Result<&'static mut Bootinfo, uefi::RawStatus> {
    // First, we need to allocate some memory for global state (framebuffer, memory information..)
    let (bootinfo_ptr, result) = allocate_pages(boot_services, BOOTINFO_SIZE_PAGES as u32);

    if !result.is_ok() {
        // TODO: print on uefi console?
        return Err(result);
    }

    let bootinfo_ptr = bootinfo_ptr.cast::<Bootinfo>();
    // SAFETY: the structure can be initialized with zeros
    unsafe { bootinfo_ptr.write_bytes(0u8, 1) };
    STUFF_PTR.store(bootinfo_ptr, Ordering::SeqCst);
    // SAFETY: pointer is valid and structure is initialized
    let bootinfo = unsafe { &mut *bootinfo_ptr };

    let result = setup_framebuffer(boot_services, bootinfo);
    if !result.is_ok() {
        return Err(result);
    }

    return Ok(bootinfo);
}

#[no_mangle]
extern "efiapi" fn efi_main(handle: uefi::ImageHandle, st: Option<&'static mut uefi::SystemTable>) -> uefi::RawStatus {
    cpu::disable_interrupts();

    let Some(st) = st else {
        return uefi::RawStatus::from_error(uefi::Error::HttpError);
    };

    let boot_services = match st.boot_services() {
        Some(p) => p,
        None => return uefi::RawStatus::from_error(uefi::Error::IpAddressConflict),
    };

    let bootinfo = match base_setup(boot_services) {
        Ok(b) => b,
        Err(status) => return status,
    };

    let (memkey, memmap) = boot_services.get_memory_map(&mut bootinfo.buf).unwrap();
    let ok = st.exit_boot_services(handle, memkey);
    assert_eq!(ok, Ok(()));
    brint!(bootinfo.fb, "Exit boot services\n");

    const TYPICAL_MEMORY: uefi::memory::Attributes = uefi::memory::Attributes::new()
        .set_noncacheable()
        .set_write_combine()
        .set_write_through()
        .set_write_back();

    for map in memmap {
        bootinfo.uefi_meminfo.push(*map);

        if map.phys_start == 0 {
            assert_eq!(bootinfo.free_memory_at_null, None);
            bootinfo.free_memory_at_null = core::num::NonZeroU64::new(map.pages);
            continue;
        }

        use uefi::memory::Type;
        if Type::from_int(map.typ) == Some(Type::Conventional) {
            assert_eq!(map.attributes, TYPICAL_MEMORY);
            bootinfo.free_memory.push(FreeMemory{ phys_start: map.phys_start, pages: map.pages });
        }
    }

    bootinfo.uefi_meminfo.sort_unstable_by_key(|m| m.phys_start);
    let mut last_mem_end = 0;
    for map in bootinfo.uefi_meminfo.iter() {
        if last_mem_end != map.phys_start {
            brint!(bootinfo.fb, "    Gap {:?}\n", Size(map.phys_start - last_mem_end));
        }
        brint!(bootinfo.fb, "{:?}\n", map);
        last_mem_end = map.phys_start + map.pages * 4096;
    }

    bootinfo.uefi_systable = Some(&*st);
    post_boot_services(bootinfo);
}

type FreeMemoryVec = arrayvec::ArrayVec<bootinfo::FreeMemory>;

fn post_allocate_page(free_memory: &mut FreeMemoryVec, pages: u64) -> NonNull<u8> {
    let free = free_memory
        .iter_mut()
        .enumerate()
        .find(move |(_, mem)| mem.pages >= pages);

    let Some((idx, mem)) = free else {
        panic!("Out of memory, requested {} pages", pages);
    };

    let ptr = core::ptr::with_exposed_provenance_mut(mem.phys_start as usize);

    if mem.pages == pages {
        free_memory.remove(idx);
    } else {
        mem.phys_start += (pages * 4096);
        mem.pages -= pages;
    }

    return NonNull::new(ptr).unwrap();
}

fn _map_memory_page(
    free_memory: &mut FreeMemoryVec,
    mut paging: NonNull<[*mut u8; 512]>,
    phys_addr: u64,
    virt: u64,
    flags: u64,
    level: u8,
) {
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
        return _map_memory_page(free_memory, lower.cast(), phys_addr, virt, flags, lower_level);
    }

    assert_eq!(paging[idx], core::ptr::null_mut());
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
    let virt = phys_addr + VIRT_OFFSET;
    _map_memory_page(free_memory, paging, phys_addr, virt, flags, 3);
}

fn uefi_type_to_flags(typ: uefi::memory::Type) -> u64 {
    use uefi::memory::Type;
    return match typ {
        Type::Reserved
        | Type::Unusable => 0,
        Type::Mmio
        | Type::MmioPortSpace => PRESENT | WRITABLE | NX | NONCACHABLE,
        Type::LoaderData
        | Type::LoaderCode
        | Type::BootServicesData
        | Type::BootServicesCode
        | Type::Persistent
        | Type::Conventional => PRESENT | WRITABLE | NX,
        Type::RuntimeServicesCode => PRESENT,
        Type::RuntimeServicesData
        | Type::AcpiReclaim
        | Type::AcpiNVS
        | Type::PalCode => PRESENT | NX,
    };
}

fn map_gigapages(free_memory: &mut FreeMemoryVec, mut paging: NonNull<[*mut u8; 512]>, memsize: u64) {
    let paging = unsafe { paging.as_mut() };

    let mut currently_allocated = 0u64;
    for lvl4 in (256usize..256+128) {
        if paging[lvl4].is_null() {
            let mut p = post_allocate_page(free_memory, 1).cast::<[*mut u8; 512]>();
            unsafe { p.as_mut().fill(core::ptr::null_mut()); }
            paging[lvl4] = p.as_ptr().map_addr(|x| x | 0b11).cast(); // present | readwrite
        }

        let mut pdp = NonNull::new(paging[lvl4]).unwrap().cast::<[*mut u8; 512]>();
        let pdp = unsafe { pdp.as_mut() };

        for lvl3 in (0..512usize) {
            let phys = ((lvl4 - 256) << 39) | (lvl3 << 30);
            let flags = PRESENT | WRITABLE | NX | (1 << 7);
            let entry = (phys as u64) | flags;
            pdp[lvl3] = core::ptr::null_mut::<u8>().with_addr(entry as usize);
            currently_allocated += GIGAPAGE_SIZE;

            if currently_allocated >= memsize {
                return;
            }
        }
    }

    panic!("too much memory?");
}

fn map_whole_memory(bootinfo: &mut Bootinfo, paging: NonNull<[*mut u8; 512]>) {
    let (memsum, memsize) = bootinfo.uefi_meminfo
        .iter()
        .filter(|d| uefi::memory::Type::from_int(d.typ).map(uefi_type_to_flags).unwrap_or(0) != 0)
        .fold((0u64, 0u64), |(sz, max), d| {
            let size_in_bytes = d.pages * 4096;
            (sz + size_in_bytes, max.max(d.phys_start + size_in_bytes))
        });

    brint!(bootinfo.fb, "memsum={}MiB or {}GiB\n", memsum >> 20, memsum >> 30);
    brint!(bootinfo.fb, "memsize={}MiB or {}GiB\n", memsize >> 20, memsize >> 30);
    if memsize > GIGAPAGE_SIZE {
        assert_eq!(memsize % GIGAPAGE_SIZE, 0);
        map_gigapages(&mut bootinfo.free_memory, paging, memsize);
    } else if memsize > MEGAPAGE_SIZE {
        assert_eq!(memsize % MEGAPAGE_SIZE, 0);
        todo!();
    } else {
        panic!("bruh");
    }
}

fn ref_to_addr<T: 'static>(r: *const T) -> u64 {
    r.addr() as u64
}

fn calc_exec_pagesize(pheaders: &[elf::ProgramHeader]) -> u64 {
    const PAGE_ALIGN: u64 = 1 << 12;
    let (start, end) = pheaders
        .iter()
        .filter(|ph| ph.segment_type() == Some(elf::SegmentType::Load))
        .fold((u64::MAX, u64::MIN), |current, ph| {
            assert_eq!(ph.p_align, PAGE_ALIGN);

            let segment_start = ph.p_vaddr & 0xFFF;
            let segment_end = (ph.p_vaddr + ph.p_memsz).next_multiple_of(PAGE_ALIGN);
            let min = core::cmp::min(current.0, segment_start);
            let max = core::cmp::max(current.1, segment_end);
            (min, max)
        });

    assert_eq!(start, 0);
    assert_eq!(end % PAGE_ALIGN, 0);
    return end;
}

fn load_kernel(bootinfo: &mut Bootinfo, paging: NonNull<[*mut u8; 512]>) -> u64 {
    let ker = elf::Elf::<elf::Amd64>::from_bytes(&KERNEL.0).unwrap();
    let ker_ph = ker.program_headers().unwrap();
    let sz = calc_exec_pagesize(ker_ph);

    let instr = post_allocate_page(&mut bootinfo.free_memory, sz / 4096);
    let mut instr = NonNull::slice_from_raw_parts(instr, sz as usize);
    let instr_addr = instr.addr().get() as u64;
    let instr = unsafe { instr.as_mut() };
    instr.fill(0xCC);

    for ph in ker_ph
        .iter()
        .filter(|ph| ph.segment_type() == Some(elf::SegmentType::Load))
    {
        let flags = 0u64
            | ((ph.p_flags.is_readable() as u64) << 0)
            | ((ph.p_flags.is_writable() as u64) << 1)
            | ((!ph.p_flags.is_executable() as u64) << 63);
        let paddr_start = instr_addr + (ph.p_vaddr & !0xFFF);
        let paddr_end = instr_addr + (ph.p_vaddr + ph.p_memsz).next_multiple_of(1 << 12);
        for addr_to_map in (paddr_start..paddr_end).step_by(4096)
        {
            map_memory_page(&mut bootinfo.free_memory, paging, addr_to_map, flags);
        }

        let fileoff = ph.p_offset as usize;
        let memoff = ph.p_vaddr as usize;
        let to_copy = ph.p_filesz as usize;
        let copy_end = memoff + to_copy;
        let to_zero = ph.p_memsz.checked_sub(ph.p_filesz).unwrap() as usize;

        let src = &KERNEL.0[fileoff..][..to_copy];
        let dst = &mut instr[memoff..][..to_copy];
        dst.copy_from_slice(src);
        let to_zero = &mut instr[copy_end..][..to_zero];
        to_zero.fill(0u8);
    }

    let entry = ker.header().e_entry.unwrap().get() + instr_addr;
    brint!(bootinfo.fb, "entry={:?}\n", entry);
    return entry;
}

fn post_boot_services(bootinfo: &'static mut Bootinfo) -> ! {
    let mut paging = post_allocate_page(&mut bootinfo.free_memory, 1).cast::<[*mut u8; 512]>();
    unsafe { paging.as_mut().fill(core::ptr::null_mut()); }

    let k_entry = load_kernel(bootinfo, paging);

    let bootinfo_addr = ref_to_addr(bootinfo);
    brint!(bootinfo.fb, "bootinfo_addr={:x}\n", bootinfo_addr);
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
    setup_idt(bootinfo, k_entry);

    let cr3 = cpu::Cr3(paging.addr().get() as u64);
    let new_stack_ptr = ref_to_addr(&bootinfo.buf) + VIRT_OFFSET + 4096;
    brint!(bootinfo.fb, "new_stack_ptr={:x}\n", new_stack_ptr);
    brint!(bootinfo.fb, "Jump!\n");

    // SAFETY: no
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
}

fn setup_idt(bootinfo: &mut Bootinfo, entry_after_jump: u64) {
    bootinfo.idt.fill(cpu::interrupt::Entry::new());
    let addr = entry_after_jump + VIRT_OFFSET;
    let flags = cpu::interrupt::Flags::new_interrupt().set_present();
    bootinfo.idt[0xE] = cpu::interrupt::Entry::with_handler_and_flags(addr, flags);

    let bootinfo_asdf: *const cpu::interrupt::Table = &bootinfo.idt;
    let bootinfo_asdf = bootinfo_asdf.map_addr(|p| (p + VIRT_OFFSET as usize));
    let idtr = cpu::interrupt::TableRegister { limit: 16 * 256 - 1, base: bootinfo_asdf };
    unsafe { idtr.apply(); }
}
