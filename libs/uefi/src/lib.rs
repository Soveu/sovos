#![no_std]

#![feature(abi_efiapi)]

use core::mem::MaybeUninit;

mod status;
mod header;
mod guid;
mod system_table;
mod boot_services;
mod runtime_services;
pub mod memory;

pub use status::*;
pub use header::*;
pub use guid::*;
pub use system_table::*;
pub use boot_services::*;
pub use runtime_services::*;
//pub use memory::*;

pub const SPECIFICATION_VERSION: u32 = (2 << 16) | 80;

pub type EfiImageEntryPointFunc = extern "efiapi" fn(ImageHandle, *const SystemTable) -> RawStatus;

#[repr(transparent)]
pub struct ImageHandle(Handle);

#[repr(transparent)]
pub struct Handle(usize);

#[repr(C)]
pub struct Config {
    guid: Guid,
    table: usize,
}

