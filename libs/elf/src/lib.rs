#![no_std]

mod definitions;
pub use definitions::*;

use bytemuck;
use core::mem;

pub struct Elf<'a, M: ElfMachine> {
    pub data: &'a [u8],
    _phantom: core::marker::PhantomData<M>,
}

pub trait ElfMachine {
    const CLASS: Class;
    const ENDIANESS: Data;
    const OSABI: OsAbi;
    const ABIVERSION: u8;
    const MACHINE: Machine;
}

pub struct Amd64;
impl ElfMachine for Amd64 {
    const CLASS: Class = Class::Bits64;
    const ENDIANESS: Data = Data::Lsb;
    const OSABI: OsAbi = OsAbi::SystemV;
    const ABIVERSION: u8 = 0;
    const MACHINE: Machine = Machine::X64;
}

#[derive(Clone, Copy, Debug)]
pub enum MemoryError {
    WrongAlignment,
    UnexpectedEnd,
    SizeMismatch,
}

#[derive(Clone, Copy, Debug)]
pub enum Error {
    WrongAlignment,
    UnexpectedEnd,
    NotElf,
    NotExec,
    WrongClass,
    WrongEndianess,
    WrongOsAbi,
    WrongMachine,
    UnsupportedVersion,
}

impl<'a, M: ElfMachine> Elf<'a, M> {
    pub fn program_headers(&self) -> Result<&[ProgramHeader], MemoryError> {
        let header = self.header();

        let phoff = match header.e_phoff {
            Some(x) => x.get(),
            None => return Err(MemoryError::UnexpectedEnd),
        };
        if phoff > usize::MAX as u64 {
            return Err(MemoryError::UnexpectedEnd);
        }
        let phoff = phoff as usize;

        if header.e_phentsize as usize != mem::size_of::<ProgramHeader>() {
            return Err(MemoryError::SizeMismatch);
        }
        let len_bytes = header.e_phnum as usize * mem::size_of::<ProgramHeader>();

        let start = phoff;
        let end = start + len_bytes;

        let chunk = match self.data.get(start..end) {
            Some(x) => x,
            None => return Err(MemoryError::UnexpectedEnd),
        };

        return match bytemuck::try_cast_slice(chunk) {
            Ok(x) => Ok(x),
            Err(bytemuck::PodCastError::AlignmentMismatch) => Err(MemoryError::WrongAlignment),
            Err(_) => unreachable!(),
        };
    }

    pub fn header(&self) -> &Header {
        bytemuck::from_bytes(&self.data[..EHSIZE_X64])
    }

    pub fn from_bytes(elf: &'a [u8]) -> Result<Self, Error> {
        let ptr = elf.as_ptr() as usize;
        if ptr % 8 != 0 {
            return Err(Error::WrongAlignment);
        }

        let header_ident = match elf.get(..mem::size_of::<HeaderIdent>()) {
            Some(x) => x,
            None => return Err(Error::UnexpectedEnd),
        };
        let header_ident: &HeaderIdent = bytemuck::from_bytes(header_ident);

        if header_ident.ei_magic != MAGIC {
            return Err(Error::NotElf);
        }
        if header_ident.ei_class != M::CLASS as u8 {
            return Err(Error::WrongClass);
        }
        if header_ident.ei_data != M::ENDIANESS as u8 {
            return Err(Error::WrongEndianess);
        }
        if header_ident.ei_version != EV_CURRENT {
            return Err(Error::UnsupportedVersion);
        }
        if header_ident.ei_osabi != M::OSABI as u8 {
            return Err(Error::WrongOsAbi);
        }
        if header_ident.ei_abiversion != M::ABIVERSION {
            return Err(Error::WrongOsAbi);
        }

        assert_eq!(mem::size_of::<Header>(), EHSIZE_X64);

        let header = match elf.get(..EHSIZE_X64) {
            Some(x) => x,
            None => return Err(Error::UnexpectedEnd),
        };
        let header: &Header = bytemuck::from_bytes(header);

        if header.e_type != Type::Executable as u16 {
            return Err(Error::NotExec);
        }
        if header.e_machine != M::MACHINE as u16 {
            return Err(Error::WrongMachine);
        }
        if header.e_version != EV_CURRENT as u32 {
            return Err(Error::UnsupportedVersion);
        }

        return Ok(Self {
            data: elf,
            _phantom: core::marker::PhantomData,
        });
    }
}
