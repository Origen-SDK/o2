use crate::prog_gen::advantest::smt8::processors::create_flow_data::FlowData;
use crate::prog_gen::{BinType, FlowCondition, GroupType, Model, PGM, ParamValue};
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
    model: Model,
    test_methods: BTreeMap<String, usize>,
    test_suites: BTreeMap<String, usize>,
    test_method_names: HashMap<usize, String>,
    resources_block: bool,
    flow_stack: Vec<FlowFile>,
}

/// Contains the data for a .flow file
#[derive(Default)]
struct FlowFile {
    name: String,
    path: PathBuf,
    indent: usize,
    execute_lines: Vec<String>,
    execute_lines_buffer: Vec<String>,
    buffer_execute_lines: bool,
    render_bins: bool,
    test_ids: Vec<(String, usize)>,
    existing_test_counter: HashMap<String, usize>,
    existing_flow_counter: HashMap<String, usize>,
    sub_flows: Vec<String>,
    flow_data: FlowData
}

impl FlowFile {
    /// Adds a line to the execute section of the flow file, automatically
    /// indenting it appropriately
    fn execute_line(&mut self, line: String) {
        let indent = "    ".repeat(self.indent);
        if self.buffer_execute_lines {
            self.execute_lines_buffer.push(format!("{}{}", indent, line));
        } else {
            self.execute_lines.push(format!("{}{}", indent, line));
        }
    }

    /// Flushes any buffered execute lines to the main execute lines
    /// section
    fn flush_buffered_execute_lines(&mut self, indent: usize) {
        for line in &self.execute_lines_buffer {
            self.execute_lines.push(format!("{}{}", "    ".repeat(indent), line));
        }
        self.execute_lines_buffer.clear();
    }
}

