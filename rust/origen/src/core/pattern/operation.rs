//! Defines the set of operation types associated with pattern generation
//  This enum will be used by all action types

// May need to be more of these and more precise names
// TODO: The operation/action type enum should come from the module that models the object (pins, regs, protocol, etc.)
#[derive(Debug, Eq, PartialEq)]
pub enum Operation {
    Read,
    Write,
    Store,
    Capture,
    DriveMem,
    ReadMem,
    DriveFromSrc,
    StoreToCap,
    Start,
    Stop,
}

impl Operation {
    pub fn to_string(&self) -> String {
        match self {
            Operation::Read => "read".to_string(),
            Operation::Write => "write".to_string(),
            Operation::Store => "store".to_string(),
            Operation::Capture => "capture".to_string(),
            Operation::DriveMem => "drive_mem".to_string(),
            Operation::ReadMem => "read_mem".to_string(),
            Operation::DriveFromSrc => "drive_from_src".to_string(),
            Operation::StoreToCap => "store_to_cap".to_string(),
            Operation::Start => "start".to_string(),
            Operation::Stop => "stop".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_to_string() {
        assert_eq!(Operation::Read.to_string(), "read");
        assert_eq!(Operation::Write.to_string(), "write");
        assert_eq!(Operation::Store.to_string(), "store");
        assert_eq!(Operation::Capture.to_string(), "capture");
        assert_eq!(Operation::DriveMem.to_string(), "drive_mem");
        assert_eq!(Operation::ReadMem.to_string(), "read_mem");
        assert_eq!(Operation::DriveFromSrc.to_string(), "drive_from_src");
        assert_eq!(Operation::StoreToCap.to_string(), "store_to_cap");
        assert_eq!(Operation::Start.to_string(), "start");
        assert_eq!(Operation::Stop.to_string(), "stop");
    }
}
