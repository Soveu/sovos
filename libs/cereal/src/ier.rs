use crate::impl_bits;

#[repr(transparent)]
pub struct InterruptEnable(pub u8);

impl InterruptEnable {
    pub const fn new() -> Self {
        Self(0)
    }

    pub const fn all_flags() -> Self {
        Self::__with_all_flags()
    }
}

impl_bits! {
    InterruptEnable = {
        /// The Received Data interrupt is a way to let you know that there is some data
        /// waiting for you to pull off of the UART. This is probably the one bit that you
        /// will use more than the rest, and has more use.
        received_data_avaliable_interrupt = 0,

        /// The Transmitter Holding Register Empty Interrupt is to let you know that the
        /// output buffer (on more advanced models of the chip like the 16550) has
        /// finished sending everything that you pushed into the buffer. This is a way to
        /// streamline the data transmission routines so they take up less CPU time.
        transmitter_holding_register_empty = 1,

        /// The Receiver Line Status Interrupt indicates that something in the LSR
        /// register has probably changed. This is usually an error condition, and if you
        /// are going to write an efficient error handler for the UART that will give
        /// plain text descriptions to the end user of your application, this is something
        /// you should consider. This is certainly something that takes a bit more
        /// advanced knowledge of programming.
        receiver_line_status_interrupt = 2,

        /// The Modem Status Interrupt is to notify you when something changes with an
        /// external modem connected to your computer. This can include things like the
        /// telephone "bell" ringing (you can simulate this in your software), that you
        /// have successfully connected to another modem (Carrier Detect has been turned
        /// on), or that somebody has "hung up" the telephone (Carrier Detect has turned
        /// off). It can also help you to know if the external modem or data equipment can
        /// continue to receive data (Clear to Send).  Essentially, this deals with the
        /// other wires in the RS-232 standard other than strictly the transmit and
        /// receive wires.
        modem_status_interrupt = 3,

        /// The other two modes are strictly for the 16750 chip, and help put the chip
        /// into a "low power" state for use on things like a laptop computer or an
        /// embedded controller that has a very limited power source like a battery. On
        /// earlier chips you should treat these bits as "Reserved", and only put a "0"
        /// into them.
        sleep_mode_16750 = 4,
        low_power_mode_16750 = 5,
    }
}
