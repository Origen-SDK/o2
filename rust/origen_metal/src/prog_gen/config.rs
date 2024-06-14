use std::sync::RwLock;

/// Configuration for the program generator, an singleton is instantiated as
/// `PROG_GEN_CONFIG` in `lib.rs`
#[derive(Debug)]
pub struct Config {
    app_name: RwLock<Option<String>>,
    unique_id: RwLock<usize>,
    debug_enabled: RwLock<bool>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            app_name: RwLock::new(None),
            unique_id: RwLock::new(0),
            debug_enabled: RwLock::new(false),
        }
    }
}

impl Config {
    pub fn debug_enabled(&self) -> bool {
        *self.debug_enabled.read().unwrap()
    }
    
    pub fn set_debug_enabled(&self, enabled: bool) {
        *self.debug_enabled.write().unwrap() = enabled;
    }

    pub fn app_name(&self) -> Option<String> {
        self.app_name.read().unwrap().clone()
    }

    pub fn set_app_name(&self, name: String) {
        *self.app_name.write().unwrap() = Some(name);
    }
    
    /// Returns an ID that it guaranteed unique across threads and within the lifetime of an Origen
    /// invocation
    pub fn generate_unique_id(&self) -> usize {
        let mut unique_id = self.unique_id.write().unwrap();
        *unique_id += 1;
        *unique_id
    }
}
