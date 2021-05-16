#[repr(C)]
pub struct TableHeader {
    /// A 64-bit signature that identifies the type of table that follows.
    /// Unique signatures have been generated for the EFI System Table, 
    /// the EFI Boot Services Table, and the EFI Runtime Services Table.
    pub signature: u64,

    /// The revision of the EFI Specification to which this table conforms.
    /// The upper 16 bits of this field contain the major revision value,
    /// and the lower 16 bits contain the minor revision value.
    /// The minor revision values are binary coded decimals and are limited to the
    /// range of 00..99.
    /// When printed or displayed UEFI spec revision is referred as 
    /// (Major revision).(Minor revision upper decimal).(Minor revision lower decimal) 
    /// or (Major revision).(Minor revision upper decimal) in case Minor revision 
    /// lower decimal is set to 0. 
    /// For example:
    /// A specification with the revision value ((2<<16) | (30)) would be referred as 2.3;
    /// A specification with the revision value ((2<<16) | (31)) would be referred as 2.3.1
    pub revision: u32,

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
    const REVISION: u32 = super::SPECIFICATION_VERSION;

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

