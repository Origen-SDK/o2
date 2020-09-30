use crate::{Result, Error, TEST};
use crate::core::dut::Dut;
use indexmap::IndexMap;
use crate::Transaction;
use crate::core::model::pins::PinBus;

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
    pub id: usize,
    pub swdclk: PinBus,
    pub swdio: PinBus,
    pub swdclk_id: Vec<usize>,
    pub swdclk_grp_id: Option<usize>,
    pub swdio_id: Vec<usize>,
    pub swdio_grp_id: Option<usize>,
    pub swdio_grp: IndexMap::<usize, Vec<usize>>,
    pub trn: u32,
}

#[allow(non_snake_case)]
impl Service {
    pub fn new(dut: &Dut, id: usize, swdclk: Option<&str>, swdio: Option<&str>) -> Result<Self> {
        let swdclk_name = swdclk.unwrap_or("swdclk");
        let swdio_name = swdio.unwrap_or("swdio");
        let (swdio_id, swdio_grp_id) = dut.pin_group_to_ids(0, swdio_name)?;
        let (swdclk_id, swdclk_grp_id) = dut.pin_group_to_ids(0, swdclk_name)?;
        let mut swd_grp = IndexMap::new();
        swd_grp.insert(swdio_grp_id, swdio_id.clone());

        Ok(Self {
            id: id,
            swdclk: PinBus::from_group(dut, "swdclk", 0)?,
            swdio: PinBus::from_group(dut, "swdio", 0)?,
            swdclk_id: swdclk_id,
            swdclk_grp_id: Some(swdclk_grp_id),
            swdio_id: swdio_id,
            swdio_grp_id: Some(swdio_grp_id),
            swdio_grp: swd_grp,
            trn: 1,
        })
    }

    pub fn update_actions(&self, dut: &crate::Dut) -> Result<()> {
        self.swdclk.update_actions(dut)?;
        self.swdio.update_actions(dut)
    }

    pub fn write_ap(&self, dut: &crate::Dut, transaction: Transaction, ack: Acknowledgements) -> Result<()> {
        let mut trans = node!(
            SWDWriteAP,
            self.id,
            transaction.clone(),
            ack,
            None
        );
        let n_id = TEST.push_and_open(trans.clone());
        self.process_transaction(dut, &mut trans)?;
        TEST.close(n_id)?;
        Ok(())
    }

    pub fn verify_ap(&self, dut: &crate::Dut, transaction: Transaction, ack: Acknowledgements, parity: Option<bool>) -> Result<()> {
        let mut trans = node!(
            SWDVerifyAP,
            self.id,
            transaction.clone(),
            ack,
            parity,
            None
        );
        let n_id = TEST.push_and_open(trans.clone());
        self.process_transaction(dut, &mut trans)?;
        TEST.close(n_id)?;
        Ok(())
    }

    pub fn write_dp(&self, dut: &crate::Dut, transaction: Transaction, ack: Acknowledgements) -> Result<()> {
        let mut trans = node!(
            SWDWriteDP,
            self.id,
            transaction.clone(),
            ack,
            None
        );
        let n_id = TEST.push_and_open(trans.clone());
        self.process_transaction(dut, &mut trans)?;
        TEST.close(n_id)?;
        Ok(())
    }

    pub fn verify_dp(&self, dut: &crate::Dut, transaction: Transaction, ack: Acknowledgements, parity: Option<bool>) -> Result<()> {
        let mut trans = node!(
            SWDVerifyDP,
            self.id,
            transaction.clone(),
            ack,
            parity,
            None
        );
        let n_id = TEST.push_and_open(trans.clone());
        self.process_transaction(dut, &mut trans)?;
        TEST.close(n_id)?;
        Ok(())
    }
}