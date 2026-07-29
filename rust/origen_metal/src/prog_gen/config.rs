use std::{path::PathBuf, sync::RwLock};
use super::UniquenessOption;

/// Configuration for the program generator, an singleton is instantiated as
/// `PROG_GEN_CONFIG` in `lib.rs`
#[derive(Debug)]
pub struct Config {
    app_name: RwLock<Option<String>>,
    unique_id: RwLock<usize>,
    debug_enabled: RwLock<bool>,
    src_files: RwLock<Vec<PathBuf>>,
    test_template_load_path: RwLock<Vec<PathBuf>>,
    uniqueness_option: RwLock<Option<UniquenessOption>>,
    smt7: RwLock<SMT7Config>,
    smt8: RwLock<SMT8Config>,
}

#[derive(Debug, Clone)]
pub struct SMT8Config {
    pub create_limits_file: bool,
    pub render_default_tmparams: bool,
}

#[derive(Debug, Clone)]
pub struct SMT7Config {
    pub render_default_tmparams: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            app_name: RwLock::new(None),
            unique_id: RwLock::new(0),
            debug_enabled: RwLock::new(false),
            src_files: RwLock::new(vec![]),
            test_template_load_path: RwLock::new(vec![]),
            uniqueness_option: RwLock::new(None),
            smt7: RwLock::new(SMT7Config {
                render_default_tmparams: true,
            }),
            smt8: RwLock::new(SMT8Config {
                create_limits_file: true,
                render_default_tmparams: true,
            }),
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
    
    pub fn set_uniqueness_option(&self, option: UniquenessOption) {
        *self.uniqueness_option.write().unwrap() = Some(option);
    }
    
    pub fn uniqueness_option(&self) -> Option<UniquenessOption> {
        self.uniqueness_option.read().unwrap().clone()
    }

    pub fn set_smt7_options(&self, render_default_tmparams: bool) {
        *self.smt7.write().unwrap() = SMT7Config {
            render_default_tmparams,
        };
    }

    pub fn smt7_options(&self) -> SMT7Config {
        self.smt7.read().unwrap().clone()
    }

    pub fn set_smt8_options(&self, create_limits_file: bool, render_default_tmparams: bool) {
        *self.smt8.write().unwrap() = SMT8Config {
            create_limits_file,
            render_default_tmparams,
        };
    }

    pub fn smt8_options(&self) -> SMT8Config {
        self.smt8.read().unwrap().clone()
    }
}
