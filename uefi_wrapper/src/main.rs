#![no_std]
#![no_main]

#![feature(panic_info_message)]

use uefi;
use fb;
use bootinfo::*;
use core::sync::atomic::{Ordering, AtomicPtr};
use core::fmt::Write;

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

    if let Some(msg) = info.message() {
        let _ = fb.write_fmt(*msg);
    }

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

    let (x, y) = (new_mode.horizontal_res, new_mode.vertical_res);
    brint!(out, "Switching mode to {}x{}\n", new_mode.horizontal_res, new_mode.vertical_res);
    gop.set_mode(mode_index).unwrap();
    let gop_mode = unsafe { &*gop.mode };
    let gop_info = gop_mode.info.unwrap();

    *out = fb::Framebuffer {
        base: gop_mode.framebuffer_base as *mut u8,
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
    let pages = core::mem::size_of::<Bootinfo>() / 4096;
    let (bootinfo_ptr, result) = allocate_pages(boot_services, pages as u32);

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
        if mtyp == Some(Type::Conventional) {
            assert_eq!(map.attributes, TYPICAL_MEMORY);
            bootinfo.free_memory.push(FreeMemory{ phys_start: map.phys_start, pages: map.pages });
        } else {
            bootinfo.uefi_meminfo.push(*map);
        }
    }

    let ok = unsafe { boot_services.exit_boot_services(handle, memkey) };
    assert_eq!(ok, Ok(()));
    brint!(bootinfo.fb, "Exit boot services\n");

    bootinfo.uefi_systable = unsafe { Some(&*st) };
    post_boot_services(bootinfo);
}

fn post_boot_services(bootinfo: &'static mut Bootinfo) -> ! {
    todo!()
}
