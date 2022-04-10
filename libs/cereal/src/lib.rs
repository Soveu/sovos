#![no_std]

// All the documentation and function are from
// https://en.wikibooks.org/wiki/Serial_Programming/8250_UART_Programming
// TODO: add some function to set/clear Break Enable bit on LCR

// UART Registers
// Base Address   DLAB   I/O Access   Abbrv.   Register Name
// +0             0      Write        THR      Transmitter Holding Buffer
// +0             0      Read         RBR      Receiver Buffer
// +0             1      Read/Write   DLL      Divisor Latch Low Byte
// +1             0      Read/Write   IER      Interrupt Enable Register
// +1             1      Read/Write   DLH      Divisor Latch High Byte
// +2             x      Read         IIR      Interrupt Identification Register
// +2             x      Write        FCR      FIFO Control Register
// +3             x      Read/Write   LCR      Line Control Register
// +4             x      Read/Write   MCR      Modem Control Register
// +5             x      Read         LSR      Line Status Register
// +6             x      Read         MSR      Modem Status Register
// +7             x      Read/Write   SR       Scratch Register

use impl_bits::impl_bits;
use cpu::{inb, outb};

mod fcr;
mod ier;
mod iir;
mod lcr;
mod lsr;
mod msr;

pub use crate::fcr::*;
pub use crate::ier::*;
pub use crate::iir::*;
pub use crate::lcr::*;
pub use crate::lsr::*;
pub use crate::msr::*;

pub const CON1_PORT: u16 = 0x3F8;
pub const CON2_PORT: u16 = 0x2F8;

pub const DLL_OFFSET: u16 = 0;
pub const DLH_OFFSET: u16 = 1;
pub const IIR_OFFSET: u16 = 2;
pub const FCR_OFFSET: u16 = 2;
pub const LCR_OFFSET: u16 = 3;
pub const SCRATCH_OFFSET: u16 = 7;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SerialPortType {
    Uart8250,
    Uart16450,
    Uart16550,
    Uart16550A,
    Uart16750,
}

unsafe fn scratch_register_check(base_port: u16) -> SerialPortType {
    const RANDOM_BYTE: u8 = 42;
    let scratch_reg = base_port + SCRATCH_OFFSET;
    outb(scratch_reg, RANDOM_BYTE);
    return if inb(scratch_reg) == RANDOM_BYTE {
        SerialPortType::Uart16450
    } else {
        SerialPortType::Uart8250
    };
}

pub unsafe fn identify_uart(base_port: u16) -> SerialPortType {
    set_fifo_state(base_port, FifoState::Enabled, IntTriggerLevel::Bytes14);
    let iir = inb(base_port + IIR_OFFSET);
    let iir = InterruptIdentification::from_u8(iir);

    return match iir.fifo_status {
        FifoStatus::NotAvaliable | FifoStatus::Enabled { functioning: false } => {
            scratch_register_check(base_port)
        },
        FifoStatus::Reserved => SerialPortType::Uart16550, // what???
        FifoStatus::Enabled { functioning: true } => {
            if iir.large_fifo_16750 {
                SerialPortType::Uart16750
            } else {
                SerialPortType::Uart16550A
            }
        },
    };
}

/// SAFETY: port and divisor must be valid
pub unsafe fn set_baud_rate(base_port: u16, divisor: u16) {
    const DLAB_BIT: u8 = 1 << 7;

    let [divisor_high, divisor_low] = divisor.to_be_bytes();

    // Set DLAB
    let lcr_port = base_port + LCR_OFFSET;
    let new_lcr = inb(lcr_port) | DLAB_BIT;
    outb(lcr_port, new_lcr);

    // Set the divisor
    let dlh_port = base_port + DLH_OFFSET;
    let dll_port = base_port + DLL_OFFSET;
    outb(dlh_port, divisor_high);
    outb(dll_port, divisor_low);

    // Clear DLAB
    let new_lcr = inb(lcr_port) & !DLAB_BIT;
    outb(lcr_port, new_lcr);
}

/// SAFETY: serial port must be under this port
pub unsafe fn setup_serial_port_on(base_port: u16) {
    let serial_type = identify_uart(base_port);
    let fifo_state = if serial_type == SerialPortType::Uart16750 {
        FifoState::Large16750
    } else {
        FifoState::Enabled
    };
    set_fifo_state(base_port, fifo_state, IntTriggerLevel::Bytes14);
    set_baud_rate(base_port, 1); // SPEEED
}
