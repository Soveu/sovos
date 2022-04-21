use crate::*;

#[repr(C)]
pub struct GraphicsOutput {
    /// ## Parameters
    /// * This - The EFI_GRAPHICS_OUTPUT_PROTOCOL instance.
    /// * Type - EFI_GRAPHICS_OUTPUT_PROTOCOL is defined in this section.
    /// * ModeNumber - The mode number to return information on. The current mode and
    /// valid modes are read-only values in the Mode structure of the
    /// EFI_GRAPHICS_OUTPUT_PROTOCOL.
    /// * SizeOfInfo - A pointer to the size, in bytes, of the Info buffer.
    /// Info A pointer to a callee allocated buffer that returns information about
    /// ModeNumber.
    ///
    /// ## Description
    /// The QueryMode() function returns information for an available graphics
    /// mode that the graphics device and the set of active video output devices
    /// supports. If ModeNumber is not between 0 and MaxMode – 1, then
    /// EFI_INVALID_PARAMETER is returned. MaxMode is available from the Mode
    /// structure of the EFI_GRAPHICS_OUTPUT_PROTOCOL. The size of the Info
    /// structure should never be assumed and the value of SizeOfInfo is the
    /// only valid way to know the size of Info.  If the
    /// EFI_GRAPHICS_OUTPUT_PROTOCOL is installed on the handle that represents
    /// a single video output device, then the set of modes returned by this
    /// service is the subset of modes supported by both the graphics controller
    /// and the video output device. If the EFI_GRAPHICS_OUTPUT_PROTOCOL is
    /// installed on the handle that represents a combination of video output
    /// devices, then the set of modes returned by this service is the subset of
    /// modes supported by the graphics controller and the all of the video
    /// output devices represented by the handle.
    ///
    /// ## Status codes returned
    /// EFI_SUCCESS Valid mode information was returned.
    /// EFI_DEVICE_ERROR A hardware error occurred trying to retrieve the video mode.
    /// EFI_INVALID_PARAMETER ModeNumber is not valid.
    pub query_mode: Option<
        extern "efiapi" fn(
            this: &Self,
            mode_number: u32,
            size_of_info: &mut usize,
            &mut *const ModeInformation,
        ) -> RawStatus,
    >,

    /// ## Parameters
    /// * This - The EFI_GRAPHICS_OUTPUT_PROTOCOL instance.
    /// * Type - EFI_GRAPHICS_OUTPUT_PROTOCOL is defined in this section.
    /// * ModeNumber - Abstraction that defines the current video mode. The current mode
    /// and valid modes are read-only values in the Mode structure of the
    /// EFI_GRAPHICS_OUTPUT_PROTOCOL.
    ///
    /// ## Description This SetMode() function sets the graphics device and the
    /// set of active video output devices to the video mode specified by
    /// ModeNumber. If ModeNumber is not supported EFI_UNSUPPORTED is returned.
    /// If a device error occurs while attempting to set the video mode, then
    /// EFI_DEVICE_ERROR is returned.  Otherwise, the graphics device is set to
    /// the requested geometry, the set of active output devices are set to the
    /// requested geometry, the visible portion of the hardware frame buffer is
    /// cleared to black, and EFI_SUCCESS is returned.
    ///
    /// ## Status codes returned
    /// EFI_SUCCESS The graphics mode specified by ModeNumber was selected.
    /// EFI_DEVICE_ERROR The device had an error and could not complete the request.
    /// EFI_UNSUPPORTED ModeNumber is not supported by this device
    pub set_mode:
        Option<unsafe extern "efiapi" fn(this: &mut Self, mode_number: u32) -> RawStatus>,

