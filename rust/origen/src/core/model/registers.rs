//! See section 3.5.1 in this doc for a good description of the IP-XACT data
//! structures upon which this is based:
//! https://www.accellera.org/images/downloads/standards/ip-xact/IP-XACT_User_Guide_2018-02-16.pdf

pub mod address_block;
pub mod bit;
pub mod bit_collection;
pub mod memory_map;
pub mod register;
pub mod register_file;

pub use address_block::AddressBlock;
pub use bit::Bit;
pub use bit_collection::BitCollection;
pub use memory_map::MemoryMap;
pub use register::Register;
pub use register::SummaryField;
pub use register_file::RegisterFile;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum AccessType {
    ReadWrite,
    ReadOnly,
    WriteOnly,
    ReadWriteOnce,
    WriteOnce,
    Unimplemented,
}

impl std::str::FromStr for AccessType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ReadWrite" | "rw" => Ok(AccessType::ReadWrite),
            "ReadOnly" | "ro" => Ok(AccessType::ReadOnly),
            "WriteOnly" | "wo" => Ok(AccessType::WriteOnly),
            "ReadWriteOnce" => Ok(AccessType::ReadWriteOnce),
            "WriteOnce" => Ok(AccessType::WriteOnce),
            _ => Err(format!("'{}' is not a valid value for AccessType", s)),
        }
    }
}

impl AccessType {
    pub fn is_readable(&self) -> bool {
        *self != AccessType::WriteOnly
    }

    pub fn is_writeable(&self) -> bool {
        *self != AccessType::ReadOnly && *self != AccessType::Unimplemented
    }

    pub fn is_writable(&self) -> bool {
        self.is_writeable()
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
