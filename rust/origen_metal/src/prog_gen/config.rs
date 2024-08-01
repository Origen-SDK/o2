use std::{path::PathBuf, sync::RwLock};

/// Configuration for the program generator, an singleton is instantiated as
/// `PROG_GEN_CONFIG` in `lib.rs`
#[derive(Debug)]
pub struct Config {
    app_name: RwLock<Option<String>>,
    unique_id: RwLock<usize>,
    debug_enabled: RwLock<bool>,
    src_files: RwLock<Vec<PathBuf>>,
    test_template_load_path: RwLock<Vec<PathBuf>>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            app_name: RwLock::new(None),
            unique_id: RwLock::new(0),
            debug_enabled: RwLock::new(false),
            src_files: RwLock::new(vec![]),
            test_template_load_path: RwLock::new(vec![]),
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

    pub fn start_src_file(&self, file: PathBuf) -> crate::Result<()> {
        let mut src_files = self.src_files.write().unwrap();
        src_files.push(file);
        Ok(())
    }

    pub fn end_src_file(&self) {
        let mut src_files = self.src_files.write().unwrap();
        src_files.pop();
    }

    pub fn current_src_file(&self) -> Option<PathBuf> {
        let src_files = self.src_files.read().unwrap();
        src_files.last().cloned()
    }
    
    pub fn set_test_template_load_path(&self, paths: Vec<PathBuf>) {
        let mut lp = self.test_template_load_path.write().unwrap();
        *lp = paths;
    }
    
    pub fn test_template_load_path(&self) -> Vec<PathBuf> {
        self.test_template_load_path.read().unwrap().clone()
    }
}
