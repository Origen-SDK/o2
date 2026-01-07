use serde::__private::de;

use crate::prog_gen::model::Flow;
use crate::prog_gen::{BinType, FlowCondition, GroupType, Model, ParamType, Test, PGM};
use crate::Result;
use crate::ast::{Node, Processor, Return};
use std::collections::{BTreeMap, HashMap};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Does the final writing of the flow AST to a SMT7 flow file
struct FlowGenerator {
    #[allow(dead_code)]
    name: String,
    description: Option<String>,
    name_override: Option<String>,
    sub_flow_open: bool,
    bypass_sub_flows: bool,
    output_dir: PathBuf,
    generated_files: Vec<PathBuf>,
    flow_header: Vec<String>,
    flow_body: Vec<String>,
    flow_footer: Vec<String>,
    indent: usize,
    model: Model,
    test_methods: BTreeMap<String, usize>,
    test_suites: BTreeMap<String, usize>,
    test_method_names: HashMap<usize, String>,
    flow_control_vars: Vec<String>,
    group_count: HashMap<String, usize>,
    inline_limits: bool,
    on_fails: Vec<Node<PGM>>,
    on_passes: Vec<Node<PGM>>,
    resources_block: bool,
    flow_stack: Vec<FlowFile>,
}

/// Contains the data for a .flow file
#[derive(Default)]
struct FlowFile {
    name: String,
    path: PathBuf,
    setup_lines: Vec<String>,
    execute_lines: Vec<String>,
}

pub fn run(ast: &Node<PGM>, output_dir: &Path, model: Model) -> Result<(Model, Vec<PathBuf>)> {
    // Write the ast to a file for debugging if needed
    let debug_ast_file = output_dir.join("debug_flow_ast.txt");
    let mut f = std::fs::File::create(&debug_ast_file)?;
    writeln!(&mut f, "{:#?}", ast)?;

    let mut p = FlowGenerator {
        name: "".to_string(),
        description: None,
        name_override: None,
        sub_flow_open: false,
        bypass_sub_flows: false,
        output_dir: output_dir.to_owned(),
        generated_files: vec![],
        flow_header: vec![],
        flow_body: vec![],
        flow_footer: vec![],
        indent: 0,
        model: model,
        test_methods: BTreeMap::new(),
        test_suites: BTreeMap::new(),
        test_method_names: HashMap::new(),
        flow_control_vars: vec![],
        group_count: HashMap::new(),
        inline_limits: true,
        on_fails: vec![],
        on_passes: vec![],
        resources_block: false,
        flow_stack: vec![],
    };

    let mut i = 0;
    for (_, t) in &p.model.tests {
        let name = format!("tm_{}", i + 1);
        p.test_method_names.insert(t.id, name.clone());
        p.test_methods.insert(name, t.id);
        i += 1;
    }

    for (_, t) in &p.model.test_invocations {
        p.test_suites
            .insert(t.get("name")?.unwrap().to_string(), t.id);
    }
    ast.process(&mut p)?;
    Ok((p.model, p.generated_files))
}

impl FlowGenerator {
    fn push_body(&mut self, line: &str) {
        if self.resources_block {
            return;
        }
        let ind = {
            if self.indent > 2 {
                4 + (3 * (self.indent - 2))
            } else {
                2 * self.indent
            }
        };
        if line == "" {
            self.flow_body.push(line.to_string());
        } else {
            self.flow_body
                .push(format!("{:indent$}{}", "", line, indent = ind));
        }
    }

    fn push_header(&mut self, line: &str) {
        if self.resources_block {
            return;
        }
        let ind = {
            if self.indent > 2 {
                4 + (3 * (self.indent - 2))
            } else {
                2 * self.indent
            }
        };
        if line == "" {
            self.flow_header.push(line.to_string());
        } else {
            self.flow_header
                .push(format!("{:indent$}{}", "", line, indent = ind));
        }
    }

    fn add_count_to_group_name(&mut self, name: &str) -> String {
        if self.group_count.contains_key(name) {
            let mut i = self.group_count[name];
            i += 1;
            self.group_count.insert(name.to_string(), i);
            format!("{}_{}", name, i)
        } else {
            self.group_count.insert(name.to_string(), 1);
            name.to_string()
        }
    }
}

