use crate::{outb, LCR_OFFSET};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Parity {
    /// When Bit 3 is a logical "0", this causes no parity bits to be sent out with the
    /// serial data word. Instead it moves on immediately to the stop bits, and is an
    /// admission that parity checking at this level is really useless.
    None  = 0b000,

    /// Each bit in the data portion of the serial word is added as a simple count of the
    /// number of logical "1" bits. If this is an odd number of bits, the parity bit will
    /// be transmitted as a logical "0". If the count is even, the parity bit will be
    /// transmitted as a logical "1" to make the number of "1" bits odd.
    Odd   = 0b001,

    /// Like Odd Parity, the bits are added together. In this case, however, if the
    /// number of bits end up as an odd number it will be transmitted as a logical
    /// "1" to make the number of "1" bits even, which is the exact opposite of odd
    /// parity.
    Even  = 0b011,

    /// In this case the parity bit will always be a logical "1". While this may seem a
    /// little unusual, this is put in for testing and diagnostics purposes. If you want
    /// to make sure that the software on the receiving end of the serial connection is
    /// responding correctly to a parity error, you can send a Mark or a Space parity,
    /// and send characters that don't meet what the receiving UART or device is
    /// expecting for parity. In addition for Mark Parity only, you can use this bit
    /// as an extra "stop bit". Keep in mind that RS-232 standards are expecting a
    /// logical "1" to end a serial data word, so a receiving computer will not be
    /// able to tell the difference between a "Mark" parity bit and a stop bit. In
    /// essence, you can have 3 or 2.5 stop bits through the use of this setting and
    /// by appropriate use of the   stop bit portion of this register as well. This
    /// is a way to "tweak" the settings   on your computer in a way that typical
    /// applications don't allow you to do, or at   least gain a deeper insight into
    /// serial data settings.
    Mark  = 0b101,

    /// Like the Mark parity, this makes the parity bit "sticky", so it doesn't change.
    /// In this case it puts in a logical "0" for the parity bit every time you
    /// transmit a character. There are not many practical uses for doing this other
    /// than a crude way to put in 9 data bits for each serial word, or for
    /// diagnostics purposes as described above.
    Space = 0b111,
}

/// The first two bits (Bit 0 and Bit 1) control how many data bits are sent for each data
/// "word" that is transmitted via serial protocol. For most serial data transmission,
/// this will be 8 bits, but you will find some of the earlier protocols and older
/// equipment that will require fewer data bits. For example, some military encryption
/// equipment only uses 5 data bits per serial "word", as did some TELEX equipment. Early
/// ASCII teletype terminals only used 7 data bits, and indeed this heritage has been
/// preserved with SMTP format that only uses 7-bit ASCII for e-mail messages. Clearly
/// this is something that needs to be established before you are able to successfully
/// complete message transmission using RS-232 protocol.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum WordLength {
    Bits5 = 0b00,
    Bits6 = 0b01,
    Bits7 = 0b10,
    Bits8 = 0b11,
}

/// Bit 2 controls how many stop bits are transmitted by the UART to the receiving device.
/// This is selectable as either one or two stop bits, with a logical "0" representing 1
/// stop bit and "1" representing 2 stop bits. In the case of 5 data bits, the UART
/// instead sends out "1.5 stop bits". Remember that a 'bit' in this context is actually a
/// time interval: at 50 baud (bits per second) each bit takes 20 ms. So "1.5 stop bits"
/// would have a minimum of 30 ms between characters. This is tied to the "5 data bits"
/// setting, since only the equipment that used 5-bit Baudot rather than 7- or 8-bit ASCII
/// used "1.5 stop bits".
///
/// Another thing to keep in mind is that the RS-232 standard only specifies that at least
/// one data bit cycle will be kept a logical "1" at the end of each serial data word (in
/// other words, a complete character from start bit, data bits, parity bits, and stop
/// bits). If you are having timing problems between the two computers but are able to in
/// general get the character sent across one at a time, you might want to add a second
/// stop bit instead of reducing baud rate.  This adds a one-bit penalty to the
/// transmission speed per character instead of halving the transmission speed by dropping
/// the baud rate (usually).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StopBits {
    One  = 0,
    More = 1,
}

pub unsafe fn setup_lcr(
    base_port: u16,
    word_len: WordLength,
    stop_bits: StopBits,
    parity: Parity,
) {
    let word_len = word_len as u8;
    let stop_bits = stop_bits as u8;
    let parity = parity as u8;

    let lcr_port = base_port + LCR_OFFSET;
    #[rustfmt::skip]
    let bits = 0u8
        | (word_len << 0)
        | (stop_bits << 2)
        | (parity<< 3)
        | (0u8 << 6)  // clear "break enable"
        | (0u8 << 7); // clear "divisor latch access"
    outb(lcr_port, bits);
}
