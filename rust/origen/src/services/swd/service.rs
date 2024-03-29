use crate::core::dut::Dut;
use crate::generator::PAT;
use crate::precludes::controller::*;
use crate::{Result, TEST};

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
            _ => Err(error!("No matching SWD acknowledgment for '{}'", ack)),
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            Self::Ok => "Ok",
            Self::Wait => "Wait",
            Self::Fault => "Fault",
            Self::None => "None",
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
    pub id: usize,
    pub swdclk: (String, usize),
    pub swdio: (String, usize),
    pub trn: u32,
}

#[allow(non_snake_case)]
impl Service {
    pub fn new(
        _dut: &Dut,
        id: usize,
        swdclk: Option<&PinGroup>,
        swdio: Option<&PinGroup>,
    ) -> Result<Self> {
        Ok(Self {
            id: id,
            swdclk: {
                if let Some(grp) = swdclk {
                    grp.to_identifier()
                } else {
                    ("swdclk".to_string(), 0)
                }
            },
            swdio: {
                if let Some(grp) = swdio {
                    grp.to_identifier()
                } else {
                    ("swdio".to_string(), 0)
                }
            },
            trn: 1,
        })
    }

    pub fn write_ap(
        &self,
        dut: &crate::Dut,
        transaction: Transaction,
        ack: Acknowledgements,
    ) -> Result<()> {
        let swdio = PinCollection::from_group(dut, &self.swdio.0, self.swdio.1)?;
        let mut t2 = transaction.clone();
        t2.apply_overlay_pin_ids(&swdio.as_ids())?;
        let mut trans = node!(PAT::SWDWriteAP, self.id, t2, ack, None);
        let n_id = TEST.push_and_open(trans.clone());
        self.process_transaction(dut, &mut trans)?;
        TEST.close(n_id)?;
        Ok(())
    }

    pub fn verify_ap(
        &self,
        dut: &crate::Dut,
        transaction: Transaction,
        ack: Acknowledgements,
        parity: Option<bool>,
    ) -> Result<()> {
        let swdio = PinCollection::from_group(dut, &self.swdio.0, self.swdio.1)?;
        let mut t2 = transaction.clone();
        t2.apply_overlay_pin_ids(&swdio.as_ids())?;
        let mut trans = node!(PAT::SWDVerifyAP, self.id, t2, ack, parity, None);
        let n_id = TEST.push_and_open(trans.clone());
        self.process_transaction(dut, &mut trans)?;
        TEST.close(n_id)?;
        Ok(())
    }

    pub fn write_dp(
        &self,
        dut: &crate::Dut,
        transaction: Transaction,
        ack: Acknowledgements,
    ) -> Result<()> {
        let swdio = PinCollection::from_group(dut, &self.swdio.0, self.swdio.1)?;
        let mut t2 = transaction.clone();
        t2.apply_overlay_pin_ids(&swdio.as_ids())?;
        let mut trans = node!(PAT::SWDWriteDP, self.id, t2, ack, None);
        let n_id = TEST.push_and_open(trans.clone());
        self.process_transaction(dut, &mut trans)?;
        TEST.close(n_id)?;
        Ok(())
    }

    pub fn verify_dp(
        &self,
        dut: &crate::Dut,
        transaction: Transaction,
        ack: Acknowledgements,
        parity: Option<bool>,
    ) -> Result<()> {
        let swdio = PinCollection::from_group(dut, &self.swdio.0, self.swdio.1)?;
        let mut t2 = transaction.clone();
        t2.apply_overlay_pin_ids(&swdio.as_ids())?;
        let mut trans = node!(PAT::SWDVerifyDP, self.id, t2, ack, parity, None);
        let n_id = TEST.push_and_open(trans.clone());
        self.process_transaction(dut, &mut trans)?;
        TEST.close(n_id)?;
        Ok(())
    }
}