    /// ## Parameters
    /// * This - The EFI_GRAPHICS_OUTPUT_PROTOCOL instance.
    /// * BltBuffer - The data to transfer to the graphics screen. Size is at
    /// least Width*Height*sizeof(EFI_GRAPHICS_OUTPUT_BLT_PIXEL).
    /// * BltOperation - The operation to perform when copying BltBuffer on to
    /// the graphics screen.
    /// * SourceX - The X coordinate of the source for the BltOperation. The
    /// origin of the screen is 0, 0 and that is the upper left-hand corner of
    /// the screen.
    /// * SourceY - The Y coordinate of the source for the BltOperation. The
    /// origin of the screen is 0, 0 and that is the upper left-hand corner of
    /// the screen.
    /// * DestinationX - The X coordinate of the destination for the
    /// BltOperation. The origin of the screen is 0, 0 and that is the upper
    /// left-hand corner of the screen.
    /// * DestinationY - The Y coordinate of the destination for the
    /// BltOperation. The origin of the screen is 0, 0 and that is the upper
    /// left-hand corner of the screen.
    /// * Width - The width of a rectangle in the blt rectangle in pixels. Each
    /// pixel is represented by an EFI_GRAPHICS_OUTPUT_BLT_PIXEL element.
    /// * Height - The height of a rectangle in the blt rectangle in pixels.
    /// Each pixel is represented by an EFI_GRAPHICS_OUTPUT_BLT_PIXEL element.
    /// * Delta - Not used for EfiBltVideoFill or the EfiBltVideoToVideo
    /// operation. If a Delta of zero is used, the entire BltBuffer is being
    /// operated on. If a subrectangle of the BltBuffer is being used then Delta
    /// represents the number of bytes in a row of the BltBuffer.
    ///
    /// ## Description
    /// The Blt() function is used to draw the BltBuffer rectangle onto the
    /// video screen.  The BltBuffer represents a rectangle of Height by Width
    /// pixels that will be drawn on the graphics screen using the operation
    /// specified by BltOperation.  The Delta value can be used to enable the
    /// BltOperation to be performed on a sub-rectangle of the BltBuffer. Table
    /// 113 describes the BltOperations that are supported on rectangles.
    /// Rectangles have coordinates (left, upper) (right, bottom):
    ///
    /// Table 113. Blt Operation Table
    /// Blt Operation Operation
    ///
    /// * EfiBltVideoFill - Write data from the BltBuffer pixel (0,0) directly
    /// to every pixel of the video display rectangle (DestinationX,
    /// DestinationY) (DestinationX + Width, DestinationY + Height). Only one
    /// pixel will be used from the BltBuffer. Delta is NOT used.
    ///
    /// * EfiBltVideoToBltBuffer - Read data from the video display rectangle
    /// (SourceX, SourceY) (SourceX + Width, SourceY + Height) and place it in
    /// the BltBuffer rectangle (DestinationX, DestinationY ) (DestinationX +
    /// Width, DestinationY + Height).  If DestinationX or DestinationY is not
    /// zero then Delta must be set to the length in bytes of a row in the
    /// BltBuffer.
    ///
    /// * EfiBltBufferToVideo - Write data from the BltBuffer rectangle
    /// (SourceX, SourceY) (SourceX + Width, SourceY + Height) directly to the
    /// video display rectangle (DestinationX, DestinationY) (DestinationX +
    /// Width, DestinationY + Height). If SourceX or SourceY is not zero then
    /// Delta must be set to the length in bytes of a row in the BltBuffer.
    ///
    /// * EfiBltVideoToVideo - Copy from the video display rectangle (SourceX,
    /// SourceY) (SourceX + Width, SourceY + Height) to the video display
    /// rectangle(DestinationX, DestinationY) (DestinationX + Width,
    /// DestinationY + Height. The BltBuffer and Delta are not used in this
    /// mode. There is no limitation on the overlapping of the source and
    /// destination rectangles.
    ///
    /// ## Status codes returned
    /// EFI_SUCCESS BltBuffer was drawn to the graphics screen.
    /// EFI_INVALID_PARAMETER BltOperation is not valid.
    /// EFI_DEVICE_ERROR The device had an error and could not complete the request
    pub blt: Option<
        unsafe extern "efiapi" fn(
            this: &mut Self,
            blt_buffer: Option<NonNull<BltPixel>>,
            blt_op: BltOperation,
            source_x: usize,
            source_y: usize,
            dest_x: usize,
            dest_y: usize,
            width: usize,
            height: usize,
            delta: Option<NonZeroUsize>,
        ) -> RawStatus,
    >,

    /// Pointer to EFI_GRAPHICS_OUTPUT_PROTOCOL_MODE data. Type
    /// EFI_GRAPHICS_OUTPUT_PROTOCOL_MODE is defined in “Related Definitions” below.
    pub mode: *const Mode,
}

#[repr(C)]
pub enum BltOperation {
    VideoFill,
    VideoToBltBuffer,
    BufferToVideo,
    VideoToVideo,
}

#[repr(C)]
pub struct BltPixel {
    pub blue:     u8,
    pub green:    u8,
    pub red:      u8,
    pub reserved: u8,
}

