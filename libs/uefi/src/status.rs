#[repr(transparent)]
pub struct RawStatus(pub usize);

impl RawStatus {
    pub const OK: Self = Self(0);

    pub fn get_efi_error(&self) -> Option<Error> {
        const EFI_ERROR_BEGIN: usize = 0x8000_0000_0000_0000 + Error::LoadError as usize;
        const EFI_ERROR_END: usize = EFI_ERROR_BEGIN + Error::HttpError as usize;

        if self.0 < EFI_ERROR_BEGIN || self.0 > EFI_ERROR_END {
            return None;
        }

        let error_code = self.0 - EFI_ERROR_BEGIN + Error::LoadError as usize;
        return unsafe {
            Some(core::mem::transmute(error_code as u8))
        };
    }

    pub fn get_efi_warning(&self) -> Option<Warning> {
        const EFI_WARN_BEGIN: usize = 0x0000_0000_0000_0000 + Warning::UnknownGlyph as usize;
        const EFI_WARN_END: usize = EFI_WARN_BEGIN + Warning::ResetRequired as usize;

        if self.0 < EFI_WARN_BEGIN || self.0 > EFI_WARN_END {
            return None;
        }

        let warn_code = self.0 - EFI_WARN_BEGIN + Warning::UnknownGlyph as usize;
        return unsafe {
            Some(core::mem::transmute(warn_code as u8))
        };
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum Error {
    LoadError = 1,
    InvalidParameter,
    Unsupported,
    BadBufferSize,
    BufferTooSmall,
    NotReady,
    DeviceError,
    WriteProtected,
    OutOfResources,
    VolumeCorrupted,
    VolumeFull,
    NoMedia,
    MediaChanged,
    NotFound,
    AccessDenied,
    NoResponse,
    NoMapping,
    Timeout,
    NotStarted,
    AlreadyStarted,
    Aborted,
    IcmpError,
    TftpError,
    ProtocolError,
    IncompatibleVersion,
    SecurityViolation,
    CrcError,
    EndOfMedia,
    EndOfFile,
    InvalidLanguage,
    CompromisedData,
    IpAddressConflict,
    HttpError,
}

#[derive(PartialEq, Eq, Debug)]
pub enum Warning {
    UnknownGlyph = 1,
    DeleteFailure,
    WriteFailure,
    BufferTooSmall,
    StaleData,
    Filesystem,
    ResetRequired,
}


