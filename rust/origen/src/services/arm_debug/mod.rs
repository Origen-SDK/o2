pub mod dp;
pub mod mem_ap;

pub use dp::DP;
pub use mem_ap::MemAP;

use std::collections::HashMap;
use crate::testers::vector_based::api::*;
use super::super::services::Service;

// Register descriptions taken from the Arm Debug RM
use crate::{Result, Error};

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ArmDebug {
    pub id: usize,

    /// Model ID, holding all the registers and current values
    pub model_id: usize,
    
    /// IDs of any MemAPs this ArmDebug instance contains.
    mem_ap_ids: HashMap<usize, usize>,
    dp_id: Option<usize>,

    // Arm debug only support JTAG or SWD.
    // Store this just as a bool:
    //  true -> JTAG
    //  false -> SWD
    pub jtagnswd: bool,

    /// The SWD Service which operates this ArmDebug
    pub swd_id: Option<usize>,
    /// The JTAG Service which operates this ArmDebug
    pub jtag_id: Option<usize>,

    swdclk_id: Option<Vec<usize>>,
    swdclk_grp_id: Option<Option<usize>>,
    swdio_id: Option<Vec<usize>>,
    swdio_grp_id: Option<Option<usize>>,
}

macro_rules! swdclk {
    ($slf:expr, $action:ident) => {{
        crate::testers::vector_based::api::$action(
            $slf.swdclk_id.as_ref().unwrap(), //.expect("SWD CLK pin has not been set or could not be found!"),
            $slf.swdclk_grp_id.unwrap() // .expect("SWD CLK pin has not been set or could not be found!")
        )
    }};
}

macro_rules! swdio {
    ($slf:expr, $action:ident) => {{
        crate::testers::vector_based::api::$action(
            $slf.swdio_id.as_ref().unwrap(), //.expect("SWD CLK pin has not been set or could not be found!"),
            $slf.swdio_grp_id.unwrap() // .expect("SWD CLK pin has not been set or could not be found!")
        )
    }};
}

macro_rules! extract_as_swd {
    ($swd_id:expr, $services:expr) => {{
        if let Some(id) = $swd_id {
            let s = $services.get_service(id)?;
            match s {
                crate::services::Service::SWD(_s) => Ok(_s),
                _ => Err(crate::Error::new(&format!(
                    "Tried to extract Service at {} as SWD but received {:?}",
                    id,
                    s
                )))
            }
        } else {
            Err(crate::Error::new(&format!("SWD is not available (swd_id is None)!")))
        }
    }};
}

impl ArmDebug {
    pub fn model_init(
        _dut: &mut crate::Dut,
        services: &mut crate::Services,
        model_id: usize,
        swd_id: Option<usize>,
        jtag_id: Option<usize>,
    ) -> Result<usize> {
        let id = services.next_id();
        let mut swdclk_id = None;
        let mut swdclk_grp_id = None;
        let mut swdio_id = None;
        let mut swdio_grp_id = None;
        if swd_id.is_some() {
            let swd = extract_as_swd!(swd_id, services)?;
            swdclk_id = Some(swd.swdclk_id.clone());
            swdclk_grp_id = Some(swd.swdclk_grp_id.clone());
            swdio_id = Some(swd.swdio_id.clone());
            swdio_grp_id = Some(swd.swdio_grp_id.clone());
        }
        let s = Self {
            id: id,
            dp_id: None,
            mem_ap_ids: HashMap::new(),
            model_id: model_id,
            jtagnswd: true,
            swd_id: swd_id,
            swdclk_id: swdclk_id,
            swdclk_grp_id: swdclk_grp_id,
            swdio_id: swdio_id,
            swdio_grp_id: swdio_grp_id,
            jtag_id: jtag_id,
        };
        services.push_service(Service::ArmDebug(s));
        Ok(id)
    }

    pub fn switch_to_swd(&mut self) -> Result<()> {
        match self.swd_id {
            Some(_id) => {
                // crate::TEST.push(crate::node!(ArmDebugSwjJTAGToSWD, self.clone()));
                let n_id = crate::TEST.push_and_open(crate::node!(ArmDebugSwjJTAGToSWD, self.id));
                let mut i: indexmap::IndexMap<usize, Vec<usize>> = indexmap::IndexMap::new();
                i.insert(self.swdio_grp_id.unwrap().unwrap(), self.swdio_id.as_ref().unwrap().clone());

                // let mut nodes: Vec<Node> = vec!();
                crate::TEST.append(&mut swdclk!(self, drive_high)?);
                // crate::TEST.append(&mut drive_high(swdclk)?);
                crate::TEST.append(&mut swdio!(self, drive_high)?);
                crate::TEST.push(repeat(50, true)?);
                crate::TEST.append(&mut drive_data(
                    &i,
                    &num::BigUint::from(0xE79E as u32),
                    16,
                    true,
                    None::<&u8>,
                    None
                )?);
                crate::TEST.push(repeat(55, true)?);
                crate::TEST.push(comment("Move to IDLE")?);
                // crate::TEST.push(set_pin_drive_low(&swdio)?);
                crate::TEST.append(&mut swdio!(self, drive_low)?);
                crate::TEST.push(repeat(4, true)?);
                crate::TEST.close(n_id)?;
                self.jtagnswd = false;
                Ok(())
            },
            None => {
                Err(Error::new(&format!("No SWD available - cannot switch to SWD")))
            }
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
            Err(Error::new(&format!(
                "Arm Debug instance at {} has not had a DP ID set yet.",
                self.id
            )))
        }
    }

    // pub fn dp(&self) -> Result<&>

    // /// Discerns how the register should be written (AP vs. DP) and adds the appropriate nodes
    // pub fn write_register(&self, dut: &MutexGuard<Dut>, bc: &BitCollection) -> Result<()> {
    //     // ...
    //     Ok(())
    // }

    // pub fn model(&self) -> &crate::core::model::Model {
    //     // ...
    // }

    // pub fn protocol(&self) -> Service {
    //     // ...
    // }
}