/// Returns true if the given parameter has a value and it is true, returns false
/// for None or Some(false)
fn is_true(test_suite: &Test, param: &str) -> Result<bool> {
    match test_suite.get(param) {
        Err(e) => bail!("An error occurred with parameter '{}': {}", param, e),
        Ok(v) => {
            if let Some(v) = v {
                v.to_bool()
            } else {
                Ok(false)
            }
        }
    }
}

fn flags(test_suite: &Test) -> Result<Vec<&'static str>> {
    let mut flags: Vec<&str> = vec![];
    if is_true(test_suite, "bypass")? {
        flags.push("bypass");
    }
    if is_true(test_suite, "set_pass")? {
        flags.push("set_pass");
    }
    if is_true(test_suite, "set_fail")? {
        flags.push("set_fail");
    }
    if is_true(test_suite, "hold")? {
        flags.push("hold");
    }
    if is_true(test_suite, "hold_on_fail")? {
        flags.push("hold_on_fail");
    }
    if is_true(test_suite, "output_on_pass")? {
        flags.push("output_on_pass");
    }
    if is_true(test_suite, "output_on_fail")? {
        flags.push("output_on_fail");
    }
    if is_true(test_suite, "pass_value")? {
        flags.push("value_on_pass");
    }
    if is_true(test_suite, "fail_value")? {
        flags.push("value_on_fail");
    }
    if is_true(test_suite, "per_pin_on_pass")? {
        flags.push("per_pin_on_pass");
    }
    if is_true(test_suite, "per_pin_on_fail")? {
        flags.push("per_pin_on_fail");
    }
    if is_true(test_suite, "log_mixed_signal_waveform")? {
        flags.push("mx_waves_enable");
    }
    if is_true(test_suite, "fail_per_label")? {
        flags.push("fail_per_label");
    }
    if is_true(test_suite, "ffc_enable")? {
        flags.push("ffc_enable");
    }
    if is_true(test_suite, "ffv_enable")? {
        flags.push("ffv_enable");
    }
    if is_true(test_suite, "frg_enable")? {
        flags.push("frg_enable");
    }
    if is_true(test_suite, "hardware_dsp_disable")? {
        flags.push("hw_dsp_disable");
    }
    if is_true(test_suite, "force_serial")? {
        flags.push("force_serial");
    }
    Ok(flags)
}

impl FlowGenerator {
    fn open_flow_file(&mut self, name: &str) -> Result<()> {
        // Create a clean name where all spaces are underscores and lowercase and any multiple underscores
        // are reduced to single underscores
        let mut name = name.replace(" ", "_").to_lowercase();
        while name.contains("__") {
            name = name.replace("__", "_");
        }
        let flow_path = match self.flow_stack.last() {
            Some(f) => {
                let d = f.path.parent().unwrap().join(f.name.to_lowercase());
                if !d.exists() {
                    std::fs::create_dir_all(&d)?;
                }
                d.join(format!("{}.flow", name.to_uppercase()))
            }
            None => self.output_dir.join(format!("{}.flow", name.to_uppercase())),
        };
        self.flow_stack.push(FlowFile {
            name: name.to_string(),
            path: flow_path,
            ..Default::default()
        });
        Ok(())
    }

    fn close_flow_file(&mut self) -> Result<()> {
        if let Some(flow_file) = self.flow_stack.pop() {
            let mut f = std::fs::File::create(&flow_file.path)?;
            self.generated_files.push(flow_file.path.clone());

            writeln!(&mut f, "hp93000,testflow,0.1")?;
            writeln!(&mut f, "language_revision = 1;")?;
            writeln!(&mut f, "")?;

            for line in &self.flow_header {
                writeln!(&mut f, "{}", line)?;
            }
            writeln!(&mut f, "")?;
            for line in &self.flow_body {
                writeln!(&mut f, "{}", line)?;
            }
            writeln!(&mut f, "")?;
            for line in &self.flow_footer {
                writeln!(&mut f, "{}", line)?;
            }
        }
        Ok(())
    }
}

