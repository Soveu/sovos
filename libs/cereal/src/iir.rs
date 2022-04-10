#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FifoStatus {
    NotAvaliable,
    Enabled { functioning: bool },
    Reserved,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResetMethod {
    ReadMsr,
    ReadIirOrWriteThr,
    ReadRbr { third_bit: bool },
    ReadLsr,
    Reserved,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InterruptIdentification {
    pub fifo_status:       FifoStatus,
    pub reset_method:      ResetMethod,
    pub large_fifo_16750:  bool,
    pub interrupt_pending: bool,
}

impl InterruptIdentification {
    pub fn from_u8(x: u8) -> Self {
        let interrupt_pending = x & (1 << 0) != 0;
        let large_fifo_16750 = x & (1 << 5) != 0;
        let reset_method = match (x >> 1) & 0b111 {
            0b000 => ResetMethod::ReadMsr,
            0b001 => ResetMethod::ReadIirOrWriteThr,
            0b010 => ResetMethod::ReadRbr { third_bit: false },
            0b011 => ResetMethod::ReadLsr,
            0b110 => ResetMethod::ReadRbr { third_bit: true },
            _ => ResetMethod::Reserved,
        };
        let fifo_status = match x >> 6 {
            0b00 => FifoStatus::NotAvaliable,
            0b01 => FifoStatus::Reserved,
            0b10 => FifoStatus::Enabled { functioning: false },
            0b11 => FifoStatus::Enabled { functioning: true },
            _ => unreachable!(),
        };
        return Self { interrupt_pending, large_fifo_16750, reset_method, fifo_status };
    }
}
