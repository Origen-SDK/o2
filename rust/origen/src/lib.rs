#![allow(incomplete_features)]
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde;
extern crate origen_core_support;
#[macro_use]
pub extern crate origen_metal;
#[allow(unused_imports)]
#[macro_use]
extern crate indexmap;
#[macro_use]
extern crate strum_macros;

pub mod core;
pub mod generator;
pub mod precludes;
pub mod services;
pub mod standards;
pub mod testers;
pub mod utility;

pub use self::core::status::Operation;
pub use self::generator::utility::transaction::Action as TransactionAction;
pub use self::generator::utility::transaction::Transaction;
pub use origen_metal::Error;

use self::core::application::Application;
use self::core::config::Config as OrigenConfig;
use self::core::config::ConfigMetadata as OrigenConfigMetadata;
pub use self::core::dut::Dut;
use self::core::frontend::Handle;
use self::core::model::registers::BitCollection;
pub use self::core::producer::Producer;
use self::core::status::Status;
pub use self::core::status::{app_present, in_app_invocation, in_global_invocation};
pub use self::core::tester::{Capture, Overlay, Tester};
pub use self::services::Services;
pub use self::utility::sessions::{setup_sessions, with_app_session, with_app_session_group};
use num_bigint::BigUint;
pub use om::prelude::frontend::*;
pub use om::{TypedValue, LOGGER};
pub use origen_metal as om;
use std::fmt;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};
use std::sync::{RwLock, RwLockReadGuard};

use generator::PAT;
use origen_metal::ast::{Attrs, Node, AST};

pub use self::core::frontend::callbacks as CALLBACKS;
pub use self::core::frontend::{
    emit_callback, with_frontend, with_frontend_app, with_optional_frontend,
};

pub use origen_metal::Result;

/// The available Origen runtime modes
pub const MODES: &'static [&'static str] = &["production", "development"];

// No idea why, but lazy_static was having none of this
// pub static BIGU1: num_bigint::BigUint = num_bigint::BigUint::from(1 as u8);
// pub static BIGU0: num_bigint::BigUint = num_bigint::BigUint::from(0 as u8);

// TODO move this to somewhere else?
pub static FE_CAT_NAME__LDAPS: &'static str = "ldaps";

lazy_static! {
    /// Provides status information derived from the runtime environment, e.g. if an app is present
    /// If an app is present then its Application struct is stored in here.
    /// Things like the current output and reference directories should be derived from here.
    pub static ref STATUS: Status = Status::default();
    /// Provides configuration information derived from origen.toml files found in the Origen
    /// installation and application file system paths
    pub static ref ORIGEN_CONFIG: OrigenConfig = OrigenConfig::default();
    pub static ref ORIGEN_CONFIG_METADATA: RwLock<OrigenConfigMetadata> = RwLock::new(OrigenConfigMetadata::default());
    /// The current device model, containing all metadata about hierarchy, regs, pins, specs,
    /// timing, etc. and responsible for maintaining the current state of the DUT (regs, pins,
    /// etc.)
    pub static ref DUT: Mutex<Dut> = Mutex::new(Dut::new("placeholder"));
    /// The global tester model.
    pub static ref TESTER: Mutex<Tester> = Mutex::new(Tester::new());
    /// Producer
    pub static ref PRODUCER: Mutex<Producer> = Mutex::new(Producer::new());
    /// Services owned by the current DUT, stored as a separate collection to avoid having to
    /// get a mutable ref on the DUT if the service needs mutation
    pub static ref SERVICES: Mutex<Services> = Mutex::new(Services::new());
    /// Storage for the current test (pattern)
    pub static ref TEST: generator::TestManager = generator::TestManager::new();
    pub static ref FRONTEND: RwLock<Handle> = RwLock::new(Handle::new());
}

impl PartialEq<AST<PAT>> for TEST {
    fn eq(&self, ast: &AST<PAT>) -> bool {
        self.to_node() == ast.to_node()
    }
}

impl PartialEq<TEST> for AST<PAT> {
    fn eq(&self, test: &TEST) -> bool {
        self.to_node() == test.to_node()
    }
}

impl PartialEq<Node<PAT>> for TEST {
    fn eq(&self, node: &Node<PAT>) -> bool {
        self.to_node() == *node
    }
}

impl fmt::Debug for TEST {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_node())
    }
}

pub mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

pub enum Value<'a> {
    Bits(BitCollection<'a>, Option<u32>), // bits holding data, optional size
    Data(BigUint, u32),                   // value, size
}

impl<'a> Value<'a> {
    pub fn to_write_transaction(&self, dut: &MutexGuard<Dut>) -> Result<Transaction> {
        match &self {
            Self::Bits(bits, _size) => bits.to_write_transaction(dut),
            Self::Data(data, width) => Transaction::new_write(data.clone(), (*width) as usize),
        }
    }

    pub fn to_verify_transaction(&self, dut: &MutexGuard<Dut>) -> Result<Transaction> {
        match &self {
            Self::Bits(bits, _size) => bits.to_verify_transaction(None, true, dut),
            Self::Data(data, width) => Transaction::new_verify(data.clone(), (*width) as usize),
        }
    }

    pub fn to_capture_transaction(&self, dut: &MutexGuard<Dut>) -> Result<Transaction> {
        match &self {
            Self::Bits(bits, _size) => bits.to_capture_transaction(dut),
            Self::Data(_data, width) => Transaction::new_capture((*width) as usize, None),
        }
    }
}

