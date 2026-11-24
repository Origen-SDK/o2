use super::UniquenessOption;
use parking_lot::RwLock;
use std::path::PathBuf;

/// Configuration for the program generator, an singleton is instantiated as
/// `PROG_GEN_CONFIG` in `lib.rs`
#[derive(Debug)]
pub struct Config {
    app_name: RwLock<Option<String>>,
    unique_id: RwLock<usize>,
    debug_enabled: RwLock<bool>,
    src_files: RwLock<Vec<PathBuf>>,
    test_template_load_path: RwLock<Vec<PathBuf>>,
    flow_visualization_enabled: RwLock<bool>,
    uniqueness_option: RwLock<Option<UniquenessOption>>,
    smt7: RwLock<SMT7Config>,
    smt8: RwLock<SMT8Config>,
    /// Format V93K SMT8 flows as TPGF (origen_testers) writes them
    smt8_tpgf_formatting: RwLock<bool>,
    /// Keep the case of V93K SMT8 sub-flow instance names
    smt8_keep_sub_flow_instance_case: RwLock<bool>,
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
            flow_visualization_enabled: RwLock::new(false),
            uniqueness_option: RwLock::new(None),
            smt7: RwLock::new(SMT7Config {
                render_default_tmparams: true,
            }),
            smt8: RwLock::new(SMT8Config {
                create_limits_file: true,
                render_default_tmparams: true,
            }),
            smt8_tpgf_formatting: RwLock::new(false),
            smt8_keep_sub_flow_instance_case: RwLock::new(false),
        }
    }
}

impl Config {
    pub fn debug_enabled(&self) -> bool {
        *self.debug_enabled.read()
    }

    pub fn set_debug_enabled(&self, enabled: bool) {
        *self.debug_enabled.write() = enabled;
    }

    pub fn app_name(&self) -> Option<String> {
        self.app_name.read().clone()
    }

    pub fn set_app_name(&self, name: String) {
        *self.app_name.write() = Some(name);
    }

    /// Returns an ID that it guaranteed unique across threads and within the lifetime of an Origen
    /// invocation
    pub fn generate_unique_id(&self) -> usize {
        let mut unique_id = self.unique_id.write();
        *unique_id += 1;
        *unique_id
    }

    pub fn start_src_file(&self, file: PathBuf) -> crate::Result<()> {
        let mut src_files = self.src_files.write();
        src_files.push(file);
        Ok(())
    }

    pub fn end_src_file(&self) {
        let mut src_files = self.src_files.write();
        src_files.pop();
    }

    pub fn current_src_file(&self) -> Option<PathBuf> {
        let src_files = self.src_files.read();
        src_files.last().cloned()
    }

    pub fn set_test_template_load_path(&self, paths: Vec<PathBuf>) {
        let mut lp = self.test_template_load_path.write();
        *lp = paths;
    }

    pub fn test_template_load_path(&self) -> Vec<PathBuf> {
        self.test_template_load_path.read().clone()
    }

    pub fn set_flow_visualization_enabled(&self, enabled: bool) {
        *self.flow_visualization_enabled.write() = enabled;
    }

    pub fn flow_visualization_enabled(&self) -> bool {
        *self.flow_visualization_enabled.read()
    }

    pub fn set_uniqueness_option(&self, option: UniquenessOption) {
        *self.uniqueness_option.write() = Some(option);
    }

    pub fn uniqueness_option(&self) -> Option<UniquenessOption> {
        self.uniqueness_option.read().clone()
    }

    pub fn set_smt7_options(&self, render_default_tmparams: bool) {
        *self.smt7.write() = SMT7Config {
            render_default_tmparams,
        };
    }

    pub fn smt7_options(&self) -> SMT7Config {
        self.smt7.read().clone()
    }

    pub fn set_smt8_options(&self, create_limits_file: bool, render_default_tmparams: bool) {
        *self.smt8.write() = SMT8Config {
            create_limits_file,
            render_default_tmparams,
        };
    }

    pub fn smt8_options(&self) -> SMT8Config {
        self.smt8.read().clone()
    }

    /// Set whether V93K SMT8 flows are formatted as TPGF (origen_testers) writes them, which is
    /// off by default. When on:
    /// * a sub-flow's input assignments, and its output copies, are each grouped in a block
    ///   headed by a `// <NAME> sub-flow input variables` (or `output`) comment;
    /// * only the flags which a flow sets itself are initialized to -1 at the start of it, not
    ///   those which only its sub-flows set;
    /// * a test suite's parameters are ordered by the byte order of their names.
    pub fn set_smt8_tpgf_formatting(&self, tpgf_formatting: bool) {
        *self.smt8_tpgf_formatting.write() = tpgf_formatting;
    }

    /// Returns true if V93K SMT8 flows are formatted as TPGF (origen_testers) writes them
    pub fn smt8_tpgf_formatting(&self) -> bool {
        *self.smt8_tpgf_formatting.read()
    }

    /// Set whether a V93K SMT8 sub-flow (or flow group) instance name keeps the case it is given,
    /// which it does not by default. The instance name is the one the calling flow declares
    /// (`flow <instance> calls ...`) and refers to (`<instance>.execute();`), and which the tester
    /// logs in the flow path of a test. The flow's own name, of its file and its definition,
    /// is upper case either way.
    pub fn set_smt8_keep_sub_flow_instance_case(&self, keep_case: bool) {
        *self.smt8_keep_sub_flow_instance_case.write() = keep_case;
    }

    /// Returns true if a V93K SMT8 sub-flow instance name keeps the case it is given
    pub fn smt8_keep_sub_flow_instance_case(&self) -> bool {
        *self.smt8_keep_sub_flow_instance_case.read()
    }
}