pub fn run(ast: &Node<PGM>, output_dir: &Path, model: Model) -> Result<(Model, Vec<PathBuf>)> {
    // For debugging
    //ast.to_file("ast.txt")?;

    let mut p = FlowGenerator {
        name: "".to_string(),
        description: None,
        name_override: None,
        sub_flow_open: false,
        bypass_sub_flows: false,
        output_dir: output_dir.to_owned(),
        generated_files: vec![],
        model: model,
        test_methods: BTreeMap::new(),
        test_suites: BTreeMap::new(),
        test_method_names: HashMap::new(),
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
    fn open_flow_file(&mut self, name: &str, flow_data: FlowData) -> Result<()> {
        // Create a clean name where all spaces are underscores and lowercase and any multiple underscores
        // are reduced to single underscores
        let mut name = name.replace(" ", "_").to_uppercase();
        while name.contains("__") {
            name = name.replace("__", "_");
        }
        if let Some(current_flow) = self.flow_stack.last_mut() {
            if current_flow.existing_flow_counter.contains_key(&name) {
                let count = current_flow.existing_flow_counter.get(&name).unwrap() + 1;
                current_flow.existing_flow_counter.insert(name.to_owned(), count);
                name = format!("{}_{}", name, count);
            } else {
                current_flow.existing_flow_counter.insert(name.to_owned(), 0);
            }
        }
        let flow_path = match self.flow_stack.last() {
            Some(f) => {
                let d = f.path.parent().unwrap().join(f.name.to_lowercase());
                if !d.exists() {
                    std::fs::create_dir_all(&d)?;
                }
                d.join(format!("{}.flow", name))
            }
            None => self.output_dir.join(format!("{}.flow", name)),
        };
        self.flow_stack.push(FlowFile {
            name: name.to_string(),
            path: flow_path,
            render_bins: true,
            flow_data,
            ..Default::default()
        });
        Ok(())
    }

    fn close_flow_file(&mut self) -> Result<FlowFile> {
        let flow_file = self.flow_stack.pop().unwrap();
        let mut f = std::fs::File::create(&flow_file.path)?;
        self.generated_files.push(flow_file.path.clone());

        writeln!(&mut f, "flow {} {{", flow_file.name)?;
        // Remove any vars from input_vars that are also in output_vars
        let sorted_input_vars = flow_file.flow_data.sorted_input_vars();
        for v in &sorted_input_vars {
            // Maybe typing is needed here later?
            if v == "JOB" {
                writeln!(&mut f, "    in {} = \"\";", v)?;
            } else {
                writeln!(&mut f, "    in {} = -1;", v)?;
            }
        }
        if !sorted_input_vars.is_empty() {
            writeln!(&mut f, "")?;
        }
        //// If not the top-level flow itself
        //if !self.flow_stack.is_empty() {
            for v in flow_file.flow_data.sorted_output_flags() {
                writeln!(&mut f, "    out {} = -1;", v)?;
            }
            if !flow_file.flow_data.output_flags.is_empty() {
                writeln!(&mut f, "")?;
            }
        //}
        writeln!(&mut f, "    setup {{")?;
        // sort the test suites by the name to ensure consistent ordering in the setup section
        let mut sorted_test_ids = flow_file.test_ids.clone();
        sorted_test_ids.sort_by_key(|(test_name, _)| test_name.clone());
        for (test_name, tid) in &sorted_test_ids {
            let test_invocation = &self.model.test_invocations[tid];
            //if flow_file.name == "ERASE_VFY" {
            //    dbg!(test_invocation);
            //}
            if let Some(test) = test_invocation.test(&self.model) {
                //if flow_file.name == "ERASE_VFY" {
                //    dbg!(test);
                //}
                writeln!(&mut f, "        suite {} calls {} {{", test_name, test.class_name.as_ref().unwrap())?;
                if let Some(pattern) = test_invocation.get("pattern")?.map(|p| p.to_string()) {
                    writeln!(&mut f, "            measurement.pattern = setupRef(OrigenTesters.patterns.{});", pattern)?;
                }
                if let Some(spec) = test_invocation.get("spec")?.map(|p| p.to_string()) {
                    writeln!(&mut f, "            measurement.specification = setupRef(OrigenTesters.specs.{});", spec)?;
                }
                let sorted_param_keys =  {
                    let mut keys: Vec<&String> = test.values.keys().collect();
                    keys.sort();
                    keys
                };

                for param in sorted_param_keys {
                    let value = test.values.get(param).unwrap();
                    match value {
                        ParamValue::String(v) | ParamValue::Any(v) => {
                            writeln!(&mut f, "            {} = \"{}\";", param, v)?;
                        }
                        ParamValue::Int(v) => {
                            writeln!(&mut f, "            {} = {};", param, v)?;
                        }
                        ParamValue::UInt(v) =>  {
                            writeln!(&mut f, "            {} = {};", param, v)?;
                        }
                        ParamValue::Float(v) => {
                            writeln!(&mut f, "            {} = {};", param, v)?;
                        }
                        ParamValue::Current(v) => {
                            writeln!(&mut f, "            {} = \"{}[A]\";", param, v)?;
                        }
                        ParamValue::Voltage(v) =>  {
                            writeln!(&mut f, "            {} = \"{}[V]\";", param, v)?;
                        }
                        ParamValue::Time(v) => {
                            writeln!(&mut f, "            {} = \"{}[s]\";", param, v)?;
                        }
                        ParamValue::Frequency(v) => {
                            writeln!(&mut f, "            {} = \"{}[Hz]\";", param, v)?;
                        }
                        ParamValue::Bool(v) => {
                            if *v {
                                writeln!(&mut f, "            {} = true;", param)?;
                            } else {
                                writeln!(&mut f, "            {} = false;", param)?;
                            }
                        }
                    }
                }
                writeln!(&mut f, "        }}")?;
            }
            writeln!(&mut f, "")?;
        }
        for sub_flow in &flow_file.sub_flows {
            let relative_path = flow_file.path.strip_prefix(&self.output_dir).unwrap().parent().unwrap().join(flow_file.name.to_lowercase());
            writeln!(&mut f, "        flow {} calls OrigenTesters.flows.{}.{} {{ }}", sub_flow, relative_path.to_str().unwrap().replace("/", "."), sub_flow)?;
        }
        writeln!(&mut f, "    }}")?;
        writeln!(&mut f, "")?;
        writeln!(&mut f, "    execute {{")?;
        for v in flow_file.flow_data.sorted_modified_flags() {
            writeln!(&mut f, "        {} = -1;", v)?;
        }
        if !flow_file.flow_data.output_flags.is_empty() {
            writeln!(&mut f, "")?;
        }
        for line in &flow_file.execute_lines {
            writeln!(&mut f, "        {}", line)?;
        }
        writeln!(&mut f, "    }}")?;
        writeln!(&mut f, "}}")?;
        Ok(flow_file)
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
                    let flow_data = if let PGM::FlowData(fdata) = &node.children[0].attrs {
                        fdata.clone()
                    } else {
                        FlowData::default()
                    };
                    self.open_flow_file(name, flow_data)?;
                    node.process_children(self)?;
                    self.close_flow_file()?;

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
            PGM::BypassSubFlows => {
                let orig = self.bypass_sub_flows;
                self.bypass_sub_flows = true;
                node.process_children(self)?;
                self.bypass_sub_flows = orig;
                Return::None
            }
            PGM::FlowDescription(desc) => {
                if self.flow_stack.len() == 1 {
                    self.description = Some(desc.to_owned());
                }
                Return::None
            }
            PGM::FlowNameOverride(name) => {
                if self.flow_stack.len() == 1 {
                    self.name_override = Some(name.to_owned());
                }
                Return::None
            }
            PGM::SubFlow(name, _fid) => {
                log_debug!("Rendering sub-flow '{}'", name);
                let flow_data = if let PGM::FlowData(fdata) = &node.children[0].attrs {
                    fdata.clone()
                } else {
                    FlowData::default()
                };
                self.open_flow_file(name, flow_data)?;
                let orig = self.sub_flow_open;
                self.sub_flow_open = true;
                node.process_children(self)?;
                self.sub_flow_open = orig;
                let flow = self.close_flow_file()?;
                let current_flow = self.flow_stack.last_mut().unwrap(); 
                for v in flow.flow_data.sorted_input_vars() {
                    current_flow.execute_line(format!("{}.{} = {};", flow.name, v, v));
                }
                current_flow.execute_line(format!("{}.execute();", flow.name));
                current_flow.sub_flows.push(flow.name.clone());
                for v in flow.flow_data.sorted_output_flags() {
                    if current_flow.flow_data.output_flags.contains(&v) || current_flow.flow_data.referenced_flags.contains(&v) {
                        current_flow.execute_line(format!("{} = {}.{};", v, flow.name, v));
                    }
                }
                Return::None
            }
            PGM::Group(name, _, kind, _) => {
                if kind == &GroupType::Flow {
                    log_debug!("Rendering group '{}'", name);
                    let flow_data = if let PGM::FlowData(fdata) = &node.children[0].attrs {
                        fdata.clone()
                    } else {
                        FlowData::default()
                    };
                    self.open_flow_file(name, flow_data)?;
                    node.process_children(self)?;
                    let flow = self.close_flow_file()?;
                    let current_flow = self.flow_stack.last_mut().unwrap(); 
                    for v in flow.flow_data.sorted_input_vars() {
                        current_flow.execute_line(format!("{}.{} = {};", flow.name, v, v));
                    }
                    current_flow.execute_line(format!("{}.execute();", flow.name));
                    current_flow.sub_flows.push(flow.name.clone());
                    for v in flow.flow_data.sorted_output_flags() {
                        if current_flow.flow_data.output_flags.contains(&v) || current_flow.flow_data.referenced_flags.contains(&v) {
                            current_flow.execute_line(format!("{} = {}.{};", v, flow.name, v));
                        }
                    }
                    if node
                        .children
                        .iter()
                        .any(|n| matches!(n.attrs, PGM::OnFailed(_) | PGM::OnPassed(_)))
                    {
                        for n in &node.children {
                            if matches!(n.attrs, PGM::OnPassed(_)) {
                                self.flow_stack.last_mut().unwrap().buffer_execute_lines = true;
                                n.process_children(self)?;
                                {
                                    let current_flow = self.flow_stack.last_mut().unwrap(); 
                                    current_flow.buffer_execute_lines = false;
                                    if !current_flow.execute_lines_buffer.is_empty() {
                                        current_flow.execute_line(format!("if ({}.pass) {{", flow.name));
                                        current_flow.flush_buffered_execute_lines(1);
                                        current_flow.execute_line("}".to_string());
                                    }
                                }
                            }
                            if matches!(n.attrs, PGM::OnFailed(_)) {
                                self.flow_stack.last_mut().unwrap().buffer_execute_lines = true;
                                n.process_children(self)?;
                                {
                                    let current_flow = self.flow_stack.last_mut().unwrap(); 
                                    current_flow.buffer_execute_lines = false;
                                    if !current_flow.execute_lines_buffer.is_empty() {
                                        current_flow.execute_line(format!("if (!{}.pass) {{", flow.name));
                                        current_flow.flush_buffered_execute_lines(1);
                                        current_flow.execute_line("}".to_string());
                                    }
                                }
                            }
                        }
                    }
                } else {
                    node.process_children(self)?;
                }
                Return::None
            }
            PGM::Log(msg) => {
                self.flow_stack.last_mut().unwrap().execute_line(format!("println(\"{}\");", msg));
                Return::None
            }
            PGM::Test(id, _flow_id) => {
                let (test_name, pattern) = {
                    let test = &self.model.test_invocations[id];
                    let current_flow = self.flow_stack.last_mut().unwrap();
                    let mut test_name = test.get("name")?.unwrap().to_string();
                    if current_flow.existing_test_counter.contains_key(&test_name) {
                        let orig_test_name = test_name.clone();
                        test_name = format!(
                            "{}_{}",
                            test_name,
                            current_flow.existing_test_counter.get(&orig_test_name).unwrap()
                        );
                        let count = current_flow.existing_test_counter.get(&orig_test_name).unwrap() + 1;
                        current_flow
                            .existing_test_counter
                            .insert(orig_test_name, count);
                    } else {
                        current_flow
                            .existing_test_counter
                            .insert(test_name.to_owned(), 1);
                    }
                    current_flow.test_ids.push((test_name.clone(), *id));
                    (
                        test_name,
                        test.get("pattern")?.map(|p| p.to_string()),
                    )
                };
                // Record any pattern reference made by this test in the model
                if let Some(pattern) = pattern {
                    self.model.record_pattern_reference(pattern, None, None);
                }
                if !self.resources_block {
                    self.flow_stack.last_mut().unwrap().execute_line(format!("{}.execute();", &test_name));
                    if node
                        .children
                        .iter()
                        .any(|n| matches!(n.attrs, PGM::OnFailed(_) | PGM::OnPassed(_)))
                    {
                        for n in &node.children {
                            if matches!(n.attrs, PGM::OnPassed(_)) {
                                self.flow_stack.last_mut().unwrap().buffer_execute_lines = true;
                                self.flow_stack.last_mut().unwrap().render_bins = false;
                                n.process_children(self)?;
                                {
                                    let current_flow = self.flow_stack.last_mut().unwrap(); 
                                    current_flow.buffer_execute_lines = false;
                                    current_flow.render_bins = true;
                                    if !current_flow.execute_lines_buffer.is_empty() {
                                        current_flow.execute_line(format!("if ({}.pass) {{", &test_name));
                                        current_flow.flush_buffered_execute_lines(1);
                                        current_flow.execute_line("}".to_string());
                                    }
                                }
                            }
                            if matches!(n.attrs, PGM::OnFailed(_)) {
                                self.flow_stack.last_mut().unwrap().buffer_execute_lines = true;
                                self.flow_stack.last_mut().unwrap().render_bins = false;
                                n.process_children(self)?;
                                {
                                    let current_flow = self.flow_stack.last_mut().unwrap(); 
                                    current_flow.buffer_execute_lines = false;
                                    current_flow.render_bins = true;
                                    if !current_flow.execute_lines_buffer.is_empty() {
                                        current_flow.execute_line(format!("if (!{}.pass) {{", &test_name));
                                        current_flow.flush_buffered_execute_lines(1);
                                        current_flow.execute_line("}".to_string());
                                    }
                                }
                            }
                        }
                    }
                }
                Return::ProcessChildren
            }
            PGM::TestStr(name, _flow_id) => {
                self.flow_stack.last_mut().unwrap().execute_line(format!("{}.execute();", name));
                if node
                    .children
                    .iter()
                    .any(|n| matches!(n.attrs, PGM::OnFailed(_) | PGM::OnPassed(_)))
                {
                    for n in &node.children {
                        if matches!(n.attrs, PGM::OnPassed(_)) {
                            self.flow_stack.last_mut().unwrap().buffer_execute_lines = true;
                            self.flow_stack.last_mut().unwrap().render_bins = false;
                            n.process_children(self)?;
                            {
                                let current_flow = self.flow_stack.last_mut().unwrap(); 
                                current_flow.buffer_execute_lines = false;
                                current_flow.render_bins = true;
                                if !current_flow.execute_lines_buffer.is_empty() {
                                    current_flow.execute_line(format!("if ({}.pass) {{", name));
                                    current_flow.flush_buffered_execute_lines(1);
                                    current_flow.execute_line("}".to_string());
                                }
                            }
                        }
                        if matches!(n.attrs, PGM::OnFailed(_)) {
                            self.flow_stack.last_mut().unwrap().buffer_execute_lines = true;
                            self.flow_stack.last_mut().unwrap().render_bins = false;
                            n.process_children(self)?;
                            {
                                let current_flow = self.flow_stack.last_mut().unwrap(); 
                                current_flow.buffer_execute_lines = false;
                                current_flow.render_bins = true;
                                if !current_flow.execute_lines_buffer.is_empty() {
                                    current_flow.execute_line(format!("if (!{}.pass) {{", name));
                                    current_flow.flush_buffered_execute_lines(1);
                                    current_flow.execute_line("}".to_string());
                                }
                            }
                        }
                    }
                }
                Return::ProcessChildren
            }
            PGM::OnFailed(_) => Return::None, // Handled within the PGMTest handler
            PGM::OnPassed(_) => Return::None, // Handled within the PGMTest handler
            PGM::Else => Return::None,        // Handled by its parent
            PGM::Condition(cond) => match cond {
                FlowCondition::IfJob(jobs) | FlowCondition::UnlessJob(jobs) => {
                    let else_node = node.children.iter().find(|n| matches!(n.attrs, PGM::Else));
                    {
                        let current_flow = self.flow_stack.last_mut().unwrap();
                        current_flow.execute_line(format!(
                            "if ({}) {{",
                            jobs
                                .iter()
                                .map(|j| {
                                    if jobs.len() > 1 {
                                        format!("(JOB == \"{}\")", j.to_uppercase())
                                    } else {
                                        format!("JOB == \"{}\"", j.to_uppercase())
                                    }
                                })
                                .collect::<Vec<String>>()
                                .join(" || ")
                        ));
                        current_flow.indent += 1;
                    }
                    if matches!(cond, FlowCondition::IfJob(_)) {
                        node.process_children(self)?;
                    } else {
                        if let Some(else_node) = else_node {
                            else_node.process_children(self)?;
                        }
                    }
                    {
                        let current_flow = self.flow_stack.last_mut().unwrap();
                        current_flow.indent -= 1;
                        current_flow.execute_line("} else {".to_string());
                        current_flow.indent += 1;
                    }
                    if matches!(cond, FlowCondition::UnlessJob(_)) {
                        node.process_children(self)?;
                    } else {
                        if let Some(else_node) = else_node {
                            else_node.process_children(self)?;
                        }
                    }
                    let current_flow = self.flow_stack.last_mut().unwrap();
                    current_flow.indent -= 1;
                    current_flow.execute_line("}".to_string());
                    Return::None
                }
                FlowCondition::IfEnable(flags) | FlowCondition::UnlessEnable(flags) => {
                    let else_node = node.children.iter().find(|n| matches!(n.attrs, PGM::Else));
                    {
                        let current_flow = self.flow_stack.last_mut().unwrap();
                        current_flow.execute_line(format!(
                            "if ({}) {{",
                            flags
                                .iter()
                                .map(|f| {
                                    if flags.len() > 1 {
                                        format!("({} == 1)", f.to_uppercase())
                                    } else {
                                        format!("{} == 1", f.to_uppercase())
                                    }
                                })  
                                .collect::<Vec<String>>()
                                .join(" || ")
                        ));
                        current_flow.indent += 1;
                    }
                    if matches!(cond, FlowCondition::IfEnable(_)) {
                        node.process_children(self)?;
                    } else {
                        if let Some(else_node) = else_node {
                            else_node.process_children(self)?;
                        }
                    }
                    {
                        let current_flow = self.flow_stack.last_mut().unwrap();
                        current_flow.indent -= 1;
                        current_flow.execute_line("} else {".to_string());
                        current_flow.indent += 1;
                    }
                    if matches!(cond, FlowCondition::UnlessEnable(_)) {
                        node.process_children(self)?;
                    } else {
                        if let Some(else_node) = else_node {
                            else_node.process_children(self)?;
                        }
                    }
                    let current_flow = self.flow_stack.last_mut().unwrap();
                    current_flow.indent -= 1;
                    current_flow.execute_line("}".to_string());
                    Return::None
                }
                FlowCondition::IfFlag(flags) | FlowCondition::UnlessFlag(flags) => {
                    let else_node = node.children.iter().find(|n| matches!(n.attrs, PGM::Else));
                    {
                        let current_flow = self.flow_stack.last_mut().unwrap();
                        current_flow.execute_line(format!(
                            "if ({}) {{",
                            flags
                                .iter()
                                .map(|f| {
                                    if flags.len() > 1 {
                                        format!("({} == 1)", f.to_uppercase())
                                    } else {
                                        format!("{} == 1", f.to_uppercase())
                                    }
                                })
                                .collect::<Vec<String>>()
                                .join(" || ")
                        ));
                        current_flow.indent += 1;
                    }
                    if matches!(cond, FlowCondition::IfFlag(_)) {
                        node.process_children(self)?;
                    } else {
                        if let Some(else_node) = else_node {
                            else_node.process_children(self)?;
                        }
                    }
                    {
                        let current_flow = self.flow_stack.last_mut().unwrap();
                        current_flow.indent -= 1;
                        current_flow.execute_line("} else {".to_string());
                        current_flow.indent += 1;
                    }
                    if matches!(cond, FlowCondition::UnlessFlag(_)) {
                        node.process_children(self)?;
                    } else {
                        if let Some(else_node) = else_node {
                            else_node.process_children(self)?;
                        }
                    }
                    let current_flow = self.flow_stack.last_mut().unwrap();
                    current_flow.indent -= 1;
                    current_flow.execute_line("}".to_string());
                    Return::None
                }
                _ => Return::ProcessChildren,
            },
            PGM::SetFlag(flag, state, _is_auto_generated) => {
                let flag = flag.to_uppercase();
                let current_flow = self.flow_stack.last_mut().unwrap();
                if *state {
                    current_flow.execute_line(format!("{} = 1;", &flag));
                } else {
                    current_flow.execute_line(format!("{} = 0;", &flag));
                }
                Return::None
            }
            PGM::Bin(bin, softbin, kind) => {
                let current_flow = self.flow_stack.last_mut().unwrap();
                // Currently only rendering pass bins or those not associated with a test (should come from the bin
                // table if its associated with a test)  (same as O1)
                match kind {
                    BinType::Bad => {
                        if current_flow.render_bins {
                            current_flow.execute_line(format!(
                                "addBin({});",
                                softbin.unwrap_or(*bin)
                            ));
                        }

                    },
                    BinType::Good => {
                        current_flow.execute_line(format!(
                            "addBin({});",
                            softbin.unwrap_or(*bin)
                        ));
                    }
                };
                Return::None
            }
            //PGM::Render(text) => {
            //    self.push_body(&format!(r#"{}"#, text));
            //    Return::None
            //}
            PGM::Resources => {
                let orig = self.resources_block;
                self.resources_block = true;
                node.process_children(self)?;
                self.resources_block = orig;
                Return::None
            }
            _ => Return::ProcessChildren,
        };
        Ok(result)
    }
}
