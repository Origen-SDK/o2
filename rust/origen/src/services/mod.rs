pub mod jtag;

use crate::{Error, Result};

#[derive(Debug)]
pub enum Service {
    None, // Not used, but removes a compiler warning until we have more service types
    JTAG(jtag::Service),
}

impl Service {}

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
}