/// This is called immediately upon Origen booting
pub fn initialize(
    verbosity: Option<u8>,
    verbosity_keywords: Vec<String>,
    cli_location: Option<String>,
    cli_version: Option<String>,
    fe_pkg_loc: Option<PathBuf>,
    fe_exe_loc: Option<PathBuf>,
) {
    if let Some(v) = verbosity {
        let _ = LOGGER.set_verbosity(v);
        let _ = LOGGER.set_verbosity_keywords(verbosity_keywords);
    }
    STATUS.set_cli_location(cli_location);
    STATUS.set_cli_version(cli_version);
    STATUS.set_fe_pkg_loc(fe_pkg_loc);
    STATUS.set_fe_exe_loc(fe_exe_loc);
    log_debug!("Initialized Origen {}", STATUS.origen_version);
    if let Some(app) = app() {
        origen_metal::PROG_GEN_CONFIG.set_app_name(app.name());
    }
    origen_metal::PROG_GEN_CONFIG.set_debug_enabled(crate::STATUS.is_debug_enabled());
}

pub fn app() -> Option<&'static Application> {
    STATUS.app.as_ref()
}

pub fn dut() -> MutexGuard<'static, Dut> {
    DUT.lock().unwrap()
}

pub fn tester() -> MutexGuard<'static, Tester> {
    TESTER.lock().unwrap()
}

pub fn producer() -> MutexGuard<'static, Producer> {
    PRODUCER.lock().unwrap()
}

/// Execute the given function with a reference to the current job.
/// Returns an error if there is no current job, otherwise the result of the given function.
pub fn with_current_job<T, F>(mut func: F) -> Result<T>
where
    F: FnMut(&core::producer::job::Job) -> Result<T>,
{
    match producer().current_job() {
        None => bail!("Something has gone wrong, a reference has been made to the current job when there is none"),
        Some(j) => func(j),
    }
}

/// Execute the given function with a mutable reference to the current job.
/// Returns an error if there is no current job, otherwise the result of the given function.
pub fn with_current_job_mut<T, F>(mut func: F) -> Result<T>
where
    F: FnMut(&mut core::producer::job::Job) -> Result<T>,
{
    match producer().current_job_mut() {
        None => bail!("Something has gone wrong, a reference has been made to the current job when there is none"),
        Some(j) => func(j),
    }
}

pub fn services() -> MutexGuard<'static, Services> {
    SERVICES.lock().unwrap()
}

pub fn origen_config_metadata<'a>() -> RwLockReadGuard<'a, OrigenConfigMetadata> {
    ORIGEN_CONFIG_METADATA.read().unwrap()
}

pub (crate) fn set_origen_config_metadata(new: OrigenConfigMetadata) {
    let mut m = ORIGEN_CONFIG_METADATA.write().unwrap();
    *m = new;
}

/// Sanitizes the given mode string and returns it, but will exit the process if it is invalid
pub fn clean_mode(name: &str) -> String {
    let mut matches: Vec<String> = Vec::new();

    for i in 0..MODES.len() {
        if MODES[i].contains(name) {
            matches.push(MODES[i].to_string());
        }
    }

    if matches.len() == 0 {
        println!(
            "No mode found matching '{}', here are the available modes:",
            name
        );
        for i in 0..MODES.len() {
            println!("    {}", MODES[i].to_string());
        }
    } else if matches.len() > 1 {
        println!(
            "'{}' is an ambiguous mode name, please try again to narrow it down to one of these:",
            name
        );
        for m in matches.iter() {
            println!("    {}", m.to_string());
        }
    } else {
        return matches[0].to_string();
    }
    std::process::exit(1);
}

/// This will be called immediately before loading a fresh set of targets. Everything
/// required to clear previous state from the existing targets should be initiated from here.
pub fn prepare_for_target_load() -> Result<()> {
    tester().reset()
}

/// Clears the current test (pattern) AST and starts a new one, this will be called by the
/// producer before loading the next pattern source file
pub fn start_new_test(name: Option<String>) {
    if let Some(name) = name {
        TEST.start(&name);
    } else {
        TEST.start("ad-hoc");
    }
}

pub fn trace_error<T: Attrs>(node: &Node<T>, error: Error) -> Result<()> {
    // Messaging may need to be slightly different for patgen
    if STATUS.operation() == Operation::GenerateFlow {
        let help = {
            let s = node.meta_string();
            if s != "" {
                s
            } else {
                if STATUS.is_debug_enabled() {
                    // Don't display children since it's potentially huge
                    let n = node.replace_children(vec![]);
                    format!("Sorry, no flow source information was found, here is the flow node that failed if it helps:\n{}", n)
                } else {
                    "Run again with the --debug switch to try and trace this back to a flow source file location".to_string()
                }
            }
        };
        bail!("{}\n{}", error, &help)
    } else {
        bail!("{}", error)
    }
}

// TODO change name?
#[cfg(all(test, not(origen_skip_frontend_tests)))]
mod tests {
    pub fn run_python(code: &str) -> crate::Result<()> {
        let mut c = std::process::Command::new("origen");
        c.arg("exec");
        c.arg("python");
        c.arg("-c");
        c.arg(&format!("import origen; {}", code));
        // Assume we're in the root of the Origen rust package
        let mut f = std::env::current_dir().unwrap();
        f.pop();
        f.pop();
        f.push("test_apps/python_app");
        c.current_dir(f);
        let res = c.output().unwrap();
        println!("status: {}", res.status);
        println!("{:?}", std::str::from_utf8(&res.stdout).unwrap());
        println!("{:?}", std::str::from_utf8(&res.stderr).unwrap());
        assert_eq!(res.status.success(), true);
        Ok(())
    }
}
