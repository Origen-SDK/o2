#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde;
extern crate meta;
#[macro_use]
extern crate pest_derive;
#[macro_use]
pub mod macros;
#[macro_use]
extern crate indexmap;

pub mod core;
pub mod error;
pub mod generator;
pub mod prog_gen;
pub mod revision_control;
pub mod services;
pub mod standards;
pub mod testers;
pub mod utility;
pub mod precludes;

pub use self::core::user::User;
pub use error::Error;
pub use self::core::metadata::Metadata;
pub use self::generator::utility::transaction::Transaction;
pub use self::generator::utility::transaction::Action as TransactionAction;

use self::core::application::Application;
use self::core::config::Config as OrigenConfig;
pub use self::core::dut::Dut;
use self::core::model::registers::BitCollection;
pub use self::core::producer::Producer;
use self::core::status::Status;
pub use self::core::tester::Tester;
use self::generator::ast::*;
pub use self::services::Services;
use self::utility::logger::Logger;
use num_bigint::BigUint;
use prog_gen::Interface;
use std::fmt;
use std::sync::{Mutex, MutexGuard};

pub type Result<T> = std::result::Result<T, Error>;

/// The available Origen runtime modes
pub const MODES: &'static [&'static str] = &["production", "development"];

// No idea why, but lazy_static was having none of this
// pub static BIGU1: num_bigint::BigUint = num_bigint::BigUint::from(1 as u8);
// pub static BIGU0: num_bigint::BigUint = num_bigint::BigUint::from(0 as u8);

lazy_static! {
    /// Provides status information derived from the runtime environment, e.g. if an app is present
    /// If an app is present then its Application struct is stored in here.
    /// Things like the current output and reference directories should be derived from here.
    pub static ref STATUS: Status = Status::default();
    /// Provides configuration information derived from origen.toml files found in the Origen
    /// installation and application file system paths
    pub static ref ORIGEN_CONFIG: OrigenConfig = OrigenConfig::default();
    pub static ref LOGGER: Logger = Logger::default();
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
    /// Storage for the current program generation run, can include multiple flows
    pub static ref INTERFACE: Interface = Interface::new();
    /// Provides info about the current user
    pub static ref USER: User = User::current();
}

impl PartialEq<AST> for TEST {
    fn eq(&self, ast: &AST) -> bool {
        self.to_node() == ast.to_node()
    }
}

impl PartialEq<Node> for TEST {
    fn eq(&self, node: &Node) -> bool {
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

impl <'a> Value<'a> {
    pub fn to_write_transaction(&self, dut: &MutexGuard<Dut>) -> Result<Transaction> {
        match &self {
            Self::Bits(bits, _size) => {
                bits.to_write_transaction(dut)
            },
            Self::Data(data, width) => {
                Transaction::new_write(data.clone(), (*width) as usize)
            }
        }
    }

    pub fn to_verify_transaction(&self, dut: &MutexGuard<Dut>) -> Result<Transaction> {
        match &self {
            Self::Bits(bits, _size) => {
                bits.to_verify_transaction(None, true, dut)
            },
            Self::Data(data, width) => {
                Transaction::new_verify(data.clone(), (*width) as usize)
            }
        }
    }
}

/// This is called immediately upon Origen booting
pub fn initialize(verbosity: Option<u8>) {
    if let Some(v) = verbosity {
        let _ = LOGGER.set_verbosity(v);
    }
    // Always keep this, as it is a way of forcing the STATUS object to be instantiated
    log_debug!("Initialized Origen {}", STATUS.origen_version);
    LOGGER.set_status_ready();
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
        None => error!("Something has gone wrong, a reference has been made to the current job when there is none"),
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
        None => error!("Something has gone wrong, a reference has been made to the current job when there is none"),
        Some(j) => func(j),
    }
}

pub fn services() -> MutexGuard<'static, Services> {
    SERVICES.lock().unwrap()
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
pub fn prepare_for_target_load() {
    tester().reset();
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
