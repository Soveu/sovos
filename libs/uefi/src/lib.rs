#![no_std]

#![feature(abi_efiapi)]
#![feature(maybe_uninit_slice)]

use core::mem::MaybeUninit;

mod boot_services;
mod guid;
mod header;
pub mod memory;
mod runtime_services;
mod status;
mod system_table;
pub mod protocols;

pub use boot_services::*;
pub use guid::*;
pub use header::*;
pub use runtime_services::*;
pub use status::*;
pub use system_table::*;

/// Version of EFI spec that this crate is based on.
pub const SPECIFICATION_VERSION: Revision = Revision::new(2, 70);

/// A type that can be used to check whether `efi_main` has good signature
pub type EfiImageEntryPointFunc = extern "efiapi" fn(ImageHandle, *const SystemTable) -> RawStatus;

/// A handle given by UEFI in `efi_main`
#[repr(transparent)]
pub struct ImageHandle(Handle);

#[repr(transparent)]
pub struct Handle(usize);

#[derive(Debug)]
#[repr(C)]
pub struct Config {
    pub guid: Guid,
    pub table: usize,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Revision(u32);

impl Revision {
    pub const fn new(upper: u16, lower: u16) -> Self {
        let lower = lower as u32;
        let upper = upper as u32;
        Self(upper << 16 | lower)
    }
    pub const fn as_tuple(self) -> (u16, u16) {
        let lower = (self.0 >> 0) as u16;
        let upper = (self.0 >> 16) as u16;
        (upper, lower)
    }
}

impl core::fmt::Display for Revision {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::fmt::Debug::fmt(&self.as_tuple(), f)
    }
}
