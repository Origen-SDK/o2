// use origen::core::frontend::App as OApp;

use super::get_pyapp;
use crate::utility::linter::_frontend::Linter;
use crate::utility::publisher::_frontend::Publisher;
use crate::utility::revision_control::_frontend::RC;
use crate::utility::unit_testers::_frontend::UnitTester;
use crate::utility::website::_frontend::Website;
use crate::utility::mailer::_frontend::Mailer;
use crate::utility::release_scribe::_frontend::ReleaseScribe;
use pyo3::prelude::*;

pub struct App {
    rc: RC,
    unit_tester: UnitTester,
    publisher: Publisher,
    linter: Linter,
    website: Website,
    mailer: Mailer,
    release_scribe: ReleaseScribe,
}

impl App {
    pub fn new() -> origen::Result<Self> {
        Ok(Self {
            rc: RC {},
            unit_tester: UnitTester {},
            publisher: Publisher {},
            linter: Linter {},
            website: Website {},
            mailer: Mailer {},
            release_scribe: ReleaseScribe {},
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

    fn publisher(&self) -> origen::Result<Option<&dyn origen::core::frontend::Publisher>> {
        Ok(Some(&self.publisher))
    }

    fn linter(&self) -> origen::Result<Option<&dyn origen::core::frontend::Linter>> {
        Ok(Some(&self.linter))
    }

    fn website(&self) -> origen::Result<Option<&dyn origen::core::frontend::Website>> {
        Ok(Some(&self.website))
    }

    fn mailer(&self) -> origen::Result<Option<&dyn origen::core::frontend::Mailer>> {
        Ok(Some(&self.mailer))
    }

    fn release_scribe(&self) -> origen::Result<Option<&dyn origen::core::frontend::ReleaseScribe>> {
        Ok(Some(&self.release_scribe))
    }
}
