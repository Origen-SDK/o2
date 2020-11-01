pub mod arm_debug;
pub mod jtag;
pub mod swd;

pub use arm_debug::ArmDebug;

use crate::{Error, Result};

#[derive(Debug)]
pub enum Service {
    JTAG(jtag::Service),
    SWD(swd::Service),
    ArmDebug(arm_debug::ArmDebug),
    ArmDebugDP(arm_debug::dp::DP),
    ArmDebugJtagDP(arm_debug::jtag_dp::JtagDP),
    ArmDebugMemAP(arm_debug::mem_ap::MemAP),
}

impl Service {
    pub fn as_swd(&self) -> Result<&swd::Service> {
        match self {
            Self::SWD(s) => Ok(s),
            _ => Err(Error::new(&format!(
                "Expected service SWD but received {:?}",
                self
            ))),
        }
    }

    pub fn as_jtag(&self) -> Result<&jtag::Service> {
        match self {
            Self::JTAG(s) => Ok(s),
            _ => Err(Error::new(&format!(
                "Expected service JTAG but received {:?}",
                self
            ))),
        }
    }

    pub fn as_arm_debug(&self) -> Result<&arm_debug::ArmDebug> {
        match self {
            Self::ArmDebug(s) => Ok(s),
            _ => Err(Error::new(&format!(
                "Expected service ArmDebug but received {:?}",
                self
            ))),
        }
    }

    pub fn as_mut_arm_debug(&mut self) -> Result<&mut arm_debug::ArmDebug> {
        match self {
            Self::ArmDebug(s) => Ok(s),
            _ => Err(Error::new(&format!(
                "Expected service ArmDebug but received {:?}",
                self
            ))),
        }
    }

    pub fn as_dp(&self) -> Result<&arm_debug::DP> {
        match self {
            Self::ArmDebugDP(s) => Ok(s),
            _ => Err(Error::new(&format!(
                "Expected service ArmDebugDP but received {:?}",
                self
            ))),
        }
    }

    pub fn as_mut_dp(&mut self) -> Result<&mut arm_debug::DP> {
        match self {
            Self::ArmDebugDP(s) => Ok(s),
            _ => Err(Error::new(&format!(
                "Expected service ArmDebugDP but received {:?}",
                self
            ))),
        }
    }

    pub fn as_jtag_dp(&self) -> Result<&arm_debug::JtagDP> {
        match self {
            Self::ArmDebugJtagDP(s) => Ok(s),
            _ => Err(Error::new(&format!(
                "Expected service ArmDebugJtagDP but received {:?}",
                self
            ))),
        }
    }

    pub fn as_mut_jtag_dp(&mut self) -> Result<&mut arm_debug::JtagDP> {
        match self {
            Self::ArmDebugJtagDP(s) => Ok(s),
            _ => Err(Error::new(&format!(
                "Expected service ArmDebugJtagDP but received {:?}",
                self
            ))),
        }
    }

    pub fn as_mem_ap(&self) -> Result<&arm_debug::MemAP> {
        match self {
            Self::ArmDebugMemAP(s) => Ok(s),
            _ => Err(Error::new(&format!(
                "Expected service ArmDebugMemAP but received {:?}",
                self
            ))),
        }
    }
}

pub struct Services {
    services: Vec<Service>,
}

impl Services {
    // This is called only once at the start of an Origen thread to create the global database,
    // then the 'change' method is called every time a new DUT is loaded
    pub fn new() -> Services {
        Services {
            services: Vec::<Service>::new(),
        }
    }

    // Called when the DUT is changed
    pub fn change(&mut self) {
        self.services.clear();
    }

    pub fn next_id(&self) -> usize {
        self.services.len()
    }

    /// Adds the given subblock to the database, returning its assigned ID
    pub fn push_service(&mut self, service: Service) -> usize {
        self.services.push(service);
        self.services.len()
    }

    /// Adds the given service to the database, returning its assigned ID
    pub fn add_service(&mut self, service: Service) -> usize {
        let id;
        {
            id = self.services.len();
        }
        self.services.push(service);
        id
    }

    /// Get a mutable reference to the service with the given ID
    pub fn get_mut_service(&mut self, id: usize) -> Result<&mut Service> {
        match self.services.get_mut(id) {
            Some(x) => Ok(x),
            None => {
                return Err(Error::new(&format!(
                    "Something has gone wrong, no service exists with ID '{}'",
                    id
                )))
            }
        }
    }

    /// Get a read-only reference to the service with the given ID, use get_mut_service if
    /// you need to modify it
    pub fn get_service(&self, id: usize) -> Result<&Service> {
        match self.services.get(id) {
            Some(x) => Ok(x),
            None => {
                return Err(Error::new(&format!(
                    "Something has gone wrong, no service exists with ID '{}'",
                    id
                )))
            }
        }
    }

    pub fn get_as_swd(&self, id: usize) -> Result<&swd::Service> {
        let s = self.get_service(id)?;
        s.as_swd()
    }

    pub fn get_as_jtag(&self, id: usize) -> Result<&jtag::Service> {
        let s = self.get_service(id)?;
        s.as_jtag()
    }

    pub fn get_as_arm_debug(&self, id: usize) -> Result<&arm_debug::ArmDebug> {
        let s = self.get_service(id)?;
        s.as_arm_debug()
    }

    pub fn get_as_dp(&self, id: usize) -> Result<&arm_debug::DP> {
        let s = self.get_service(id)?;
        s.as_dp()
    }

    pub fn get_as_mut_dp(&mut self, id: usize) -> Result<&mut arm_debug::DP> {
        let s = self.get_mut_service(id)?;
        s.as_mut_dp()
    }

    pub fn get_as_jtag_dp(&self, id: usize) -> Result<&arm_debug::JtagDP> {
        let s = self.get_service(id)?;
        s.as_jtag_dp()
    }

    pub fn get_as_mut_jtag_dp(&mut self, id: usize) -> Result<&mut arm_debug::JtagDP> {
        let s = self.get_mut_service(id)?;
        s.as_mut_jtag_dp()
    }

    pub fn get_as_mem_ap(&self, id: usize) -> Result<&arm_debug::MemAP> {
        let s = self.get_service(id)?;
        s.as_mem_ap()
    }

    pub fn get_as_mut_arm_debug(&mut self, id: usize) -> Result<&mut arm_debug::ArmDebug> {
        let s = self.get_mut_service(id)?;
        match s {
            Service::ArmDebug(a) => Ok(a),
            _ => Err(Error::new(&format!(
                "Expected service ArmDebug but received {:?}",
                s
            ))),
        }
    }
}