/// If a bit is set in RedMask, GreenMask, or BlueMask then those bits of the
/// pixel represent the corresponding color. Bits in RedMask, GreenMask,
/// BlueMask, and ReserverdMask must not over lap bit positions. The values for
/// the red, green, and blue components in the bit mask represent the color
/// intensity. The color intensities must increase as the color values for a
/// each color mask increase with a minimum intensity of all bits in a color
/// mask clear to a maximum intensity of all bits in a color mask set.
#[repr(C)]
pub struct PixelBitmask {
    pub red:      u32,
    pub green:    u32,
    pub blue:     u32,
    pub reserved: u32,
}

#[repr(C)]
pub enum PixelFormat {
    /// A pixel is 32-bits and byte zero represents red, byte one represents
    /// green, byte two represents blue, and byte three is reserved. This is the
    /// definition for the physical frame buffer. The byte values for the red,
    /// green, and blue components represent the color intensity. This color
    /// intensity value range from a minimum intensity of 0 to maximum intensity
    /// of 255.
    Rgbr8bpc,

    /// A pixel is 32-bits and byte zero represents blue, byte one represents
    /// green, byte two represents red, and byte three is reserved. This is the
    /// definition for the physical frame buffer. The byte values for the red,
    /// green, and blue components represent the color intensity. This color
    /// intensity value range from a minimum intensity of 0 to maximum intensity
    /// of 255.
    Bgrr8bpc,

    /// The pixel definition of the physical frame buffer is defined by
    /// EFI_PIXEL_BITMASK.
    Bitmask,

    /// This mode does not support a physical frame buffer.
    BltOnly,
}

#[repr(C)]
pub struct ModeInformation {
    /// The version of this data structure. A value of zero represents the
    /// EFI_GRAPHICS_OUTPUT_MODE_INFORMATION structure as defined in this
    /// specification.  Future version of this specification may extend this
    /// data structure in a backwards compatible way and increase the value of
    /// Version.
    pub version: u32,

    /// The size of video screen in pixels in the X dimension.
    pub horizontal_res: u32,

    /// The size of video screen in pixels in the Y dimension.
    pub vertical_res: u32,

    /// Enumeration that defines the physical format of the pixel. A value of
    /// PixelBltOnly implies that a linear frame buffer is not available for
    /// this mode.
    pub pixel_format: u32,

    /// This bit-mask is only valid if PixelFormat is set to PixelPixelBitMask. A bit
    /// being set defines what bits are used for what purpose such as Red, Green,
    /// Blue, or Reserved.
    pub pixel_info: PixelBitmask,

    /// Defines the number of pixel elements per video memory line. For
    /// performance reasons, or due to hardware restrictions, scan lines may be
    /// padded to an amount of memory alignment. These padding pixel elements
    /// are outside the area covered by HorizontalResolution and are not
    /// visible. For direct frame buffer access, this number is used as a span
    /// between starts of pixel lines in video memory. Based on the size of an
    /// individual pixel element and PixelsPerScanline, the offset in video
    /// memory from pixel element (x, y) to pixel element (x, y+1) has to be
    /// calculated as "sizeof( PixelElement ) * PixelsPerScanLine", not "sizeof(
    /// PixelElement ) * HorizontalResolution", though in many cases those
    /// values can coincide. This value can depend on video hardware and mode
    /// resolution. GOP implementation is responsible for providing accurate
    /// value for this field.
    pub pixels_per_scanline: u32,
}

#[repr(C)]
pub struct Mode {
    /// The number of modes supported by QueryMode() and SetMode().
    pub max_mode: u32,

    /// Current Mode of the graphics device. Valid mode numbers are 0 to MaxMode -1.
    pub mode: u32,

    /// Pointer to read-only EFI_GRAPHICS_OUTPUT_MODE_INFORMATION data.
    pub info: *const ModeInformation,

    /// Size of Info structure in bytes. Future versions of this specification may
    /// increase the size of the EFI_GRAPHICS_OUTPUT_MODE_INFORMATION data.
    pub size_of_info: usize,

    /// Base address of graphics linear frame buffer. Info contains information
    /// required to allow software to draw directly to the frame buffer without
    /// using Blt().Offset zero in FrameBufferBase represents the upper left
    /// pixel of the display.
    pub framebuffer_base: usize,

    /// Amount of frame buffer needed to support the active mode as defined by
    /// PixelsPerScanLine x VerticalResolution x PixelElementSize.
    pub framebuffer_size: usize,
}
