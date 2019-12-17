//! Defines the set of actions associated with pattern generation

pub enum Operation {
    Read,
    Write,
    Store,
    Drivemem,
    Start,
    Stop,
}

impl Operation {
    pub fn to_string(&self) -> String {
        match self {
            Operation::Read => "read".to_string(),
            Operation::Write => "write".to_string(),
            Operation::Store => "store".to_string(),
            Operation::Drivemem => "drivemem".to_string(),
            Operation::Start => "start".to_string(),
            Operation::Stop => "stop".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::pattern::operation::Operation;

    #[test]
    fn converts_to_string() {
        assert_eq!(Operation::Read.to_string(), "read");
        assert_eq!(Operation::Write.to_string(), "write");
        assert_eq!(Operation::Store.to_string(), "store");
        assert_eq!(Operation::Drivemem.to_string(), "drivemem");
        assert_eq!(Operation::Start.to_string(), "start");
        assert_eq!(Operation::Stop.to_string(), "stop");
    }
}
