use super::get_pyapp;
use crate::utility::linter::_frontend::Linter;
use crate::utility::publisher::_frontend::Publisher;
use crate::utility::release_scribe::_frontend::ReleaseScribe;
use crate::utility::unit_testers::_frontend::UnitTester;
use crate::utility::website::_frontend::Website;
use pyapi_metal::prelude::frontend::*;
use pyo3::prelude::*;

pub struct App {
    rc: RevisionControlFrontend,
    unit_tester: UnitTester,
    publisher: Publisher,
    linter: Linter,
    website: Website,
    release_scribe: ReleaseScribe,
}

impl App {
    pub fn new() -> origen::Result<Self> {
        Ok(Self {
            rc: RevisionControlFrontend {},
            unit_tester: UnitTester {},
            publisher: Publisher {},
            linter: Linter {},
            website: Website {},
            release_scribe: ReleaseScribe {},
        })
    }
}

impl origen::core::frontend::App for App {
    fn check_production_status(&self) -> origen::Result<bool> {
        Python::with_gil(|py| {
            let pyapp = get_pyapp(py)?;
            let app = pyapp.borrow(py);
            Ok(app.check_production_status()?)
        })
    }

    fn publish(&self) -> origen::Result<()> {
        todo!()
    }

    fn rc(&self) -> origen::Result<Option<&dyn RevisionControlFrontendAPI>> {
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

    // TODO
    // fn mailer(&self) -> origen::Result<Option<&dyn origen::core::frontend::Mailer>> {
    //     todo!()
    // }

    fn release_scribe(&self) -> origen::Result<Option<&dyn origen::core::frontend::ReleaseScribe>> {
        Ok(Some(&self.release_scribe))
    }
}
