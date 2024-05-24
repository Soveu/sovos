#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RawStatus(pub usize);

impl RawStatus {
    #[track_caller]
    fn get(&self) -> Result<Option<Warning>, Error> {
        const EFI_ERROR_BEGIN: usize = 0x8000_0000_0000_0000 + Error::LoadError as usize;
        const EFI_ERROR_END: usize = EFI_ERROR_BEGIN + Error::HttpError as usize;

        const EFI_WARN_BEGIN: usize =
            0x0000_0000_0000_0000 + Warning::UnknownGlyph as usize;
        const EFI_WARN_END: usize = EFI_WARN_BEGIN + Warning::ResetRequired as usize;

        if self.0 == 0 {
            return Ok(None);
        }

        if (EFI_ERROR_BEGIN..=EFI_ERROR_END).contains(&self.0) {
            let error_code = self.0 - EFI_ERROR_BEGIN + Error::LoadError as usize;
            /* SAFETY: we have just checked if the enum is in range */
            return unsafe { Err(core::mem::transmute(error_code as u8)) };
        }

        if (EFI_WARN_BEGIN..=EFI_WARN_END).contains(&self.0) {
            let warn_code = self.0 - EFI_WARN_BEGIN + Warning::UnknownGlyph as usize;
            /* SAFETY: we have just checked if the enum is in range */
            return unsafe { Ok(Some(core::mem::transmute(warn_code as u8))) };
        }

        panic!("Invalid UEFI status {:x}", self.0);
    }

    #[track_caller]
    pub fn ok_or_expect_errors(&self, expected_errors: &[Error]) -> Result<(), Error> {
        let error = match self.get() {
            Err(e) => e,
            Ok(None) => return Ok(()),
            Ok(Some(warn)) => {
                panic!("This crate doesn't handle warnings, got {:?}", warn)
            },
        };

        if expected_errors.iter().any(|&err| err == error) {
            return Err(error);
        }

        panic!(
            "Invalid UEFI implementation - got Error::{:?}, expected one of {:?}",
            error, expected_errors,
        );
    }

    pub const fn ok() -> Self {
        Self(0)
    }

    pub const fn from_warning(w: Warning) -> Self {
        Self(w as usize)
    }

    pub const fn from_error(e: Error) -> Self {
        Self(0x8000_0000_0000_000 + e as usize)
    }

    pub const fn is_ok(self) -> bool {
        self.0 == 0
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
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

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Warning {
    UnknownGlyph = 1,
    DeleteFailure,
    WriteFailure,
    BufferTooSmall,
    StaleData,
    Filesystem,
    ResetRequired,
}
