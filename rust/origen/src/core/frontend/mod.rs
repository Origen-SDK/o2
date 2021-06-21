use crate::{Metadata, Result};

pub mod callbacks;
use indexmap::IndexMap;
use std::collections::HashMap;
use std::path::Path;

pub fn with_frontend_app<T, F>(mut func: F) -> Result<T>
where
    F: FnMut(&dyn App) -> Result<T>,
{
    let handle = crate::FRONTEND.read().unwrap();
    match handle.frontend() {
        Some(fe) => match fe.app()? {
            Some(app) => func(app.as_ref()),
            None => error!("No application is currently available!"),
        },
        None => error!("No frontend is currently available!"),
    }
}

pub fn emit_callback(
    callback: &str,
    args: Option<Vec<Metadata>>,
    kwargs: Option<IndexMap<String, Metadata>>,
    opts: Option<HashMap<String, Metadata>>,
) -> Result<Vec<Metadata>> {
    with_frontend(|f| f.emit_callback(callback, args.as_ref(), kwargs.as_ref(), opts.as_ref()))
}

pub fn with_frontend<T, F>(mut func: F) -> Result<T>
where
    F: FnMut(&dyn Frontend) -> Result<T>,
{
    let handle = crate::FRONTEND.read().unwrap();
    handle.with_frontend(|f| func(f))
}

pub fn with_optional_frontend<T, F>(mut func: F) -> Result<Option<T>>
where
    F: FnMut(&dyn Frontend) -> Result<T>,
{
    let handle = crate::FRONTEND.read().unwrap();
    handle.with_optional_frontend(|f| func(f))
}

pub struct Handle {
    frontend: Option<Box<dyn Frontend + std::marker::Sync + std::marker::Send>>,
}

impl Handle {
    pub fn new() -> Self {
        Self { frontend: None }
    }

    pub fn frontend(&self) -> Option<&dyn Frontend> {
        // Ok(&*(*self.frontend.as_ref().unwrap()))
        match self.frontend.as_ref() {
            Some(f) => Some(f.as_ref()),
            None => None,
        }
    }

    pub fn set_frontend(
        &mut self,
        frontend: Box<dyn Frontend + std::marker::Sync + std::marker::Send>,
    ) -> Result<()> {
        callbacks::register_callbacks(frontend.as_ref())?;
        self.frontend = Some(frontend);
        Ok(())
    }

    pub fn with_frontend<T, F>(&self, mut func: F) -> Result<T>
    where
        F: FnMut(&dyn Frontend) -> Result<T>,
    {
        match self.frontend.as_ref() {
            Some(f) => func(f.as_ref()),
            None => error!("No frontend is currently available!"),
        }
    }

    pub fn with_optional_frontend<T, F>(&self, mut func: F) -> Result<Option<T>>
    where
        F: FnMut(&dyn Frontend) -> Result<T>,
    {
        Ok(match self.frontend.as_ref() {
            Some(f) => Some(func(f.as_ref())?),
            None => None,
        })
    }
}

pub trait Frontend {
    fn app(&self) -> Result<Option<Box<dyn App>>>;
    fn emit_callback(
        &self,
        callback: &str,
        args: Option<&Vec<Metadata>>,
        kwargs: Option<&IndexMap<String, Metadata>>,
        opts: Option<&HashMap<String, Metadata>>,
    ) -> Result<Vec<Metadata>>;
    fn register_callback(&self, callback: &str, description: &str) -> Result<()>;
    fn list_local_dependencies(&self) -> Result<Vec<String>>;
    fn on_dut_change(&self) -> Result<()>;
}

pub trait App {
    fn rc(&self) -> Result<Option<&dyn RC>>;
    fn unit_tester(&self) -> Result<Option<&dyn UnitTester>>;
    fn publisher(&self) -> Result<Option<&dyn Publisher>>;
    fn linter(&self) -> Result<Option<&dyn Linter>>;
    fn website(&self) -> Result<Option<&dyn Website>>;

    fn get_rc(&self) -> Result<&dyn RC> {
        match self.rc()? {
            Some(rc) => Ok(rc),
            None => error!("No RC is available on the application!"),
        }
    }

    fn get_unit_tester(&self) -> Result<&dyn UnitTester> {
        match self.unit_tester()? {
            Some(ut) => Ok(ut),
            None => error!("No unit tester is available on the application!"),
        }
    }

    fn get_publisher(&self) -> Result<&dyn Publisher> {
        match self.publisher()? {
            Some(pb) => Ok(pb),
            None => error!("No publisher is available on the application!")
        }
    }

    fn get_linter(&self) -> Result<&dyn Linter> {
        match self.linter()? {
            Some(l) => Ok(l),
            None => error!("No linter is available on the application!")
        }
    }

    fn get_website(&self) -> Result<&dyn Website> {
        match self.website()? {
            Some(w) => Ok(w),
            None => error!("No website is available on the application!")
        }
    }

    // fn setup_production_status_checks(&self) -> Result<()>;
    // fn cleanup_production_status_checks(&self) -> Result<()>;
    // fn production_status_checks_pre(&self, stop_at_first_fail: bool) -> Result<IndexMap<String, (bool, String)>>;
    // fn production_status_checks_post(&self, stop_at_first_fail: bool) -> Result<IndexMap<String, (bool, String)>>;

    fn check_production_status(&self) -> Result<bool>;
    fn publish(&self) -> Result<()>;
}

pub trait Linter {}

pub trait UnitTester {
    fn run(&self) -> Result<UnitTestStatus>;
}

pub trait RC {
    fn is_modified(&self) -> Result<bool>;
    fn status(&self) -> Result<crate::revision_control::Status>;
    fn checkin(&self, files_or_dirs: Option<Vec<&Path>>, msg: &str) -> Result<String>;
    fn tag(&self, tag: &str, force: bool, msg: Option<&str>) -> Result<()>;
}

pub trait Publisher {
    fn build_package(&self) -> Result<BuildResult>;
    fn upload(&self, build: &BuildResult, dry_run: bool) -> Result<UploadResult>;

    fn build_and_upload(&self, dry_run: bool) -> Result<(BuildResult, UploadResult)> {
        let br = self.build_package()?;
        Ok((br.clone(), self.upload(&br, dry_run)?))
    }
}

pub trait Website {
    fn build(&self) -> Result<BuildResult>;
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

#[derive(Debug, Clone)]
pub struct BuildResult {
    pub succeeded: bool,
    pub build_contents: Option<Vec<String>>,
    pub message: Option<String>,
    pub metadata: Option<IndexMap<String, Metadata>>
}

impl BuildResult {}

#[derive(Debug, Clone)]
pub struct UploadResult {
    pub succeeded: bool,
    pub message: Option<String>,
    pub metadata: Option<IndexMap<String, Metadata>>
}

#[derive(Debug, Clone)]
pub struct GenericResult {
    pub succeeded: bool,
    pub message: Option<String>,
    pub metadata: Option<IndexMap<String, Metadata>>
}

impl GenericResult {
    pub fn new(succeeded: bool, message: Option<String>, metadata: Option<IndexMap<String, Metadata>>) -> Self {
        Self {
            succeeded: succeeded,
            message: message,
            metadata: metadata
        }
    }

    pub fn new_with_empty_metadata(succeeded: bool, message: Option<String>) -> Self {
        Self::new(succeeded, message, Some(IndexMap::new()))
    }
}