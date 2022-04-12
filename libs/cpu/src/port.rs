use crate::instructions::{outb, inb, rep_outsb, rep_insb};

pub trait Write {}
pub trait Read {}

#[derive(Clone, Copy)]
pub struct Readonly;
#[derive(Clone, Copy)]
pub struct Writeonly;
#[derive(Clone, Copy)]
pub struct ReadWrite;

impl Read for Readonly {}
impl Write for Writeonly {}
impl Read for ReadWrite {}
impl Write for ReadWrite {}

pub struct Port<P: Copy = ReadWrite> {
    port: u16,
    _permission: P,
}

impl<P: Copy> Copy for Port<P> {}
impl<P: Copy> Clone for Port<P> {
    fn clone(&self) -> Self {
        *self
    }
}

impl Port<Readonly> {
    pub const unsafe fn new_readonly(port: u16) -> Self {
        Self {
            port,
            _permission: Readonly,
        }
    }
}

impl Port<Writeonly> {
    pub const unsafe fn new_writeonly(port: u16) -> Self {
        Self {
            port,
            _permission: Writeonly,
        }
    }
}

impl Port<ReadWrite> {
    pub const unsafe fn new(port: u16) -> Self {
        Self {
            port,
            _permission: ReadWrite,
        }
    }
}

impl<P: Copy + Write> Port<P> {
    pub unsafe fn write(self, x: u8) {
        outb(self.port, x)
    }
    pub unsafe fn write_slice(self, s: &[u8]) {
        rep_outsb(self.port, s.as_ptr(), s.len());
    }
}

impl<P: Copy + Read> Port<P> {
    pub unsafe fn read(self) -> u8 {
        inb(self.port)
    }
    pub unsafe fn read_slice(self, s: &mut [u8]) {
        rep_insb(self.port, s.as_mut_ptr(), s.len());
    }
}
