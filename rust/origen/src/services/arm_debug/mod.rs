pub mod dp;
pub mod jtag_dp;
pub mod mem_ap;

pub use dp::DP;
pub use jtag_dp::JtagDP;
pub use mem_ap::MemAP;

use super::super::services::Service;
use crate::core::model::pins::PinCollection;
use crate::testers::api::ControllerAPI;
use crate::{Dut, Services, Transaction};
use std::collections::HashMap;
use std::sync::MutexGuard;
use std::sync::RwLock;

use crate::generator::PAT;
use crate::Result;

impl ControllerAPI for ArmDebug {
    fn name(&self) -> String {
        "ArmDebug".to_string()
    }
}

#[derive(Debug, Serialize)]
pub struct ArmDebug {
    pub id: usize,

    /// Model ID, holding all the registers and current values
    pub model_id: usize,

    /// IDs of any MemAPs this ArmDebug instance contains.
    mem_ap_ids: HashMap<usize, usize>,
    dp_id: Option<usize>,
    jtag_dp_id: Option<usize>,

    // Arm debug only support JTAG or SWD.
    // Store this just as a bool:
    //  true -> JTAG
    //  false -> SWD
    pub jtagnswd: RwLock<bool>,

    /// The SWD Service which operates this ArmDebug
    pub swd_id: Option<usize>,
    /// The JTAG Service which operates this ArmDebug
    pub jtag_id: Option<usize>,
}

impl ArmDebug {
    pub fn model_init(
        _dut: &mut crate::Dut,
        services: &mut crate::Services,
        model_id: usize,
        swd_id: Option<usize>,
        jtag_id: Option<usize>,
    ) -> Result<usize> {
        if swd_id.is_none() && jtag_id.is_none() {
            bail!("ArmDebug must be instantiated with a SWD and/or JTAG interface. Neither was provided.");
        }
        let id = services.next_id();
        let s = Self {
            id: id,
            dp_id: None,
            jtag_dp_id: None,
            mem_ap_ids: HashMap::new(),
            model_id: model_id,
            jtagnswd: RwLock::new(true),
            swd_id: swd_id,
            jtag_id: jtag_id,
        };
        services.push_service(Service::ArmDebug(s));
        Ok(id)
    }

    pub fn switch_to_swd(
        &self,
        dut: &MutexGuard<Dut>,
        services: &MutexGuard<Services>,
    ) -> Result<()> {
        match self.swd_id {
            Some(id) => {
                let swd = services.get_as_swd(id)?;
                let swdclk = PinCollection::from_group(dut, &swd.swdclk.0, swd.swdclk.1)?;
                let swdio = PinCollection::from_group(dut, &swd.swdio.0, swd.swdio.1)?;
                let n_id = crate::TEST.push_and_open(node!(PAT::ArmDebugSwjJTAGToSWD, self.id));
                self.comment("Switching ArmDebug protocol to SWD");
                swdclk.drive_high();
                swdio.drive_high().repeat(50);
                swdio.push_transaction(&Transaction::new_write(
                    num_bigint::BigUint::from(0xE79E as u32),
                    16,
                )?)?;
                swdio.repeat(55);
                swdio.drive_low().repeat(4);

                crate::TEST.close(n_id)?;
                *self.jtagnswd.write().unwrap() = false;
                Ok(())
            }
            None => Err(error!("No SWD available - cannot switch to SWD")),
        }
    }

    pub fn set_dp_id(&mut self, dp_id: usize) -> Result<()> {
        self.dp_id = Some(dp_id);
        Ok(())
    }

    pub fn dp_id(&self) -> Result<usize> {
        if let Some(id) = self.dp_id {
            Ok(id)
        } else {
            Err(error!(
                "Arm Debug instance at {} has not had a DP ID set yet.",
                self.id
            ))
        }
    }

    pub fn set_jtag_dp_id(&mut self, jtag_dp_id: usize) -> Result<()> {
        self.jtag_dp_id = Some(jtag_dp_id);
        Ok(())
    }

    pub fn jtag_dp_id(&self) -> Result<usize> {
        if let Some(id) = self.jtag_dp_id {
            Ok(id)
        } else {
            Err(error!(
                "Arm Debug instance at {} has not had a JTAG DP ID set yet.",
                self.id
            ))
        }
    }
}
