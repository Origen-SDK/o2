use crate::{Result, Error, Value, TEST};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Acknowledgements {
    Ok,
    Wait,
    Fault,
    None,
}

impl Acknowledgements {
    pub fn from_str(ack: &str) -> Result<Self> {
        match ack {
            "Ok" => Ok(Self::Ok),
            "Wait" => Ok(Self::Wait),
            "Fault" => Ok(Self::Fault),
            "None" => Ok(Self::None),
            _ => Err(Error::new(&format!(
                "No matching SWD acknowledgment for '{}'",
                ack
            )))
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            Self::Ok => "Ok",
            Self::Wait => "Wait",
            Self::Fault => "Fault",
            Self::None => "None"
        }
    }
}

#[macro_export]
macro_rules! swd_ok {
    () => {
        crate::services::swd::Acknowledgements::Ok
    };
}

#[derive(Clone, Debug)]
pub struct Service {
}

impl Service {
    pub fn new() -> Self {
        Self {}
    }

    pub fn write_ap(&self, value: Value, ap_addr: u32, ack: Acknowledgements) -> Result<()> {
        let trans = match value {
            Value::Bits(bits, _size) => node!(
                SWDWriteAP,
                bits.data()?,
                ap_addr,
                ack
            ),
            Value::Data(value, _size) => node!(
                SWDWriteAP,
                value,
                ap_addr,
                ack
            )
        };
        TEST.push(trans);
        Ok(())
    }

    pub fn verify_ap(&self, value: Value, ap_addr: u32, ack: Acknowledgements, parity: Option<bool>) -> Result<()> {
        let trans = match value {
            Value::Bits(bits, _size) => node!(
                SWDVerifyAP,
                bits.data()?,
                ap_addr,
                ack,
                parity
            ),
            Value::Data(value, _size) => node!(
                SWDVerifyAP,
                value,
                ap_addr,
                ack,
                parity
            )
        };
        TEST.push(trans);
        Ok(())
    }

    pub fn write_dp(&self, value: Value, dp_addr: u32, ack: Acknowledgements) -> Result<()> {
        let trans = match value {
            Value::Bits(bits, _size) => node!(
                SWDWriteDP,
                bits.data()?,
                dp_addr,
                ack
            ),
            Value::Data(value, _size) => node!(
                SWDWriteDP,
                value,
                dp_addr,
                ack
            )
        };
        TEST.push(trans);
        Ok(())
    }

    pub fn verify_dp(&self, value: Value, dp_addr: u32, ack: Acknowledgements, parity: Option<bool>) -> Result<()> {
        let trans = match value {
            Value::Bits(bits, _size) => node!(
                SWDVerifyDP,
                bits.data()?,
                dp_addr,
                ack,
                parity
            ),
            Value::Data(value, _size) => node!(
                SWDVerifyDP,
                value,
                dp_addr,
                ack,
                parity
            )
        };
        TEST.push(trans);
        Ok(())
    }

    pub fn line_reset(&self) -> Result<()> {
        TEST.push(node!(SWDLineReset));
        Ok(())
    }
}