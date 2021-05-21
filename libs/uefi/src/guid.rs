#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct Guid(u32, u16, u16, [u8; 8]);

impl Guid {
    // ACPI 2.0 or newer tables should use EFI_ACPI_TABLE_GUID//
    pub const MPS_TABLE: Guid =
        Guid(0xeb9d2d2f, 0x2d88, 0x11d3, [0x9a,0x16,0x00,0x90,0x27,0x3f,0xc1,0x4d]);
    pub const EFI_ACPI_TABLE: Guid =
        Guid(0x8868e871, 0xe4f1, 0x11d3, [0xbc,0x22,0x00,0x80,0xc7,0x3c,0x88,0x81]);
    pub const ACPI_TABLE: Guid =
        Guid(0xeb9d2d30, 0x2d88, 0x11d3, [0x9a,0x16,0x00,0x90,0x27,0x3f,0xc1,0x4d]);
    pub const SMBIOS_TABLE: Guid =
        Guid(0xeb9d2d31, 0x2d88, 0x11d3, [0x9a,0x16,0x00,0x90,0x27,0x3f,0xc1,0x4d]);
    pub const SMBIOS3_TABLE: Guid =
        Guid(0xf2fd1544, 0x9794, 0x4a2c, [0x99,0x2e,0xe5,0xbb,0xcf,0x20,0xe3,0x94]);
    pub const SAL_SYSTEM_TABLE: Guid = 
        Guid(0xeb9d2d32, 0x2d88, 0x11d3, [0x9a,0x16,0x00,0x90,0x27,0x3f,0xc1,0x4d]);
}

impl core::fmt::Debug for Guid {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let table_name: &'static str = match *self {
            Self::MPS_TABLE => "MPS Table",
            Self::EFI_ACPI_TABLE => "EFI ACPI Table",
            Self::ACPI_TABLE => "ACPI Table",
            Self::SMBIOS_TABLE => "SMBIOS Table",
            Self::SMBIOS3_TABLE => "SMBIOS 3 Table",
            Self::SAL_SYSTEM_TABLE => "SAL System Table",
            _ => "",
        };

        if table_name == "" {
            return f.debug_tuple("Guid")
                .field(&self.0)
                .field(&self.1)
                .field(&self.2)
                .field(&self.3)
                .finish();
        }

        return f.debug_tuple("Guid").field(&table_name).finish();
    }
}
