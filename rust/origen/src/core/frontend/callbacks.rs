use crate::Result;
use indexmap::IndexMap;

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
