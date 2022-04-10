use crate::{inb, outb, FCR_OFFSET};

#[derive(Clone, Copy)]
pub enum IntTriggerLevel {
    Bytes1  = 0,
    Bytes4  = 1,
    Bytes8  = 2,
    Bytes14 = 3,
}

#[derive(Clone, Copy)]
pub enum FifoState {
    Disabled   = 0,
    Enabled    = 0b0000_0001,
    Large16750 = 0b0010_0001,
}

/// SAFETY: serial port must be valid and `FifoState::Large16750` cannot
/// be set for older UARTs.
///
/// Also remember that it effectively quadruples interrupt trigger level.
pub unsafe fn set_fifo_state(
    base_port: u16,
    fifo_state: FifoState,
    int_trigger_level: IntTriggerLevel,
) {
    let int_trigger_level = (int_trigger_level as u8) << 6;
    let fifo_state = fifo_state as u8;
    let fcr_port = base_port + FCR_OFFSET;
    outb(fcr_port, int_trigger_level | fifo_state);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FifoSize {
    Bytes16 = 16,
    Bytes64 = 64,
}

/// This is the number of characters that would be stored in the FIFO before an interrupt
/// is triggered that will let you know data should be removed from the FIFO. If you
/// anticipate that large amounts of data will be sent over the serial data link, you
/// might want to increase the size of the buffer. The reason why the maximum value for
/// the trigger is less than the size of the FIFO buffer is because it may take a little
/// while for some software to access the UART and retrieve the data. Remember that when
/// the FIFO is full, you will start to lose data from the FIFO, so it is important to
/// make sure you have retrieved the data once this threshold has been reached. If you are
/// encountering software timing problems in trying to retrieve the UART data, you might
/// want to lower the threshold value. At the extreme end where the threshold is set to 1
/// byte, it will act essentially like the basic 8250, but with the added reliability that
/// some characters may get caught in the buffer in situations where you don't have a
/// chance to get all of them immediately.
pub fn calculate_interrupt_trigger_level(bits: u8, sz: FifoSize) -> u8 {
    match (sz, bits & 0b11) {
        (FifoSize::Bytes16, 0b00) => 1,
        (FifoSize::Bytes16, 0b01) => 4,
        (FifoSize::Bytes16, 0b10) => 8,
        (FifoSize::Bytes16, 0b11) => 14,

        (FifoSize::Bytes64, 0b00) => 1,
        (FifoSize::Bytes64, 0b01) => 16,
        (FifoSize::Bytes64, 0b10) => 32,
        (FifoSize::Bytes64, 0b11) => 56,

        _ => unreachable!(),
    }
}
