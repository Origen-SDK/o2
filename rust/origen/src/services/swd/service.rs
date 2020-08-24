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
    swdclk: String,
    swdio: String,
}

#[allow(non_snake_case)]
impl Service {
    pub fn new() -> Self {
        Self {
            swdclk: "swdclk".to_string(),
            swdio: "swdio".to_string()
        }
    }

    pub fn write_ap(&self, value: Value, A: u32, ack: Acknowledgements) -> Result<()> {
        let trans = match value {
            Value::Bits(bits, _size) => node!(
                SWDWriteAP,
                bits.data()?,
                A,
                ack,
                Some(bits.overlay_enables()),
                bits.get_overlay()?
            ),
            Value::Data(value, _size) => node!(
                SWDWriteAP,
                value,
                A,
                ack,
                None,
                None
            )
        };
        TEST.push(trans);
        Ok(())
    }

    pub fn verify_ap(&self, value: Value, A: u32, ack: Acknowledgements, parity: Option<bool>) -> Result<()> {
        let trans = match value {
            Value::Bits(bits, _size) => node!(
                SWDVerifyAP,
                bits.data()?,
                A,
                ack,
                parity,
                Some(bits.verify_enables()),
                Some(bits.capture_enables()),
                Some(bits.overlay_enables()),
                bits.get_overlay()?
            ),
            Value::Data(value, _size) => node!(
                SWDVerifyAP,
                value,
                A,
                ack,
                parity,
                Some(num::BigUint::from(0xFFFF_FFFF as usize)),
                None,
                None,
                None
            )
        };
        TEST.push(trans);
        Ok(())
    }

    pub fn write_dp(&self, value: Value, A: u32, ack: Acknowledgements) -> Result<()> {
        let trans = match value {
            Value::Bits(bits, _size) => node!(
                SWDWriteDP,
                bits.data()?,
                A,
                ack,
                Some(bits.overlay_enables()),
                bits.get_overlay()?
            ),
            Value::Data(value, _size) => node!(
                SWDWriteDP,
                value,
                A,
                ack,
                None,
                None
            )
        };
        TEST.push(trans);
        Ok(())
    }

    pub fn verify_dp(&self, value: Value, A: u32, ack: Acknowledgements, parity: Option<bool>) -> Result<()> {
        let trans = match value {
            Value::Bits(bits, _size) => node!(
                SWDVerifyDP,
                bits.data()?,
                A,
                ack,
                parity,
                Some(bits.verify_enables()),
                Some(bits.capture_enables()),
                Some(bits.overlay_enables()),
                bits.get_overlay()?
            ),
            Value::Data(value, _size) => node!(
                SWDVerifyDP,
                value,
                A,
                ack,
                parity,
                Some(num::BigUint::from(0xFFFF_FFFF as usize)),
                None,
                None,
                None
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