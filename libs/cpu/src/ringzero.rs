#![cfg(feature = "ringzero")]

use core::arch::asm;
use crate::{impl_bits, paging, PhysAddr, VirtAddr};

/// The processor halt instruction (HLT) halts instruction execution, leaving
/// the processor in the halt state. No registers or machine state are modified
/// as a result of executing the HLT instruction. The processor remains in the
/// halt state until one of the following occurs:
/// * A non-maskable interrupt (NMI).
/// * An enabled, maskable interrupt (INTR).
/// * Processor reset (RESET).
/// * Processor initialization (INIT).
/// * System-management interrupt (SMI).
#[inline(always)]
pub fn halt() {
    unsafe {
        asm!("hlt", options(nostack, nomem));
    }
}

/*
/// The load segment-limit (LSL) instruction uses a segment-selector in the
/// source operand to reference a descriptor in the GDT or LDT.
/// LSL performs a set of preliminary access-right checks and, if successful,
/// loads the segment-descriptor limit field into the destination register.
/// Software can use the limit value in comparisons with pointer offsets to prevent
/// segment limit violations.
#[inline(always)]
pub fn load_segment_limit() {
    unsafe {
        asm!("lsl", options(nostack, nomem));
    }
}
*/

#[inline(always)]
pub fn disable_interrupts() {
    unsafe {
        asm!("cli", options(nostack, nomem));
    }
}

#[inline(always)]
pub fn enable_interrupts() {
    unsafe {
        asm!("sti", options(nostack, nomem));
    }
}

#[inline(always)]
pub fn set_global_interrupt_flag() {
    unsafe {
        asm!("stgi", options(nostack, nomem));
    }
}

#[inline(always)]
pub fn clear_global_interrupt_flag() {
    unsafe {
        asm!("clgi", options(nostack, nomem));
    }
}

#[inline(always)]
pub fn clts() {
    unsafe {
        asm!("clts", options(nostack, nomem));
    }
}

/*
#[inline(always)]
pub fn load_access_rights_byte() {
    unsafe {
        asm!("lar", options(nostack, nomem));
    }
}
*/

/// Safety:  Unlike the WBINVD instruction, no modified cache lines are
/// written to memory. The INVD instruction should only be used in
/// situations where memory coherency is not required.
#[inline(always)]
pub unsafe fn invd() {
    asm!("invd", options(nostack, nomem));
}

/// The writeback and invalidate (WBINVD) and writeback no invalidate (WBNOINVD)
/// instructions are used to write all modified cache lines to memory so that
/// memory contains the most recent copy of data. After the writes are complete,
/// the WBINVD instruction invalidates all cache lines, whereas the WBNOINVD
/// instruction may leave the lines in the cache hierarchy in a non-modified
/// state. These instructions operate on all caches in the memory hierarchy,
/// including caches that are external to the processor. See the instructions'
/// description in Volume 3 for further operational details
#[inline(always)]
pub fn wbinvd() {
    unsafe {
        asm!("wbinvd", options(nostack, nomem));
    }
}

/// See [`wbinvd`]
#[inline(always)]
pub fn wbnoinvd() {
    unsafe {
        asm!("wbnoinvd", options(nostack, nomem));
    }
}

#[inline(always)]
pub fn incssp() {
    unsafe {
        asm!("incssp", options(nostack, nomem));
    }
}

/// The invalidate TLB entry (INVLPG) instruction can be used to invalidate
/// specific entries within the TLB. The source operand is a virtual-memory
/// address that specifies the TLB entry to be invalidated. Invalidating a TLB
/// entry does not remove the associated page-table entry from the data cache.
/// See “Translation-Lookaside Buffer (TLB)” on page 147 for more information.
#[inline(always)]
pub fn invlpg() {
    unsafe {
        asm!("invlpg", options(nostack, nomem));
    }
}

/// The invalidate TLB entry in a Specified ASID instruction (INVLPGA) can be
/// used to invalidate TLB entries associated with the specified ASID. See
/// "Invalidate Page, Alternate ASID" on page 498
#[inline(always)]
pub fn invlpga() {
    unsafe {
        asm!("invlpga", options(nostack, nomem));
    }
}

/// The invalidate TLB with Broadcast instruction (INVLPGB) can be used to
/// invalidate a specified range of TLB entries on the local processor and
/// broadcast the invalidation to remote processors. See "INVLPGB" in Volume 3
#[inline(always)]
pub fn invlpgb() {
    unsafe {
        asm!("invlpgb", options(nostack, nomem));
    }
}

