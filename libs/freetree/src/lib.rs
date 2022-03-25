//#![no_std]
#![allow(unused_parens)]

#![feature(is_sorted)]
#![feature(slice_ptr_len)]
#![feature(slice_ptr_get)]

mod unique;
pub use unique::Unique;
pub mod poc;
