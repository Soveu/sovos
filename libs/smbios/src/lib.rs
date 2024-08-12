#![no_std]

pub mod v2;
pub mod v3;

mod text_iter;
pub use text_iter::*;

#[repr(C, packed)]
pub struct Header {
    pub typ:    u8,
    pub len:    u8,
    pub handle: u16,
}

#[repr(C)]
pub struct Entry {
    pub header: Header,
    pub data:   [u8],
}

#[repr(u8)]
pub enum HeaderType {
    Bios                           = 0,
    System,
    Mainboard,
    Chassis,
    Processor                      = 4,

    MemoryController               = 5,
    MemoryModule                   = 6,

    Cache                          = 7,
    PortConnector                  = 8,
    SystemSlots                    = 9,
    OnBoardDevices                 = 10,
    OemStrings                     = 11,
    SystemConfigOptions,
    BiosLang,
    GroupAssoc,
    SystemEventLog,
    PhysicalMemoryArray,
    MemoryDevice,
    U32MemoryErrorInformation,
    MemoryArrayMappedAddress,
    MemoryDeviceMappedAddress,
    BuiltinPointerDevice,
    PortableBattery,
    SystemReset,
    HardwareSecurity,
    SystemPowerControls,
    VoltageProbe,
    CoolingDevice,
    TemperatureProbe,
    ElectricalCurrentProbe,
    OutOfBandRemoteAccess,
    BootIntegrityServicesEntryPoint,
    SystemBoot                     = 32,

    U64MemoryErrorInformation      = 33,
    ManagmentDevice,
    ManagmentDeviceComponent,
    ManagmentDeviceThreshold,
    MemoryChannel,
    IpmiDevice,
    SystemPowerSupply,
    AdditionalInformation,
    OnboardDevicesExtended,
    ManagmentControllerHostInterface,
    TpmDevice,
    ProcessorAdditionalInformation = 44,

    Inactive                       = 126,
    EndOfTable                     = 127,
}

impl HeaderType {
    pub fn from_u8(x: u8) -> Option<Self> {
        let x = match x {
            0..=44 => unsafe { core::mem::transmute::<u8, HeaderType>(x) },
            126 => Self::Inactive,
            127 => Self::EndOfTable,
            _ => return None,
        };

        return Some(x);
    }
}