/// The invalidate TLB entries in Specified PCID instruction (INVPCID) can be
/// used to invalidate TLB entries of the specified Processor Context ID. See
/// “INVPCID” in Volume 3.
#[inline(always)]
pub fn invlpcid() {
    unsafe {
        asm!("invlpcid", options(nostack, nomem));
    }
}

#[repr(transparent)]
pub struct Cr4(u64);

impl_bits!(Cr4 = {
    vme = 0,
    pvi = 1,
    time_stamp_disable = 2,
    debug_extension = 3,
    page_size_extensions = 4,
    physical_address_extension = 5,
    machine_check = 6,
    page_global = 7,
    perf_counter = 8,
    os_fxsave_fxrstor = 9,
    os_simd_float_exceptions = 10,
    usermode_instruction_prevention = 11,

    // Only in Intel manual
    intel_vmx = 13,
    intel_smx = 14,

    fsgsbase = 16,
    process_context_id = 17,
    os_xsave = 18,

    /// Doesn't allow kernel to exec usermode instructions
    supervisormode_exec_prot = 20,

    /// Doesn't allow kernel to access usermode memory if some stuff
    /// isnt set up
    supervisormode_access_prot = 21,
    protection_key = 22,

    // For now only in AMD manual
    control_flow_enforcement = 23,
});

impl Cr4 {
    pub fn new(phys_addr: u64, pcid: u16) -> Self {
        debug_assert!(phys_addr & 0xFFF == 0);
        debug_assert!(pcid <= 0xFFF);
        Self(phys_addr | (pcid as u64))
    }

    pub fn get() -> Self {
        let cr4: u64;

        unsafe {
            asm!(
                "mov {:r}, cr4",
                out(reg) cr4,
                options(nomem, nostack, preserves_flags),
            );
        }

        Self(cr4)
    }

    pub unsafe fn set(cr4: Self) {
        asm!(
            "mov cr4, {:r}",
            in(reg) cr4.0,
            options(nomem, nostack)
        );
    }
}

#[repr(transparent)]
pub struct Cr0(u64);

impl_bits!(Cr0 = {
    protection_enable = 0,
    monitor_coprocessor = 1,
    emulation = 2,
    task_switched = 3,
    extension_type = 4,
    numeric_error = 5,

    /// Ring 0-2 normally can write to pages marked as non-writable
    write_protect = 16,

    alignment_check = 18,

    not_write_through = 29,
    cache_disable = 30,
    paging = 31,
});

impl Cr0 {
    pub fn get() -> Self {
        let cr0: u64;

        unsafe {
            asm!(
                "mov {:r}, cr0",
                out(reg) cr0,
                options(nomem, nostack, preserves_flags),
            );
        }

        Self(cr0)
    }

    pub unsafe fn set(cr0: Self) {
        asm!(
            "mov cr0, {:r}",
            in(reg) cr0.0,
            options(nomem, nostack)
        );
    }
}

#[repr(transparent)]
pub struct Cr2(pub VirtAddr);

impl Cr2 {
    pub fn get() -> Self {
        let cr2: u64;

        unsafe {
            asm!(
                "mov {:r}, cr2",
                out(reg) cr2,
                options(nomem, nostack, preserves_flags),
            );
        }

        Self(VirtAddr::new(cr2))
    }
}

#[repr(transparent)]
pub struct Cr3(pub u64);

impl Cr3 {
    pub fn from_addr(addr: PhysAddr<paging::Table<paging::PML4Entry>>) -> Self {
        Self(addr.as_u64())
    }

    pub fn set_disable_cache(self) -> Self {
        Self(self.0 | (1 << 4))
    }

    pub fn set_writethrough(self) -> Self {
        Self(self.0 | (1 << 3))
    }

    pub fn clear_writethrough(self) -> Self {
        Self(self.0 & !(1 << 3))
    }

    pub fn clear_disable_cache(self) -> Self {
        Self(self.0 & !(1 << 4))
    }

    pub fn get() -> Self {
        let cr3: u64;

        unsafe {
            asm!(
                "mov {:r}, cr3",
                out(reg) cr3,
                options(nomem, nostack, preserves_flags),
            );
        }

        Self(cr3)
    }

    pub unsafe fn set(cr3: Self) {
        asm!(
            "mov cr3, {:r}",
            in(reg) cr3.0,
            options(nostack)
        );
    }
}
