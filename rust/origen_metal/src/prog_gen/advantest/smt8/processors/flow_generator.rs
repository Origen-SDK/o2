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
    limits_file: Option<std::fs::File>,
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
    ast.to_file("smt8_flow.txt")?;

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
        limits_file: None,
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
    fn current_flow_path(&self) -> Option<String> {
        if self.flow_stack.is_empty() || self.flow_stack.len() == 1 {
            None
        } else {
            let mut p = "".to_string();
            for f in &self.flow_stack[1..self.flow_stack.len()] {
                if p.is_empty() {
                    p = f.name.to_string();
                } else {
                    p = format!("{}.{}", p, f.name);
                }
            }
            Some(p)
        }
    }

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
                log_debug!("Rendering flow '{}' for V93k SMT8", name);

                let limits_dir = self.output_dir.parent().unwrap().join("limits");
                if !limits_dir.exists() {
                    std::fs::create_dir_all(&limits_dir)?;
                }
                let limits_file = limits_dir.join(format!(
                    "Main.{}_Tests.csv",
                    name.replace(" ", "_").to_uppercase()
                ));

                let mut f = std::fs::File::create(&limits_file)?;
                self.generated_files.push(limits_file.clone());
                writeln!(&mut f, "Test Suite,Test,Test Number,Test Text,Low Limit,High Limit,Unit,Soft Bin")?;
                writeln!(&mut f, ",,,,default,default")?;
                self.limits_file = Some(f);


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
                let mut test_number = "".to_string();
                let mut lo_limit = "".to_string();
                let mut hi_limit = "".to_string();
                let (bin, softbin) = extract_bin(&node.children);
                let (test_name, pattern, tname) = {
                    let test_invocation = &self.model.test_invocations[id];
                    let mut test_name = test_invocation.get("name")?.unwrap().to_string();
                    {
                        let current_flow = self.flow_stack.last_mut().unwrap();
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
                        if let Some(tn) = test_invocation.number {
                            test_number = format!("{}", tn);
                        }
                        if let Some(l) = &test_invocation.lo_limit {
                            lo_limit = format!("{}{}", l.value, l.unit_str());
                        }
                        if let Some(h) = &test_invocation.hi_limit {
                            hi_limit = format!("{}{}", h.value, h.unit_str());
                        }
                        //if *id == 6 {
                        //    dbg!(&test_invocation);
                        //    let test = test_invocation.test(&self.model).unwrap();
                        //    dbg!(test);
                        //}
                        (
                            test_name,
                            test_invocation.get("pattern")?.map(|p| p.to_string()),
                            test_invocation.tname.clone()
                        )
                    }

                };
                if !self.resources_block {
                    let test_path = match self.current_flow_path() {
                        Some(p) => format!("{}.{}", p, &test_name),
                        None => test_name.clone(),
                    };  
                    let b = if let Some(softbin) = softbin {
                        softbin.to_string()
                    } else if let Some(bin) = bin {
                        bin.to_string()
                    } else {
                        "".to_string()
                    };
                    // Test Suite,Test,Test Number,Test Text,Low Limit,High Limit,Unit,Soft Bin"
                    let test_text = if let Some(test_name_alt) = &tname {
                        format!("{}.{}", test_name, test_name_alt)
                    } else {
                        test_name.clone()
                    };
                    writeln!(
                        self.limits_file.as_mut().unwrap(),
                        "{},{},{},{},{},{},,{}",
                        test_path,
                        &tname.as_ref().unwrap_or(&test_name),
                        test_number,
                        &test_text,
                        &lo_limit,
                        &hi_limit,
                        b
                    )?;
                }
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
            PGM::TestStr(name, _flow_id, _bin, softbin, number) => {
                if !self.resources_block {
                    let test_path = match self.current_flow_path() {
                        Some(p) => format!("{}.{}", p, &name),
                        None => name.clone(),
                    };  
                    writeln!(
                        self.limits_file.as_mut().unwrap(),
                        "{},{},{},{},0,0,,{}",
                        test_path,
                        &name,
                        number.as_ref().map(|n| n.to_string()).unwrap_or_default(),
                        &name,
                        softbin.as_ref().map(|b| b.to_string()).unwrap_or_default()
                    )?;
                }
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


fn extract_bin(nodes: &Vec<Box<Node<PGM>>>) -> (Option<usize>, Option<usize>) {
    for n in nodes {
        match &n.attrs {
            PGM::OnFailed(_) => {
                for n in &n.children {
                    if let PGM::Bin(bin, softbin, _) = n.attrs {
                        return (Some(bin), softbin);
                    }
                }
            }
            _ => {}
        }
    }
    (None, None)
}