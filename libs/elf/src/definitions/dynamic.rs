use bytemuck::{Pod, Zeroable};
use impl_bits::impl_bits;

#[repr(u64)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tag {
    Null = 0,
    Needed,
    PltRelSz,
    PltGot,
    Hash,
    StrTab,
    SymTab,
    Rela,
    RelaSz,
    RelaEnt,
    StrSz,
    SymEnt,
    Init,
    Fini,
    SoName,
    RPath,
    Symbolic,
    Rel,
    RelSz,
    RelEnt,
    PltRel,
    Debug,
    TextRel,
    JmpRel,
    BindNow,
    InitArray,
    FiniArray,
    InitArraySz,
    FiniArraySz,
    RunPath,
    Flags,
    Encoding,
    PreinitArray,
    PreinitArraySz,
    SymtabShndx,
    RelrSz,
    Relr,
    RelrEnt = 37,

    // sysv abi
    X86_64Plt = 0x7000_0000,
    X86_64PltSz = 0x7000_0001,
    X86_64PltEnt = 0x7000_0003,

    Flags1 = 0x6fff_fffb,
    GnuHash = 0x6fff_fef5,
}

#[repr(C)]
pub struct Entry {
    pub tag: Tag,
    pub val: u64,
}

#[repr(transparent)]
pub struct Flags(u64);
impl_bits!(Flags = {
    origin = 0,
    symbolic = 1,
    textrel = 2,
    bind_now = 3,
    static_tls = 4,
});

#[repr(transparent)]
pub struct Flags1(u64);
impl_bits!(Flags1 = {
    now = 0,
    global = 1,
    group = 2,
    nodelete = 3,
    loadfltr = 4,
    init_first = 5,
    no_open = 6,
    origin = 7,
    direct = 8,
    trans = 9,
    interpose = 10,
    no_deflib = 11,
    no_dump = 12,
    confalt = 13,
    end_filtee = 14,
    dispreldne = 15,
    disprelpnd = 16,
    no_direct = 17,
    ignmulder = 18,
    no_ksyms = 19,
    no_hdr = 20,
    edited = 21,
    no_reloc = 22,
    sym_interpose = 23,
    glob_audit = 24,
    singleton = 25,
    stub = 26,
    pie = 27,
    kmod = 28,
    weak_filter = 29,
    no_common = 30,
});

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Symbol {
    pub st_name: u32,
    pub st_info: u8,
    pub st_other: u8,
    pub st_shndx: u16,
    pub st_value: u64,
    pub st_size: u64,
}

impl core::fmt::Debug for Entry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.tag == Tag::Flags {
            return write!(f, "({:?}: {:?})", self.tag, Flags(self.val));
        }

        if self.tag == Tag::Flags1 {
            return write!(f, "({:?}: {:?})", self.tag, Flags1(self.val));
        }

        write!(f, "({:?}: {})", self.tag, self.val)
    }
}

unsafe impl Zeroable for Symbol {}
unsafe impl Pod for Symbol {}
