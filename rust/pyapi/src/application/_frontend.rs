// use origen::core::frontend::App as OApp;

use pyo3::prelude::*;
use super::get_pyapp;
use crate::utility::revision_control::_frontend::RC;
use crate::utility::unit_testers::_frontend::UnitTester;

pub struct App {
    rc: RC,
    unit_tester: UnitTester
}

impl App {
    pub fn new() -> origen::Result<Self> {
        Ok(Self {
            rc: RC {},
            unit_tester: UnitTester {},
        })
    }
}

impl origen::core::frontend::App for App {
    fn check_production_status(&self) -> origen::Result<bool> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let pyapp = get_pyapp(py)?;
        let app = pyapp.borrow(py);
        Ok(app.check_production_status()?)
    }

    fn publish(&self) -> origen::Result<()> {
        todo!()
    }

    fn rc(&self) -> origen::Result<Option<&dyn origen::core::frontend::RC>> {
        Ok(Some(&self.rc))
    }

    fn unit_tester(&self) -> origen::Result<Option<&dyn origen::core::frontend::UnitTester>> {
        Ok(Some(&self.unit_tester))
    }
}
