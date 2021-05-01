use crate::Result;

pub mod callbacks;
use indexmap::IndexMap;
use std::path::{Path};

pub fn with_frontend_app<T, F>(mut func: F) -> Result<T>
where
    F: FnMut(&dyn App) -> Result<T>,
{
    let handle = crate::FRONTEND.read().unwrap();
    match handle.frontend() {
        Some(fe) => {
            match fe.app()? {
                Some(app) => func(app.as_ref()),
                None => error!("No application is currently available!"),
            }
        },
        None => error!("No frontend is currently available!")
    }
}


pub struct Handle {
    // callbacks: IndexMap<String, Callback>,
    frontend: Option<Box<dyn Frontend + std::marker::Sync + std::marker::Send>>,
    callbacks: IndexMap<String, callbacks::Callback>
}

impl Handle {
    pub fn new() -> Self {
        Self {
            frontend: None,
            callbacks: IndexMap::new(),
        }
    }

    pub fn frontend(&self) -> Option<&dyn Frontend> {
        // Ok(&*(*self.frontend.as_ref().unwrap()))
        match self.frontend.as_ref() {
            Some(f) => Some(f.as_ref()),
            None => None
        }
    }

    pub fn set_frontend(&mut self, frontend: Box<dyn Frontend + std::marker::Sync + std::marker::Send>) -> Result<()> {
        self.frontend = Some(frontend);
        Ok(())
    }
}

pub trait Frontend {
    fn app(&self) -> Result<Option<Box<dyn App>>>;
}

pub trait App {
    fn rc(&self) -> Result<Option<&dyn RC>>;
    fn unit_tester(&self) -> Result<Option<&dyn UnitTester>>;

    fn get_rc(&self) -> Result<&dyn RC> {
        match self.rc()? {
            Some(rc) => Ok(rc),
            None => error!("No RC is available on the application!")
        }
    }

    fn get_unit_tester(&self) -> Result<&dyn UnitTester> {
        match self.unit_tester()? {
            Some(ut) => Ok(ut),
            None => error!("No unit tester is available on the application!")
        }
    }

    // fn lint(&self) -> Result<Box<dyn Linter>>;
    // fn package(&self) -> Result<Box<dyn Package>>;
    // fn website(&self) -> Result<Box<dyn Website>>;
    fn check_production_status(&self) -> Result<bool>;
    fn publish(&self) -> Result<()>;

}

pub trait Linter {
}

pub trait UnitTester {
    fn run(&self) -> Result<UnitTestStatus>;
}

pub trait RC {
    fn is_modified(&self) -> Result<bool>;
    fn status(&self) -> Result<crate::revision_control::Status>;
    fn checkin(&self, files_or_dirs: Option<Vec<&Path>>, msg: &str) -> Result<String>;
    fn tag(&self, tag: &str, force: bool, msg: Option<&str>) -> Result<()>;
}

pub trait Package {
    fn build(&self) -> Result<()>;
}

pub trait Website {
    fn build(&self) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct UnitTestStatus {
    // tests: Vec<TestResult>,
    pub passed: Option<bool>,
    pub text: Option<String>,
}

impl UnitTestStatus {
    pub fn passed(&self) -> bool {
        match self.passed {
            Some(p) => p,
            None => {
                // for t in self.tests {
                //     if t.failed {
                //         self.passed = Some(false);
                //         return false
                //     }
                // }
                // self.passed = Some(true);
                true
            }
        }
    }

//     fn non_empty_and_passed(&self) -> bool {
//         match self.passed {
//             Some(p) => p,
//             None => {
//                 if tests.is_empty() {
//                     self.passed = false;
//                     false
//                 } else {
//                     self.passed()
//                 }
//             }
//         }
//     }

//     fn tests(&self) -> &Vec<TestResult> {
//         &self.tests
//     }
}