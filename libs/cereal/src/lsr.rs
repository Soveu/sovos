use crate::impl_bits;

#[repr(transparent)]
pub struct LineStatus(pub u8);

impl LineStatus {
    pub const fn new() -> Self {
        Self(0)
    }

    pub const fn all_flags() -> Self {
        Self::__with_all_flags()
    }
}

impl_bits! {
    LineStatus = {
        /// The Data Ready Bit (Bit 0) is really the simplest part here. This is a way to
        /// simply inform you that there is data available for your software to extract
        /// from the UART.  When this bit is a logical "1", it is time to read the
        /// Receiver Buffer (RBR). On UARTs with a FIFO that is active, this bit will
        /// remain in a logical "1" state until you have read all of the contents of the
        /// FIFO.
        data_ready = 0,

        /// Overrun errors (Bit 1) are a sign of poor programming or an operating system
        /// that is not giving you proper access to the UART. This error condition occurs
        /// when there is a character waiting to be read, and the incoming shift register
        /// is attempting to move the contents of the next character into the Receiver
        /// Buffer (RBR). On UARTs with a FIFO, this also indicates that the FIFO is full
        /// as well.
        overrun_error = 1,

        /// Parity errors (Bit 2) can also indicate a mismatched baud rate like the
        /// framing errors (particularly if both errors are occurring at the same time).
        /// This bit is raised when the parity algorithm that is expected (odd, even,
        /// mark, or space) has not been found.  If you are using "no parity" in the setup
        /// of the UART, this bit should always be a logical "0". When framing errors are
        /// not occurring, this is a way to identify that there are some problems with the
        /// cabling, although there are other issues you may have to deal with as well.
        parity_error = 2,

        /// Framing errors (Bit 3) occur when the last bit is not a stop bit. Or to be
        /// more precise the stop bit is a logical "0". There are several causes for this,
        /// including that you have the timing between the two computer mismatched. This
        /// is usually caused by a mismatch in baud rate, although other causes might be
        /// involved as well, including problems in the physical cabling between the
        /// devices or that the cable is too long. You may even have the number of data
        /// bits off, so when errors like this are encountered, check the serial data
        /// protocol very closely to make sure that all of the settings for the UART (data
        /// bit length, parity, and stop bit count) are what should be expected.
        framing_error = 3,

        /// The Break Interrupt (Bit 4) gets to a logical state of "1" when the serial
        /// data input line has received "0" bits for a period of time that is at least as
        /// long as an entire serial data "word", including the start bit, data bits,
        /// parity bit, and stop bits, for the given baud rate in the Divisor Latch Bytes.
        /// (The normal state of a serial line is to send "1" bits when idle, or send
        /// start bit which is always one "0" bit, then send variable data and parity
        /// bits, then stop bit which is "1", continued into more "1"s if line goes idle.)
        /// A long sequence of "0" bits instead of the normal state usually means that the
        /// device that is sending serial data to your computer has stopped for some
        /// reason. Often with serial communications this is a normal condition, but in
        /// this way you have a way to monitor just how the other device is functioning.
        /// Some serial terminals have a key which make them generate this "break
        /// condition" as an out-of-band signaling method.
        break_interrupt = 4,

        /// Bits 5 and 6 refer to the condition of the character transmitter circuits and
        /// can help you to identify if the UART is ready to accept another character. Bit
        /// 6 is set to a logical "1" if all characters have been transmitted (including
        /// the FIFO, if active), and the "shift register" is done transmitting as well.
        /// This shift register is an internal memory block within the UART that grabs
        /// data from the Transmitter Holding Buffer (THB) or the FIFO and is the
        /// circuitry that does the actual transformation of the data to a serial format,
        /// sending out one bit of the data at a time and "shifting" the contents of the
        /// shift register down one bit to get the value of the next bit. Bit 5 merely
        /// tells you that the UART is capable of receiving more characters, including
        /// into the FIFO for transmitting.
        empty_transmitter_holding_reg = 5,
        empty_data_holding_regs = 6,

        /// Bit 7 refers to errors that are with characters in the FIFO. If any character
        /// that is currently in the FIFO has had one of the other error messages listed
        /// here (like a framing error, parity error, etc.), this is reminding you that
        /// the FIFO needs to be cleared as the character data in the FIFO is unreliable
        /// and has one or more errors. On UART chips without a FIFO this is a reserved
        /// bit field.
        fifo_error = 7,
    }
}
