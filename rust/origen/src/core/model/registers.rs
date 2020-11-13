//! See section 3.5.1 in this doc for a good description of the IP-XACT data
//! structures upon which this is based:
//! https://www.accellera.org/images/downloads/standards/ip-xact/IP-XACT_User_Guide_2018-02-16.pdf

pub mod address_block;
pub mod bit;
pub mod bit_collection;
pub mod field;
pub mod macro_api;
pub mod memory_map;
pub mod register;
pub mod register_file;

pub use address_block::AddressBlock;
pub use bit::Bit;
pub use bit_collection::BitCollection;
pub use field::{Field, SummaryField};
pub use memory_map::MemoryMap;
pub use register::Register;
pub use register_file::RegisterFile;

use std::fmt;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum AccessType {
    RO,            // Read-Only
    RW,            // Read-Write
    RC,            // Read-only, Clear-on-read
    RS,            // Set-on-read (all bits become '1' on read)
    WRC,           // Writable, clear-on-read
    WRS,           // Writable, Sets-on-read
    WC,            // Clear-on-write
    WS,            // Set-on-write
    WSRC,          // Set-on-write, clear-on-read
    WCRS,          // Clear-on-write, set-on-read
    W1C,           // Write '1' to clear bits
    W1S,           // Write '1' to set bits
    W1T,           // Write '1' to toggle bits
    W0C,           // Write '0' to clear bits
    W0S,           // Write '0' to set bits
    W0T,           // Write '0' to toggle bits
    W1SRC,         // Write '1' to set and clear-on-read
    W1CRS,         // Write '1' to clear and set-on-read
    W0SRC,         // Write '0' to set and clear-on-read
    W0CRS,         // Write '0' to clear and set-on-read
    WO,            // Write-only
    WOC,           // When written sets the field to '0'. Read undeterministic
    WORZ,          // Write-only, Reads zero
    WOS,           // When written sets all bits to '1'. Read undeterministic
    W1,            // Write-once. Next time onwards, write is ignored. Read returns the value
    WO1,           // Write-once. Next time onwards, write is ignored. Read is undeterministic
    DC,            // RW but no check
    ROWZ,          // Read-only, value is cleared on read
    Unimplemented, // Means an unimplemented bit in the Origen model
}

impl std::str::FromStr for AccessType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "RO" | "ro" => Ok(AccessType::RO),
            "RW" | "rw" => Ok(AccessType::RW),
            "RC" | "rc" => Ok(AccessType::RC),
            "RS" | "rs" => Ok(AccessType::RS),
            "WRC" | "wrc" => Ok(AccessType::WRC),
            "WRS" | "wrs" => Ok(AccessType::WRS),
            "WC" | "wc" => Ok(AccessType::WC),
            "WS" | "ws" => Ok(AccessType::WS),
            "WSRC" | "wsrc" => Ok(AccessType::WSRC),
            "WCRS" | "wcrs" => Ok(AccessType::WCRS),
            "W1C" | "w1c" => Ok(AccessType::W1C),
            "W1S" | "w1s" => Ok(AccessType::W1S),
            "W1T" | "w1t" => Ok(AccessType::W1T),
            "W0C" | "w0c" => Ok(AccessType::W0C),
            "W0S" | "w0s" => Ok(AccessType::W0S),
            "W0T" | "w0t" => Ok(AccessType::W0T),
            "W1SRC" | "w1src" => Ok(AccessType::W1SRC),
            "W1CRS" | "w1crs" => Ok(AccessType::W1CRS),
            "W0SRC" | "w0src" => Ok(AccessType::W0SRC),
            "W0CRS" | "w0crs" => Ok(AccessType::W0CRS),
            "WO" | "wo" => Ok(AccessType::WO),
            "WOC" | "woc" => Ok(AccessType::WOC),
            "WORZ" | "worz" => Ok(AccessType::WORZ),
            "WOS" | "wos" => Ok(AccessType::WOS),
            "W1" | "w1" => Ok(AccessType::W1),
            "WO1" | "wo1" => Ok(AccessType::WO1),
            "DC" | "dc" => Ok(AccessType::DC),
            "ROWZ" | "rowz" => Ok(AccessType::ROWZ),
            _ => Err(format!("'{}' is not a valid value for AccessType", s)),
        }
    }
}

impl fmt::Display for AccessType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}

impl AccessType {
    pub fn is_readable(&self) -> bool {
        *self != AccessType::WO
    }

    pub fn is_writeable(&self) -> bool {
        *self != AccessType::RO && *self != AccessType::Unimplemented
    }

    pub fn is_writable(&self) -> bool {
        self.is_writeable()
    }

    pub fn is_unimplemented(&self) -> bool {
        *self == AccessType::Unimplemented
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum BitOrder {
    LSB0,
    MSB0,
}

impl std::str::FromStr for BitOrder {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "MSB0" | "msb0" => Ok(BitOrder::MSB0),
            "LSB0" | "lsb0" => Ok(BitOrder::LSB0),
            _ => Err(format!("'{}' is not a valid value for BitOrder", s)),
        }
    }
}

//#[derive(Debug)]
//pub enum Usage {
//    Read,
//    Write,
//    ReadWrite,
//}
