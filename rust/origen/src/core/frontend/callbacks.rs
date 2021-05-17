use crate::Result;
use indexmap::IndexMap;
use super::Frontend;

pub static BEFORE_TESTER_RESET: &str = "before_tester_reset";
pub static AFTER_TESTER_RESET: &str = "after_tester_reset";

pub fn register_callbacks(f: &dyn Frontend) -> Result<()> {
    f.register_callback(
        BEFORE_TESTER_RESET,
        "Emitted just before the global tester are reset. This would be the last chance do gleam anything from the global tester.",
    )?;
    f.register_callback(
        AFTER_TESTER_RESET,
        "Emitted just after the global tester is reset but before any targets are (re)loaded.",
    )?;
    Ok(())
}

pub trait FrontendHandler {
    fn emit(&self, callback: &str) -> Result<()>;
    fn register_new(&self, callback: &str) -> Result<()>;
}

pub struct Listener {
    // ...
}

pub struct Callback {
    // ...
}

pub struct Callbacks {
    callbacks: IndexMap<String, Callback>,
    frontend_handler: Option<Box<dyn FrontendHandler + std::marker::Send>>
}

impl Callbacks {
    pub fn emit(&self, callback: &str) -> Result<()> {
        self.frontend_handler.as_ref().unwrap().emit(callback)
    }

    pub fn new() -> Self {
        Self {
            callbacks: IndexMap::new(),
            frontend_handler: None
        }
    }

    pub fn initialize_frontend(&mut self, frontend_handler: Box<dyn FrontendHandler + std::marker::Send>) {
        self.frontend_handler = Some(frontend_handler);
        self.frontend_handler.as_ref().unwrap().register_new("logfiles_closed");
    }
}
