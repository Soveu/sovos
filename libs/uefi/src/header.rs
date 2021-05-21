#[repr(C)]
pub struct TableHeader {
    /// A 64-bit signature that identifies the type of table that follows.
    /// Unique signatures have been generated for the EFI System Table, 
    /// the EFI Boot Services Table, and the EFI Runtime Services Table.
    pub signature: u64,

    pub revision: crate::Revision,

    /// The size, in bytes, of the entire table including the EFI_TABLE_HEADER.
    pub header_size: u32,

    /// The 32-bit CRC for the entire table. This value is computed by setting 
    /// this field to 0, and computing the 32-bit CRC for HeaderSize bytes.
    /// Note: Unless otherwise specified, UEFI uses a standard CCITT32 CRC algorithm with a seed
    /// polynomial value of 0x04c11db7 for its CRC calculations.
    pub crc32: u32,

    /// Reserved field that must be set to 0.
    _reserved: u32,
}

#[derive(PartialEq, Eq, Debug)]
pub enum VerifyError {
    Signature,
    Revision,
    TableSize,
}

pub trait Verify: Sized {
    const SIGNATURE: u64;
    const REVISION: crate::Revision = crate::SPECIFICATION_VERSION;

    fn get_header(&self) -> &TableHeader;

    fn verify_signature(&self) -> bool {
        self.get_header().signature == Self::SIGNATURE
    }
    fn verify_revision(&self) -> bool {
        self.get_header().revision >= Self::REVISION
    }
    fn verify_size(&self) -> bool {
        self.get_header().header_size as usize >= core::mem::size_of::<Self>()
    }

    fn verify(&self) -> Result<(), VerifyError> {
        if !self.verify_signature() {
            return Err(VerifyError::Signature);
        }
        if !self.verify_size() {
            return Err(VerifyError::TableSize);
        }
        if !self.verify_revision() {
            return Err(VerifyError::Revision);
        }

        return Ok(());
    }
}

