use crate::impl_bits;

#[repr(transparent)]
pub struct ModemStatus(pub u8);

impl ModemStatus {
    pub const fn new() -> Self {
        Self(0)
    }

    pub const fn all_flags() -> Self {
        Self::__with_all_flags()
    }
}

impl_bits! {
    ModemStatus = {
        /// A note regarding the "delta" bits (Bits 0, 1, 2, and 3). In this case the word
        /// "delta" means change, as in a change in the status of one of the bits. This
        /// comes from other scientific areas like rocket science where delta-vee means a
        /// change in velocity. For the purposes of this register, each of these bits will
        /// be a logical "1" the next time you access this Modem Status register if the
        /// bit it is associated with (like Delta Data Carrier Detect with Carrier Detect)
        /// has changed its logical state from the previous time you accessed this
        /// register. The Trailing Edge Ring Indicator is pretty much like the rest,
        /// except it is in a logical "1" state only if the "Ring Indicator" bit went from
        /// a logical "1" to a logical "0" condition. There really isn't much practical
        /// use for this knowledge, but there is some software that tries to take
        /// advantage of these bits and perform some manipulation of the data received
        /// from the UART based on these bits. If you ignore these 4 bits you can still
        /// make a very robust serial communications software.
        delta_clear_to_send = 0,
        delta_data_set_ready = 1,
        trailing_edge_ring_indicator = 2,
        delta_data_carrier_detect = 3,

        /// The "Data Set Ready" and "Clear To Send" bits (Bits 4 and 5) are found
        /// directly on an RS-232 cable, and are matching wires to "Request To Send" and
        /// "Data Terminal Ready" that are transmitted with the "Modem Control Register
        /// (MCR). With these four bits in two registers, you can perform "hardware flow
        /// control", where you can signal to the other device that it is time to send
        /// more data, or to hold back and stop sending data while you are trying to
        /// process the information. More will be written about this subject in another
        /// module when we get to data flow control.
        clear_to_send = 4,
        data_set_ready = 5,

        /// Bits 7 and 6 are directly related to modem activity. Carrier Detect will stay
        /// in a logical state of "1" while the modem is "connect" to another modem. When
        /// this goes to a logical state of "0", you can assume that the phone connection
        /// has been lost. The Ring Indicator bit is directly tied to the RS-232 wire also
        /// labeled "RI" or Ring Indicator.  Usually this bit goes to a logical state of
        /// "1" as a result of the "ring voltage" on the telephone line is detected, like
        /// when a conventional telephone will be ringing to inform you that somebody is
        /// trying to call you.
        ring_indicator = 6,
        carrier_detect = 7,
    }
}
