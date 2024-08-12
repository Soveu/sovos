#![allow(dead_code)]

pub mod dynamic;

use core::num::NonZeroU64;
use bytemuck::{Pod, Zeroable};

pub const MAGIC: [u8; 4] = *b"\x7FELF";
pub const EV_CURRENT: u8 = 1;
pub const EHSIZE_X86: usize = 52;
pub const EHSIZE_X64: usize = 64;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct HeaderIdent {
    pub ei_magic:      [u8; 4],
    pub ei_class:      u8,
    pub ei_data:       u8,
    pub ei_version:    u8,
    pub ei_osabi:      u8,
    pub ei_abiversion: u8,
    pub ei_pad:        [u8; 7],
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Header {
    pub e_ident:     HeaderIdent,
    pub e_type:      u16,
    pub e_machine:   u16,
    pub e_version:   u32,
    pub e_entry:     Option<NonZeroU64>,
    pub e_phoff:     Option<NonZeroU64>,
    pub e_shoff:     Option<NonZeroU64>,
    pub e_flags:     u32,
    pub e_ehsize:    u16,
    pub e_phentsize: u16,
    pub e_phnum:     u16,
    pub e_shentsize: u16,
    pub e_shnum:     u16,
    pub e_shstrndx:  u16,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ProgramHeader {
    pub p_type:   u32,
    pub p_flags:  ProgramHeaderFlags,
    pub p_offset: u64,
    pub p_vaddr:  u64,
    /* Practical Binary Analysis says it should be zero, but readelf on some
     * binaries shows that it is equal to p_vaddr */

    pub p_paddr: u64,

    pub p_filesz: u64,
    pub p_memsz:  u64,
    pub p_align:  u64,
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct ProgramHeaderFlags(u32);

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SectionHeader {
    pub sh_name: u32,
    pub sh_type: u32,

    pub sh_flags:  u64,
    pub sh_addr:   u64,
    pub sh_offset: u64,
    pub sh_size:   u64,

    pub sh_link:      u32,
    pub sh_info:      u32,
    pub sh_addralign: u64,
    pub sh_entsize:   u64,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum SectionType {
    Null          = 0,
    Progbits,
    Symtab,
    Strtab,
    Rela,
    Hash,
    Dynamic,
    Note,
    Nobits,
    Rel           = 9,

    Dynsym        = 11,

    InitArray     = 14,
    FiniArray,
    PreinitArray,
    Group,
    SymtabShIndex = 18,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum Class {
    Bits32 = 1,
    Bits64 = 2,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum Data {
    Lsb = 1,
    Msb = 2,
}

#[repr(u16)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Type {
    None         = 0,
    Relocatable  = 1,
    Executable   = 2,
    SharedObject = 3,
    Core         = 4,
}

#[repr(u16)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Machine {
    None    = 0,
    PowerPC = 20,
    Power64 = 21,
    Arm     = 40,
    X86     = 3,
    X64     = 62,
    AArch64 = 183,
    AmdGpu  = 224,
    RiscV   = 243,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OsAbi {
    SystemV    = 0,   /* UNIX System V ABI */
    Hpux       = 1,   /* HP-UX */
    NetBSD     = 2,   /* NetBSD. */
    GnuLinux   = 3,   /* Object uses GNU ELF extensions. */
    Solaris    = 6,   /* Sun Solaris. */
    Aix        = 7,   /* IBM AIX. */
    Irix       = 8,   /* SGI Irix. */
    FreeBSD    = 9,   /* FreeBSD. */
    Tru64      = 10,  /* Compaq TRU64 UNIX. */
    Modesto    = 11,  /* Novell Modesto. */
    OpenBSD    = 12,  /* OpenBSD. */
    ArmAEABI   = 64,  /* ARM EABI */
    Arm        = 97,  /* ARM */
    Standalone = 255, /* Standalone (embedded) application */
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SegmentType {
    Null,
    Load,
    Dynamic,
    Interpreter,
    Note,
    SharedLib,
    ProgramHeader,
    ThreadLocalStorage,

    OsSpecificGnuStack = 1685382481,
    OsSpecificGnuRelro = 1685382482,
    OsSpecificGnuProperty= 1685382483,

    OsSpecific(u32),
    CpuSpecific(u32),
}

impl HeaderIdent {
    pub const fn class(&self) -> Option<Class> {
        Class::from_integer(self.ei_class)
    }

    pub const fn data(&self) -> Option<Data> {
        Data::from_integer(self.ei_class)
    }

    pub const fn osabi(&self) -> Option<OsAbi> {
        OsAbi::from_integer(self.ei_class)
    }
}

impl Header {
    pub const fn typ(&self) -> Option<Type> {
        Type::from_integer(self.e_type)
    }

    pub const fn machine(&self) -> Option<Machine> {
        Machine::from_integer(self.e_machine)
    }
}

impl ProgramHeader {
    pub fn segment_type(&self) -> Option<SegmentType> {
        SegmentType::from_integer(self.p_type)
    }
}

impl ProgramHeaderFlags {
    #[allow(clippy::identity_op)]
    pub fn is_executable(&self) -> bool {
        (self.0 >> 0) & 1 == 1
    }

    pub fn is_writable(&self) -> bool {
        (self.0 >> 1) & 1 == 1
    }

    pub fn is_readable(&self) -> bool {
        (self.0 >> 2) & 1 == 1
    }

    pub fn os_specific_flags(&self) -> u8 {
        (self.0 >> 20) as u8
    }

    pub fn cpu_specific_flags(&self) -> u8 {
        (self.0 >> 28) as u8
    }
}

impl SectionHeader {
    pub const fn typ(&self) -> Option<SectionType> {
        SectionType::from_integer(self.sh_type)
    }
}

impl SectionType {
    pub const fn from_integer(x: u32) -> Option<Self> {
        let typ = match x {
            0 => Self::Null,
            1 => Self::Progbits,
            2 => Self::Symtab,
            3 => Self::Strtab,
            4 => Self::Rela,
            5 => Self::Hash,
            6 => Self::Dynamic,
            7 => Self::Note,
            8 => Self::Nobits,
            9 => Self::Rel,

            11 => Self::Dynsym,
            14 => Self::InitArray,
            15 => Self::FiniArray,
            16 => Self::PreinitArray,
            17 => Self::Group,
            18 => Self::SymtabShIndex,

            _ => return None,
        };

        return Some(typ);
    }
}

impl Class {
    pub const fn from_integer(x: u8) -> Option<Self> {
        let r = match x {
            1 => Self::Bits32,
            2 => Self::Bits64,
            _ => return None,
        };

        return Some(r);
    }
}

impl Data {
    pub const fn from_integer(x: u8) -> Option<Self> {
        let r = match x {
            1 => Self::Lsb,
            2 => Self::Msb,
            _ => return None,
        };

        return Some(r);
    }
}

impl Type {
    pub const fn from_integer(x: u16) -> Option<Self> {
        let t = match x {
            0 => Self::None,
            1 => Self::Relocatable,
            2 => Self::Executable,
            3 => Self::SharedObject,
            4 => Self::Core,
            _ => return None,
        };

        return Some(t);
    }
}

impl Machine {
    pub const fn from_integer(x: u16) -> Option<Self> {
        let machine = match x {
            0 => Machine::None,
            20 => Machine::PowerPC,
            21 => Machine::Power64,
            40 => Machine::Arm,
            3 => Machine::X86,
            62 => Machine::X64,
            183 => Machine::AArch64,
            224 => Machine::AmdGpu,
            243 => Machine::RiscV,
            _ => return None,
        };

        return Some(machine);
    }
}

impl OsAbi {
    pub const fn from_integer(x: u8) -> Option<Self> {
        let osabi = match x {
            0 => OsAbi::SystemV,
            1 => OsAbi::Hpux,
            2 => OsAbi::NetBSD,
            3 => OsAbi::GnuLinux,
            6 => OsAbi::Solaris,
            7 => OsAbi::Aix,
            8 => OsAbi::Irix,
            9 => OsAbi::FreeBSD,
            10 => OsAbi::Tru64,
            11 => OsAbi::Modesto,
            12 => OsAbi::OpenBSD,
            64 => OsAbi::ArmAEABI,
            97 => OsAbi::Arm,
            255 => OsAbi::Standalone,
            _ => return None,
        };

        return Some(osabi);
    }
}
impl SegmentType {
    pub const fn from_integer(x: u32) -> Option<Self> {
        let ret = match x {
            0 => Self::Null,
            1 => Self::Load,
            2 => Self::Dynamic,
            3 => Self::Interpreter,
            4 => Self::Note,
            5 => Self::SharedLib,
            6 => Self::ProgramHeader,
            7 => Self::ThreadLocalStorage,

            1685382481 => Self::OsSpecificGnuStack,
            1685382482 => Self::OsSpecificGnuRelro,
            1685382483 => Self::OsSpecificGnuProperty,

            0x6000_0000..=0x6FFF_FFFF => Self::OsSpecific(x),
            0x7000_0000..=0x7FFF_FFFF => Self::CpuSpecific(x),
            _ => return None,
        };

        return Some(ret);
    }
}

impl core::fmt::Debug for ProgramHeaderFlags {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut perms = ['_'; 3];
        if self.is_readable() {
            perms[0] = 'R';
        }
        if self.is_writable() {
            perms[1] = 'W';
        }
        if self.is_executable() {
            perms[2] = 'X';
        }
        return write!(f, "{}{}{}", perms[0], perms[1], perms[2]);
    }
}

impl core::fmt::Debug for ProgramHeader {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!(
            "ProgramHeader {{ offset: 0x{:016X}, vaddr: 0x{:016X}, \
            paddr: 0x{:016X}, filesz: {}, memsz: {}, p_align: 0x{:016X}, \
            type: {:?}, flags: {:?} }}",

            self.p_offset,
            self.p_vaddr,
            self.p_paddr,
            self.p_filesz,
            self.p_memsz,
            self.p_align,
            SegmentType::from_integer(self.p_type),
            self.p_flags,
        ))
    }
}

impl core::fmt::Debug for SectionHeader {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!(
            "SectionHeader {{ flags: 0x{:08X}, addr: 0x{:016X}, \
            offset: 0x{:016X}, size: {}, link: {}, info: {}, addralign: {}, \
            entsize: {}, name: {:?}, type: {:?} }}",

            self.sh_flags,
            self.sh_addr,
            self.sh_offset,
            self.sh_size,
            self.sh_link,
            self.sh_info,
            self.sh_addralign,
            self.sh_entsize,
            self.sh_name,
            SectionType::from_integer(self.sh_type),
        ))
    }
}

unsafe impl Zeroable for ProgramHeader {}
unsafe impl Pod for ProgramHeader {}

unsafe impl Zeroable for SectionHeader {}
unsafe impl Pod for SectionHeader {}

unsafe impl Zeroable for HeaderIdent {}
unsafe impl Pod for HeaderIdent {}

unsafe impl Zeroable for Header {}
unsafe impl Pod for Header {}
