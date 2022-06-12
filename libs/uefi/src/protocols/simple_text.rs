use core::fmt;

use crate::*;

macro_rules! uefi_fn_ptr {
    ($($arg:tt)*) => { Option<unsafe extern "efiapi" fn($($arg)*) -> RawStatus> };
}

#[repr(C)]
pub struct OutputMode {
    pub max_mode:       i32,
    pub mode:           i32,
    pub attribute:      i32,
    pub cursor_column:  i32,
    pub cursor_row:     i32,
    pub cursor_visible: i32,
}

#[repr(C)]
pub struct Output {
    /// ## Parameters
    ///
    /// * This - A pointer to the `SimpleTextOutputProtocol` instance.
    /// * ExtendedVerification - Indicates that the driver may perform a more exhaustive
    ///   verification operation of the device during reset
    ///
    /// ## Description
    ///
    /// The Reset() function resets the text output device hardware.
    /// The cursor position is set to (0, 0), and the screen is cleared to
    /// the default background color for the output device.
    /// As part of initialization process, the firmware/device will make a quick
    /// but reasonable attempt to verify that the device is functioning.
    /// If the ExtendedVerification flag is TRUE the firmware may take an
    /// extended amount of time to verify the device is operating on reset.
    /// Otherwise the reset operation is to occur as quickly as possible.
    /// The hardware verification process is not defined by this specification
    /// and is left up to the platform firmware or driver to implement.
    pub reset: uefi_fn_ptr!(this: &mut Self, extended_verification: bool),

    /// # Parameters
    ///
    /// * `this` - A pointer to the `SimpleTextOutputProtocol` instance.
    /// * `string` - The Null-terminated string to be displayed on the output device(s).
    ///   All output devices must also support the Unicode drawing character codes
    ///   defined in "Related Definitions."
    pub output_string: uefi_fn_ptr!(this: &mut Self, string: *const u16),

    /// # Parameters
    ///
    /// * `this` - A pointer to the `SimpleTextOutputProtocol` instance.
    /// * `string` - The Null-terminated string to be displayed on the output device(s).
    ///   All output devices must also support the Unicode drawing character codes
    ///   defined in “Related Definitions.”
    ///
    /// # Description
    ///
    /// The TestString() function verifies that all characters in a string
    /// can be output to the target device. This function provides a way to
    /// know if the desired character codes are supported for rendering on the
    /// output device(s). This allows the installation procedure (or EFI image)
    /// to at least select character codes that the output devices are capable
    /// of displaying. Since the output device(s) may be changed between boots,
    /// if the loader cannot adapt to such changes it is recommended that the
    /// loader call OutputString() with the text it has and ignore any
    /// "unsupported" error codes. Devices that are capable of displaying the
    /// Unicode character codes will do so.
    pub test_string: uefi_fn_ptr!(this: &Self, string: *const u16),

    pub query_mode:    usize,
    pub set_mode:      usize,
    pub set_attribute: usize,

    /// # Parameters
    ///
    /// * `this` - A pointer to the `SimpleTextOutputProtocol` instance.
    ///
    /// # Description
    ///
    /// The ClearScreen() function clears the output device(s) display to the
    /// currently selected background color. The cursor position is set to (0,
    /// 0)
    pub clear_screen: uefi_fn_ptr!(this: &mut Self),

    pub set_cursor_position: usize,
    pub enable_cursor:       usize,

    pub mode: *const OutputMode,
}

#[derive(Clone, Copy, Debug)]
pub enum Verification {
    None     = 0,
    Extended = 1,
}

impl Verification {
    const fn to_bool(self) -> bool {
        match self {
            Self::None => false,
            Self::Extended => true,
        }
    }
}

impl Output {
    pub fn reset(&mut self, ver: Verification) -> Result<(), Error> {
        let f = self.reset.expect("buggy UEFI: simple_text::Output::reset is null");
        let result = unsafe { (f)(self, ver.to_bool()) };
        return result.ok_or_expect_errors(&[Error::DeviceError]);
    }

    unsafe fn test_raw_utf16(&self, s: *const u16) -> Result<(), Error> {
        let f =
            self.test_string.expect("buggy UEFI: simple_text::Output::test_string is null");
        let result = (f)(self, s);
        return result.ok_or_expect_errors(&[Error::Unsupported]);
    }

    unsafe fn print_raw_utf16(&mut self, s: *const u16) -> Result<(), Error> {
        let f = self
            .output_string
            .expect("buggy UEFI: simple_text::Output::output_string is null");
        let result = (f)(self, s);
        return result.ok_or_expect_errors(&[Error::Unsupported, Error::DeviceError]);
    }

    pub fn print_utf8(&mut self, s: &str) -> Result<(), Error> {
        let mut utf16_buf = [0u16; 256];
        let mut i = 0usize;

        for c in s.chars() {
            if i == 250 {
                utf16_buf[i] = 0; // insert the final null byte
                unsafe {
                    self.test_raw_utf16(utf16_buf.as_ptr())?;
                    self.print_raw_utf16(utf16_buf.as_ptr())?;
                }
                i = 0;
            }

            if c == '\n' {
                utf16_buf[i] = b'\r' as u16;
                i += 1;
            }

            utf16_buf[i] = if c as u32 > u16::MAX as u32 { '?' as u16 } else { c as u16 };

            i += 1;
        }

        utf16_buf[i] = 0;
        unsafe {
            self.test_raw_utf16(utf16_buf.as_ptr())?;
            self.print_raw_utf16(utf16_buf.as_ptr())?;
        }

        return Ok(());
    }
}

impl fmt::Write for Output {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        match self.print_utf8(s) {
            Ok(()) => Ok(()),
            Err(_) => Err(fmt::Error),
        }
    }
}