impl Processor<PGM> for FlowGenerator {
    fn on_node(&mut self, node: &Node<PGM>) -> crate::Result<Return<PGM>> {
        let result = match &node.attrs {
            PGM::ResourcesFilename(name, kind) => {
                self.model.set_resources_filename(name.to_owned(), kind);
                Return::Unmodified
            }
            PGM::Flow(name) => {
                {
                    log_debug!("Rendering flow '{}' for V93k SMT8", name);
                    self.name = name.to_owned();
                    self.model.select_flow(name)?;
                    self.open_flow_file(name)?;
                    let _ = node.process_children(self);
                    self.close_flow_file()?;
                    // Process the flow AST, this will also generate the lines in the main body
                    // of the flow
            //        self.indent += 1;
            //        self.indent += 1;
            //        let _ = node.process_children(self);
            //        self.indent -= 1;
            //        self.push_body("");

            //        let flow_name = match &self.name_override {
            //            Some(n) => n.to_owned(),
            //            None => name.to_uppercase(),
            //        };
            //        self.push_body(&format!(
            //            "}}, open,\"{}\",\"{}\"",
            //            flow_name,
            //            self.description.as_ref().unwrap_or(&"".to_string())
            //        ));
            //        self.indent -= 1;

            //        // Populate the flow header lines now that the flow has been fully generated
            //        self.indent += 1;
            //        self.push_header("{");
            //        self.indent += 1;

            //        if !self.flow_control_vars.is_empty() {
            //            self.push_header("{");
            //            self.indent += 1;
            //            // O1 did not sort these, so maintaing that for diffing
            //            //self.flow_control_vars.sort();
            //            //self.flow_control_vars.dedup();
            //            let mut lines = vec![];
            //            let mut done_flags: HashMap<String, bool> = HashMap::new();
            //            for var in &self.flow_control_vars {
            //                if !done_flags.contains_key(var) {
            //                    done_flags.insert(var.to_owned(), true);
            //                    lines.push(format!("{} = -1;", var));
            //                }
            //            }
            //            for line in lines {
            //                self.push_header(&line);
            //            }
            //            self.indent -= 1;
            //            self.push_header("}, open,\"Init Flow Control Vars\", \"\"");
            //            self.indent -= 1;
            //        }

            //        // Now render the file
            //        let flow_file = self.output_dir.join(&format!("{}.tf", name));
            //        let mut f = std::fs::File::create(&flow_file)?;
            //        self.generated_files.push(flow_file);

            //        writeln!(&mut f, "hp93000,testflow,0.1")?;
            //        writeln!(&mut f, "language_revision = 1;")?;
            //        writeln!(&mut f, "")?;
            //        writeln!(&mut f, "testmethodparameters")?;
            //        writeln!(&mut f, "")?;
            //        for (name, id) in &self.test_methods {
            //            writeln!(&mut f, "{}:", name)?;
            //            for (name, kind, value) in self.model.tests.get(id).unwrap().sorted_params()
            //            {
            //                if let Some(v) = value {
            //                    match kind {
            //                        ParamType::Voltage => {
            //                            writeln!(&mut f, r#"  "{}" = "{}[V]";"#, name, v)?
            //                        }
            //                        ParamType::Current => {
            //                            writeln!(&mut f, r#"  "{}" = "{}[A]";"#, name, v)?
            //                        }
            //                        ParamType::Time => {
            //                            writeln!(&mut f, r#"  "{}" = "{}[s]";"#, name, v)?
            //                        }
            //                        _ => writeln!(&mut f, r#"  "{}" = "{}";"#, name, v)?,
            //                    }
            //                }
            //            }
            //        }
            //        writeln!(&mut f, "")?;
            //        writeln!(&mut f, "end")?;
            //        writeln!(
            //            &mut f,
            //            "-----------------------------------------------------------------"
            //        )?;
            //        writeln!(&mut f, "testmethodlimits")?;
            //        if self.inline_limits {
            //            writeln!(&mut f, "")?;
            //            for (name, id) in &self.test_methods {
            //                let tm = self.model.tests.get(id).unwrap();
            //                writeln!(&mut f, "{}:", name)?;
            //                if tm.sub_tests.is_empty() {
            //                    let test_name = match tm.get("TestName") {
            //                        Ok(n) => match n {
            //                            Some(n) => Some(n.to_string()),
            //                            None => None,
            //                        },
            //                        Err(_) => None,
            //                    };
            //                    let test_name = match test_name {
            //                        Some(x) => x,
            //                        None => "Functional".to_string(),
            //                    };
            //                    let number = match tm.invocation(&self.model).unwrap().number {
            //                        Some(x) => format!("{}", x),
            //                        None => "".to_string(),
            //                    };
            //                    if tm.hi_limit.is_none() && tm.lo_limit.is_none() {
            //                        // Don't know why, but to align to O1, can be removed later if necessary
            //                        if number == "" {
            //                            writeln!(
            //                                &mut f,
            //                                r#"  "{}" = "":"NA":"":"NA":"":"":"";"#,
            //                                test_name
            //                            )?;
            //                        } else {
            //                            writeln!(
            //                                &mut f,
            //                                r#"  "{}" = "":"NA":"":"NA":"":"{}":"0";"#,
            //                                test_name, &number
            //                            )?;
            //                        }
            //                    } else if tm.lo_limit.is_none() {
            //                        let limit = tm.hi_limit.as_ref().unwrap();
            //                        writeln!(
            //                            &mut f,
            //                            r#"  "{}" = "":"NA":"{}":"LE":"{}":"{}":"0";"#,
            //                            test_name,
            //                            limit.value,
            //                            limit.unit_str(),
            //                            &number
            //                        )?;
            //                    } else if tm.hi_limit.is_none() {
            //                        let limit = tm.lo_limit.as_ref().unwrap();
            //                        writeln!(
            //                            &mut f,
            //                            r#"  "{}" = "{}":"GE":"":"NA":"{}":"{}":"0";"#,
            //                            test_name,
            //                            limit.value,
            //                            limit.unit_str(),
            //                            &number
            //                        )?;
            //                    } else {
            //                        let lo_limit = tm.lo_limit.as_ref().unwrap();
            //                        let hi_limit = tm.hi_limit.as_ref().unwrap();
            //                        writeln!(
            //                            &mut f,
            //                            r#"  "{}" = "{}":"GE":"{}":"LE":"{}":"{}":"0";"#,
            //                            test_name,
            //                            lo_limit.value,
            //                            hi_limit.value,
            //                            lo_limit.unit_str(),
            //                            &number
            //                        )?;
            //                    }
            //                } else {
            //                    bail!("Inline multi-limits is not implemented for V93K SMT7 yet, multiple limits encountered for test '{}'", name);
            //                }
            //            }
            //        }
            //        writeln!(&mut f, "")?;
            //        writeln!(&mut f, "end")?;
            //        writeln!(
            //            &mut f,
            //            "-----------------------------------------------------------------"
            //        )?;
            //        writeln!(&mut f, "testmethods")?;
            //        writeln!(&mut f, "")?;
            //        for (name, id) in &self.test_methods {
            //            let tm = self.model.tests.get(id).unwrap();
            //            writeln!(&mut f, "{}:", name)?;
            //            if let Some(class) = &tm.class_name {
            //                writeln!(&mut f, r#"  testmethod_class = "{}";"#, class)?;
            //            }
            //        }
            //        writeln!(&mut f, "")?;
            //        writeln!(&mut f, "end")?;
            //        writeln!(
            //            &mut f,
            //            "-----------------------------------------------------------------"
            //        )?;
            //        writeln!(&mut f, "test_suites")?;
            //        writeln!(&mut f, "")?;

            //        for (name, id) in &self.test_suites {
            //            let ts = self.model.test_invocations.get(id).unwrap();
            //            writeln!(&mut f, "{}:", name)?;
            //            if let Some(v) = ts.get("comment")? {
            //                writeln!(&mut f, "  comment = \"{}\";", v)?;
            //            }
            //            if is_true(ts, "log_first")? {
            //                writeln!(&mut f, "  ffc_on_fail = 1;")?;
            //            }
            //            let fls = flags(ts)?;
            //            if !fls.is_empty() {
            //                writeln!(&mut f, "  local_flags = {};", fls.join(", "))?;
            //            }
            //            writeln!(&mut f, "  override = 1;")?;
            //            if let Some(v) = ts.get("analog_set")? {
            //                writeln!(&mut f, "  override_anaset = \"{}\";", v)?;
            //            }
            //            if let Some(v) = ts.get("level_equation")? {
            //                writeln!(&mut f, "  override_lev_equ_set = \"{}\";", v)?;
            //            }
            //            if let Some(v) = ts.get("level_spec")? {
            //                writeln!(&mut f, "  override_lev_spec_set = \"{}\";", v)?;
            //            }
            //            if let Some(v) = ts.get("level_set")? {
            //                writeln!(&mut f, "  override_levset = {};", v)?;
            //            }
            //            if let Some(v) = ts.get("pattern")? {
            //                writeln!(&mut f, "  override_seqlbl = \"{}\";", v)?;
            //            }
            //            if let Some(v) = ts.get("test_number")? {
            //                writeln!(&mut f, "  override_test_number = \"{}\";", v)?;
            //            }
            //            if let Some(v) = ts.test_id {
            //                writeln!(&mut f, "  override_testf = {};", self.test_method_names[&v])?;
            //            }
            //            if let Some(v) = ts.get("timing_equation")? {
            //                writeln!(&mut f, "  override_tim_equ_set = \"{}\";", v)?;
            //            }
            //            if let Some(v) = ts.get("timing_spec")? {
            //                writeln!(&mut f, "  override_tim_spec_set = \"{}\";", v)?;
            //            }
            //            if let Some(v) = ts.get("timing_set")? {
            //                writeln!(&mut f, "  override_timset = \"{}\";", v)?;
            //            }
            //            if let Some(v) = ts.get("site_control")? {
            //                writeln!(&mut f, "  site_control = \"{}\";", v)?;
            //            }
            //            if let Some(v) = ts.get("site_match")? {
            //                writeln!(&mut f, "  site_match = {};", v)?;
            //            }
            //            if let Some(v) = ts.get("test_level")? {
            //                writeln!(&mut f, "  test_level = \"{}\";", v)?;
            //            }
            //        }
            //        writeln!(&mut f, "")?;
            //        writeln!(&mut f, "end")?;
            //        writeln!(
            //            &mut f,
            //            "-----------------------------------------------------------------"
            //        )?;
            //        writeln!(&mut f, "test_flow")?;
            //        writeln!(&mut f, "")?;
            //        for line in &self.flow_header {
            //            writeln!(&mut f, "{}", line)?;
            //        }
            //        for line in &self.flow_body {
            //            writeln!(&mut f, "{}", line)?;
            //        }
            //        for line in &self.flow_footer {
            //            writeln!(&mut f, "{}", line)?;
            //        }
            //        writeln!(&mut f, "")?;
            //        writeln!(&mut f, "end")?;
            //        writeln!(
            //            &mut f,
            //            "-----------------------------------------------------------------"
            //        )?;
            //        writeln!(&mut f, "binning")?;
            //        writeln!(&mut f, "end")?;
            //        writeln!(
            //            &mut f,
            //            "-----------------------------------------------------------------"
            //        )?;
            //        writeln!(&mut f, "oocrule")?;
            //        writeln!(&mut f, "")?;
            //        writeln!(&mut f, "")?;
            //        writeln!(&mut f, "end")?;
            //        writeln!(
            //            &mut f,
            //            "-----------------------------------------------------------------"
            //        )?;
            //        writeln!(&mut f, "context")?;
            //        writeln!(&mut f, "")?;
            //        writeln!(&mut f, "")?;
            //        writeln!(&mut f, "end")?;
            //        writeln!(
            //            &mut f,
            //            "-----------------------------------------------------------------"
            //        )?;
            //        writeln!(&mut f, "hardware_bin_descriptions")?;
            //        writeln!(&mut f, "")?;
            //        let flow = self.model.get_flow(None).unwrap();
            //        for (number, bin) in &flow.hardbins {
            //            if let Some(desc) = &bin.description {
            //                writeln!(&mut f, "  {} = \"{}\";", number, desc)?;
            //            }
            //        }
            //        writeln!(&mut f, "")?;
            //        writeln!(&mut f, "end")?;
            //        writeln!(
            //            &mut f,
            //            "-----------------------------------------------------------------"
            //        )?;
                }
                Return::None
            }
            //PGM::BypassSubFlows => {
            //    let orig = self.bypass_sub_flows;
            //    self.bypass_sub_flows = true;
            //    let _ = node.process_children(self);
            //    self.bypass_sub_flows = orig;
            //    Return::None
            //}
            //PGM::FlowDescription(desc) => {
            //    if !self.sub_flow_open {
            //        self.description = Some(desc.to_owned());
            //    }
            //    Return::None
            //}
            //PGM::FlowNameOverride(name) => {
            //    if !self.sub_flow_open {
            //        self.name_override = Some(name.to_owned());
            //    }
            //    Return::None
            //}
            PGM::SubFlow(name, _fid) => {
                log_debug!("Rendering sub-flow '{}'", name);
                self.open_flow_file(name)?;
                let _ = node.process_children(self);
                self.close_flow_file()?;
            //    let orig = self.sub_flow_open;
            //    self.sub_flow_open = true;
            //    self.push_body("{");
            //    self.indent += 1;
            //    let _ = node.process_children(self);
            //    self.indent -= 1;
            //    let name = self.add_count_to_group_name(name);
            //    if self.bypass_sub_flows {
            //        self.push_body(&format!("}}, groupbypass, open,\"{}\", \"\"", &name));
            //    } else {
            //        self.push_body(&format!("}}, open,\"{}\", \"\"", &name));
            //    }
            //    self.sub_flow_open = orig;
                Return::None
            }
            PGM::Group(name, _, kind, _) => {
                if kind == &GroupType::Flow {
                    log_debug!("Rendering group '{}'", name);
                    self.open_flow_file(name)?;
                    let _ = node.process_children(self);
                    self.close_flow_file()?;
            //        let mut pop_on_passed = false;
            //        let mut pop_on_failed = false;
            //        for n in &node.children {
            //            if matches!(n.attrs, PGM::OnPassed(_)) {
            //                self.on_passes.push(*n.clone());
            //                pop_on_passed = true;
            //            }
            //            if matches!(n.attrs, PGM::OnFailed(_)) {
            //                self.on_fails.push(*n.clone());
            //                pop_on_failed = true;
            //            }
            //        }
            //        let name = self.add_count_to_group_name(name);
            //        self.push_body("{");
            //        self.indent += 1;
            //        let _ = node.process_children(self);
            //        self.indent -= 1;
            //        self.push_body(&format!("}}, open,\"{}\", \"\"", &name));
            //        if pop_on_passed {
            //            let _ = self.on_passes.pop();
            //        }
            //        if pop_on_failed {
            //            let _ = self.on_fails.pop();
            //        }
                } else {
                    let _ = node.process_children(self);
                }
                Return::None
            }
            //PGM::Log(msg) => {
            //    self.push_body(&format!("print_dl(\"{}\");", msg));
            //    Return::None
            //}
            //PGM::Test(id, _flow_id) => {
            //    let (test_name, pattern) = {
            //        let test = &self.model.test_invocations[id];
            //        (
            //            test.get("name")?.unwrap().to_string(),
            //            test.get("pattern")?.map(|p| p.to_string()),
            //        )
            //    };
            //    // Record any pattern reference made by this test in the model
            //    if let Some(pattern) = pattern {
            //        self.model.record_pattern_reference(pattern, None, None);
            //    }
            //    if node
            //        .children
            //        .iter()
            //        .any(|n| matches!(n.attrs, PGM::OnFailed(_) | PGM::OnPassed(_)))
            //        || !self.on_fails.is_empty()
            //        || !self.on_passes.is_empty()
            //    {
            //        self.push_body(&format!("run_and_branch({})", &test_name));
            //        self.push_body("then");
            //        self.push_body("{");
            //        self.indent += 1;
            //        for n in &node.children {
            //            if matches!(n.attrs, PGM::OnPassed(_)) {
            //                n.process_children(self)?;
            //            }
            //        }
            //        for n in self.on_passes.clone() {
            //            n.process_children(self)?;
            //        }
            //        self.indent -= 1;
            //        self.push_body("}");
            //        self.push_body("else");
            //        self.push_body("{");
            //        self.indent += 1;
            //        for n in &node.children {
            //            if matches!(n.attrs, PGM::OnFailed(_)) {
            //                n.process_children(self)?;
            //            }
            //        }
            //        for n in self.on_fails.clone() {
            //            n.process_children(self)?;
            //        }
            //        self.indent -= 1;
            //        self.push_body("}");
            //    } else {
            //        self.push_body(&format!("run({});", &test_name));
            //    }
            //    Return::ProcessChildren
            //}
            //PGM::TestStr(name, _flow_id) => {
            //    if node
            //        .children
            //        .iter()
            //        .any(|n| matches!(n.attrs, PGM::OnFailed(_) | PGM::OnPassed(_)))
            //        || !self.on_fails.is_empty()
            //        || !self.on_passes.is_empty()
            //    {
            //        self.push_body(&format!("run_and_branch({})", name));
            //        self.push_body("then");
            //        self.push_body("{");
            //        self.indent += 1;
            //        for n in &node.children {
            //            if matches!(n.attrs, PGM::OnPassed(_)) {
            //                n.process_children(self)?;
            //            }
            //        }
            //        for n in self.on_passes.clone() {
            //            n.process_children(self)?;
            //        }
            //        self.indent -= 1;
            //        self.push_body("}");
            //        self.push_body("else");
            //        self.push_body("{");
            //        self.indent += 1;
            //        for n in &node.children {
            //            if matches!(n.attrs, PGM::OnFailed(_)) {
            //                n.process_children(self)?;
            //            }
            //        }
            //        for n in self.on_fails.clone() {
            //            n.process_children(self)?;
            //        }
            //        self.indent -= 1;
            //        self.push_body("}");
            //    } else {
            //        self.push_body(&format!("run({});", name));
            //    }
            //    Return::ProcessChildren
            //}
            //PGM::OnFailed(_) => Return::None, // Done manually within the PGMTest handler
            //PGM::OnPassed(_) => Return::None, // Done manually within the PGMTest handler
            //PGM::Else => Return::None,        // Handled by its parent
            //PGM::Condition(cond) => match cond {
            //    FlowCondition::IfJob(jobs) | FlowCondition::UnlessJob(jobs) => {
            //        let mut jobstr = "if".to_string();
            //        let else_node = node.children.iter().find(|n| matches!(n.attrs, PGM::Else));
            //        for (i, job) in jobs.iter().enumerate() {
            //            if i > 0 {
            //                jobstr += " or";
            //            }
            //            jobstr += &format!(" @JOB == \"{}\"", job.to_uppercase())
            //        }
            //        jobstr += " then";
            //        self.push_body(&jobstr);
            //        self.push_body("{");
            //        self.indent += 1;
            //        if matches!(cond, FlowCondition::IfJob(_)) {
            //            node.process_children(self)?;
            //        } else {
            //            if let Some(else_node) = else_node {
            //                else_node.process_children(self)?;
            //            }
            //        }
            //        self.indent -= 1;
            //        self.push_body("}");
            //        self.push_body("else");
            //        self.push_body("{");
            //        self.indent += 1;
            //        if matches!(cond, FlowCondition::UnlessJob(_)) {
            //            node.process_children(self)?;
            //        } else {
            //            if let Some(else_node) = else_node {
            //                else_node.process_children(self)?;
            //            }
            //        }
            //        self.indent -= 1;
            //        self.push_body("}");
            //        Return::None
            //    }
            //    FlowCondition::IfEnable(flags) | FlowCondition::UnlessEnable(flags) => {
            //        let mut flagstr = "if".to_string();
            //        let else_node = node.children.iter().find(|n| matches!(n.attrs, PGM::Else));
            //        for (i, flag) in flags.iter().enumerate() {
            //            if i > 0 {
            //                flagstr += " or";
            //            }
            //            flagstr += &format!(" @{} == 1", &flag)
            //        }
            //        flagstr += " then";
            //        self.push_body(&flagstr);
            //        self.push_body("{");
            //        self.indent += 1;
            //        if matches!(cond, FlowCondition::IfEnable(_)) {
            //            node.process_children(self)?;
            //        } else {
            //            if let Some(else_node) = else_node {
            //                else_node.process_children(self)?;
            //            }
            //        }
            //        self.indent -= 1;
            //        self.push_body("}");
            //        self.push_body("else");
            //        self.push_body("{");
            //        self.indent += 1;
            //        if matches!(cond, FlowCondition::UnlessEnable(_)) {
            //            node.process_children(self)?;
            //        } else {
            //            if let Some(else_node) = else_node {
            //                else_node.process_children(self)?;
            //            }
            //        }
            //        self.indent -= 1;
            //        self.push_body("}");
            //        Return::None
            //    }
            //    FlowCondition::IfFlag(flags) | FlowCondition::UnlessFlag(flags) => {
            //        let mut flagstr = "if".to_string();
            //        let else_node = node.children.iter().find(|n| matches!(n.attrs, PGM::Else));
            //        for (i, flag) in flags.iter().enumerate() {
            //            if i > 0 {
            //                flagstr += " or";
            //            }
            //            flagstr += &format!(" @{} == 1", flag)
            //        }
            //        flagstr += " then";
            //        self.push_body(&flagstr);
            //        self.push_body("{");
            //        self.indent += 1;
            //        if matches!(cond, FlowCondition::IfFlag(_)) {
            //            node.process_children(self)?;
            //        } else {
            //            if let Some(else_node) = else_node {
            //                else_node.process_children(self)?;
            //            }
            //        }
            //        self.indent -= 1;
            //        self.push_body("}");
            //        self.push_body("else");
            //        self.push_body("{");
            //        self.indent += 1;
            //        if matches!(cond, FlowCondition::UnlessFlag(_)) {
            //            node.process_children(self)?;
            //        } else {
            //            if let Some(else_node) = else_node {
            //                else_node.process_children(self)?;
            //            }
            //        }
            //        self.indent -= 1;
            //        self.push_body("}");
            //        Return::None
            //    }
            //    _ => Return::ProcessChildren,
            //},
            //PGM::SetFlag(flag, state, _is_auto_generated) => {
            //    let flag = format!("@{}", &flag);
            //    if *state {
            //        self.push_body(&format!("{} = 1;", &flag));
            //    } else {
            //        self.push_body(&format!("{} = 0;", &flag));
            //    }
            //    if !self.resources_block {
            //        self.flow_control_vars.push(flag.to_string());
            //    }
            //    Return::None
            //}
            //PGM::Bin(bin, softbin, kind) => {
            //    let softbin = match softbin {
            //        None => "".to_string(),
            //        Some(s) => format!("{}", s),
            //    };
            //    let t = match kind {
            //        BinType::Bad => ("fail", "bad", "red"),
            //        BinType::Good => ("", "good", "green"),
            //    };
            //    self.push_body(&format!(
            //        r#"stop_bin "{}", "{}", , {}, noreprobe, {}, {}, over_on;"#,
            //        softbin, t.0, t.1, t.2, bin
            //    ));
            //    Return::None
            //}
            //PGM::Render(text) => {
            //    self.push_body(&format!(r#"{}"#, text));
            //    Return::None
            //}
            //PGM::Resources => {
            //    let orig = self.resources_block;
            //    self.resources_block = true;
            //    node.process_children(self)?;
            //    self.resources_block = orig;
            //    Return::None
            //}
            _ => Return::ProcessChildren,
        };
        Ok(result)
    }
}
