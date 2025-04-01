use crate::prog_gen::{FlowCondition, Model, ParamType, Test, PGM};
use crate::Result;
use crate::ast::{Node, Processor, Return};
use std::collections::{BTreeMap, HashMap};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Does the final writing of the flow AST to a SMT8 flow file
pub struct FlowGenerator {
    #[allow(dead_code)]
    name: String,
    description: Option<String>,
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
    ///
    curr_flow_key: String,
    file_hier: Vec<String>,
    flow_files: BTreeMap<String, FlowFile>,
}

pub struct FlowFile {
    id: String,
    file: String,
    indent: usize,
    ///
    subflows: HashMap<String, usize>,
    subflow_defs: HashMap<String, FlowDef>,
    suites: HashMap<String, usize>,
    suite_defs: HashMap<String, SuiteDef>,
    ///
    flow_in_vars: HashMap<String, String>,
    flow_control_vars: Vec<String>,
    ///
    setup_header: Vec<String>,
    setup_footer: Vec<String>,
    ///
    flow_header: Vec<String>,
    flow_body: Vec<String>,
    flow_footer: Vec<String>,
}

pub struct SuiteDef {
    klass: String,
    lines: Vec<String>,
}

pub struct FlowDef {
    path: String,
    lines: Vec<String>,
}

impl FlowFile {
    fn new(id: &str, path: &Vec<String>) -> Self {
        let mut file_name = path.join("/");
        if !file_name.is_empty() {
            file_name.push_str("/");
        }
        file_name.push_str(id.to_uppercase().as_str());
        file_name.push_str(".flow");

        Self {
            id: id.to_owned(),
            file: file_name,
            indent: 1,
            subflows: HashMap::new(),
            subflow_defs: HashMap::new(),
            suites: HashMap::new(),
            suite_defs: HashMap::new(),
            flow_in_vars: HashMap::new(),
            flow_control_vars: vec![],
            setup_header: vec![],
            setup_footer: vec![],
            flow_header: vec![],
            flow_body: vec![],
            flow_footer: vec![],
        }
    }

    fn push_body(&mut self, line: &str) {
        let ind = 4 + (4 * self.indent);
        if line == "" {
            self.flow_body.push(line.to_string());
        } else {
            self.flow_body
                .push(format!("{:indent$}{}", "", line, indent = ind));
        }
    }
}

pub fn run(ast: &Node<PGM>, output_dir: &Path, model: Model) -> Result<(Model, Vec<PathBuf>)> {
    let mut p = FlowGenerator {
        name: "".to_string(),
        description: None,
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
        file_hier: vec![],
        curr_flow_key: "".to_string(),
        flow_files: BTreeMap::new(),
    };

    let mut i = 0;
    for (_, t) in &p.model.tests {
        let name = format!("tm_{}", i + 1);
        p.test_method_names.insert(t.id, name.clone());
        p.test_methods.insert(name, t.id);
        i += 1;
    }

    // create id look-up table for test suites
    for (_, t) in &p.model.test_invocations {
        p.test_suites
            .insert(t.get("name")?.unwrap().to_string(), t.id);
    }
    println!("ast: {:#?}", ast);
    ast.process(&mut p)?;
    let _ = p.render_flow_files();
    Ok((p.model, p.generated_files))
}

impl FlowGenerator {
    fn current_flow_file(&mut self) -> &mut FlowFile {
        self.flow_files.get_mut(&self.curr_flow_key).unwrap()
    }

    fn flow_file_name(&mut self, flow_name: &str) -> String {
        let mut name = self.file_hier.join("/");
        if !name.is_empty() {
            name.push_str("/");
        }
        name.push_str(flow_name.to_uppercase().as_str());
        name.push_str(".flow");
        name
    }

    fn flow_path(&mut self, flow_name: &str) -> String {
        let root = self.output_dir.parent().unwrap();
        let root = root.file_name().unwrap().to_str().unwrap().to_string();
        let sub_dir = self.output_dir.file_name().unwrap().to_str().unwrap().to_string();
        let mut name = root + "." + &sub_dir;
        if !self.file_hier.is_empty() {  // shouldn't really happen?
            name.push_str(".");
            name.push_str(self.file_hier.join(".").as_str());
        }
        name.push_str(".");
        name.push_str(flow_name.to_uppercase().as_str());
        name
    }

    fn render_flow_files(&mut self) -> Result<()> {
        println!("render_flow_files");
        // Now render the files
        for flow in self.flow_files.values() {
            let flow_file = self.output_dir.join(flow.file.clone());
            let flow_file_dir = flow_file.parent().unwrap();
            std::fs::create_dir_all(&flow_file_dir)?;
            let mut f = std::fs::File::create(&flow_file)?;

            self.generated_files.push(flow_file);

            writeln!(&mut f, "flow {} {{", &flow.id.to_uppercase())?;
            if !flow.flow_in_vars.is_empty() {
                for (var, default) in &flow.flow_in_vars {
                    writeln!(&mut f, "    in {} = {};", var, default)?;
                }
                writeln!(&mut f, "")?;
            }
            if !flow.flow_control_vars.is_empty() {
                // O1 did not sort these, so maintaing that for diffing
                //self.flow_control_vars.sort();
                //self.flow_control_vars.dedup();
                let mut done_flags: HashMap<String, bool> = HashMap::new();
                for var in &flow.flow_control_vars {
                    if !done_flags.contains_key(var) {
                        done_flags.insert(var.to_owned(), true);
                        writeln!(&mut f, "    out {} = -1;", var)?;
                    }
                }
                writeln!(&mut f, "")?;
            }

            writeln!(&mut f, "    setup {{")?;

            let mut sorted_suites = flow.suite_defs.keys().cloned().collect::<Vec<String>>();
            let mut first = true;
            sorted_suites.sort();
            for suite in sorted_suites {
                if !first {
                    writeln!(&mut f, "")?;
                } else {
                    first = false;
                }
                let suite_def = flow.suite_defs.get(&suite).unwrap();
                if suite_def.lines.is_empty() {
                    writeln!(&mut f, "        suite {} calls {} {{ }}", suite, suite_def.klass)?;
                } else {
                    writeln!(&mut f, "        suite {} calls {} {{", suite, suite_def.klass)?;
                    for line in &suite_def.lines {
                        writeln!(&mut f, "            {}", line)?;
                    }
                    writeln!(&mut f, "        }}")?;
                }
            }

            let mut sorted_subflows = flow.subflow_defs.keys().cloned().collect::<Vec<String>>();
            let mut first = true;
            sorted_subflows.sort();
            for subflow in sorted_subflows {
                if first {
                    writeln!(&mut f, "")?;
                    first = false;   
                }
                let subflow_def = flow.subflow_defs.get(&subflow).unwrap();
                if subflow_def.lines.is_empty() {
                    writeln!(&mut f, "        flow {} calls {} {{ }}", subflow, subflow_def.path)?;
                } else {
                    writeln!(&mut f, "        flow {} calls {} {{", subflow, subflow_def.path)?;
                    for line in &subflow_def.lines {
                        writeln!(&mut f, "            {}", line)?;
                    }
                    writeln!(&mut f, "        }}")?;
                }
            }

            writeln!(&mut f, "    }}")?;
            writeln!(&mut f, "")?;

            writeln!(&mut f, "    execute {{")?;
            for line in &flow.flow_header {
                writeln!(&mut f, "{}", line)?;
            }
            for line in &flow.flow_body {
                writeln!(&mut f, "{}", line)?;
            }
            for line in &flow.flow_footer {
                writeln!(&mut f, "{}", line)?;
            }
            writeln!(&mut f, "    }}")?;
            writeln!(&mut f, "}}")?;
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
            PGM::SubFlow(name, _fid) => {
                let orig = self.sub_flow_open;
                self.sub_flow_open = true;

                if self.resources_block {
                    let _ = node.process_children(self);
                } else {
                    let orig_flow = self.curr_flow_key.to_owned();
                    let name = {
                        let parent_flow_file = self.current_flow_file();

                        let name = name.to_uppercase(); // all sub-flows are upper case
                        match &parent_flow_file.subflows.contains_key(&name) {
                            true => {
                                let new_index = parent_flow_file.subflows[&name] + 1;
                                parent_flow_file.subflows.insert(name.to_owned(), new_index.to_owned());
                                format!("{}_{}", name, parent_flow_file.subflows[&name])
                            }
                            false => {
                                parent_flow_file.subflows.insert(name.to_owned(), 0);
                                name
                            }
                        }
                    };

                    let flow_file = FlowFile::new(&name, &self.file_hier);
                    log_debug!("Sub-flow: {}, {}", name, flow_file.file);
                    self.curr_flow_key = flow_file.file.clone();
                    self.flow_files.insert(flow_file.file.clone(), flow_file);
                    
                    self.file_hier.push(name.to_lowercase());
                    let _ = node.process_children(self);
                    self.file_hier.pop();

                    self.curr_flow_key = orig_flow;
                    let mut flow_def = FlowDef { path: self.flow_path(&name).clone(), lines: vec![] };
                    let parent_flow_file = self.current_flow_file();
                    parent_flow_file.subflow_defs.insert(name.to_owned(), flow_def);
                    parent_flow_file.push_body(format!("{}.execute();", name).as_str());
                }

                self.sub_flow_open = orig;
                Return::None
            }
            PGM::Flow(name) => {
                self.name = name.to_owned();
                self.model.select_flow(name)?;

                let flow_file = FlowFile::new(&name, &self.file_hier);
                log_debug!("Flow: {}, file: {}", name, flow_file.file);
                self.curr_flow_key = flow_file.file.clone();
                self.flow_files.insert(flow_file.file.clone(), flow_file);

                // Continue processing the AST
                self.file_hier.push(name.to_lowercase());
                let _ = node.process_children(self);
                self.file_hier.pop();
                Return::None
            }
            PGM::Log(msg) => {
                self.current_flow_file().push_body(&format!("println(\"{}\");", msg));
                Return::None
            }
            PGM::Test(id, _flow_id) => {
                let test = &self.model.test_invocations[id];
                let test_id = test.test_id.unwrap();
                let tm = self.model.tests.get(&test_id).unwrap();
                let (test_name, klass, pattern) = {
                    (
                        test.get("name")?.unwrap().to_string(),
                        tm.class_name.clone().unwrap().to_string(),
                        test.get("pattern")?.map(|p| p.to_string()),
                    )
                };

                let test_name = match &self.flow_files[&self.curr_flow_key].suites.contains_key(&test_name) {
                    true => {
                        let new_index = self.flow_files[&self.curr_flow_key].suites[&test_name] + 1;
                        self.current_flow_file().suites.insert(test_name.to_owned(), new_index.to_owned());
                        format!("{}_{}", test_name, self.current_flow_file().suites[&test_name])
                    }
                    false => {
                        self.current_flow_file().suites.insert(test_name.to_owned(), 0);
                        test_name.to_owned()
                    }
                };

                //let flow = self.current_flow_file();
                let mut suite_def = SuiteDef { klass: klass.clone(), lines: vec![] };
                for (name, kind, value) in self.model.tests.get(&test_id).unwrap().sorted_params() {
                    if let Some(v) = value {
                        suite_def.lines.push(format!("\"{}\" = \"{}\";", name, v));
                    }
                }
                self.current_flow_file().suite_defs.insert(test_name.to_owned(), suite_def);

                if !self.resources_block {
                    self.current_flow_file().push_body(format!("{}.execute();", test_name).as_str());
                }

                Return::None
            }
            PGM::OnFailed(_) => Return::None, // Done manually within the PGMTest handler
            PGM::OnPassed(_) => Return::None, // Done manually within the PGMTest handler
            PGM::Else => Return::None,        // Handled by its parent
            PGM::Condition(cond) => match cond {
                FlowCondition::IfJob(jobs) | FlowCondition::UnlessJob(jobs) => {
                    self.current_flow_file().flow_in_vars.insert("JOB".to_string(), "\"\"".to_string());
                    let mut jobstr = "if (".to_string();
                    let else_node = node.children.iter().find(|n| matches!(n.attrs, PGM::Else));
                    for (i, job) in jobs.iter().enumerate() {
                        if i > 0 {
                            jobstr += " or ";
                        }
                        jobstr += &format!("JOB == \"{}\"", job.to_uppercase())
                    }
                    jobstr += ") {";
                    self.current_flow_file().push_body(&jobstr);
                    self.current_flow_file().indent += 1;
                    if matches!(cond, FlowCondition::IfJob(_)) {
                        node.process_children(self)?;
                    } else {
                        if let Some(else_node) = else_node {
                            else_node.process_children(self)?;
                        }
                    }
                    self.current_flow_file().indent -= 1;
                    self.current_flow_file().push_body("} else {");
                    self.current_flow_file().indent += 1;
                    if matches!(cond, FlowCondition::UnlessJob(_)) {
                        node.process_children(self)?;
                    } else {
                        if let Some(else_node) = else_node {
                            else_node.process_children(self)?;
                        }
                    }
                    self.current_flow_file().indent -= 1;
                    self.current_flow_file().push_body("}");
                    Return::None
                }
                FlowCondition::IfEnable(flags) | FlowCondition::UnlessEnable(flags) => {
                    //let mut flow = self.flow_files.get_mut(&self.current_flow).unwrap();
                    let mut flagstr = "if (".to_string();
                    let else_node = node.children.iter().find(|n| matches!(n.attrs, PGM::Else));
                    for (i, flag) in flags.iter().enumerate() {
                        if i > 0 {
                            flagstr += " or ";
                        }
                        flagstr += &format!("{} == 1", flag.to_uppercase());
                        self.current_flow_file().flow_in_vars.insert(flag.to_uppercase(), "-1".to_string());
                    }
                    flagstr += ") {";
                    self.current_flow_file().push_body(&flagstr);
                    self.current_flow_file().indent += 1;
                    if matches!(cond, FlowCondition::IfEnable(_)) {
                        node.process_children(self)?;
                    } else {
                        if let Some(else_node) = else_node {
                            else_node.process_children(self)?;
                        }
                    }
                    self.current_flow_file().indent -= 1;
                    self.current_flow_file().push_body("} else {");
                    self.current_flow_file().indent += 1;
                    if matches!(cond, FlowCondition::UnlessEnable(_)) {
                        node.process_children(self)?;
                    } else {
                        if let Some(else_node) = else_node {
                            else_node.process_children(self)?;
                        }
                    }
                    self.current_flow_file().indent -= 1;
                    self.current_flow_file().push_body("}");
                    Return::None
                }
                FlowCondition::IfFlag(flags) | FlowCondition::UnlessFlag(flags) => {
                    //let mut flow = self.flow_files.get_mut(&self.current_flow).unwrap();
                    let mut flagstr = "if (".to_string();
                    let else_node = node.children.iter().find(|n| matches!(n.attrs, PGM::Else));
                    for (i, flag) in flags.iter().enumerate() {
                        if i > 0 {
                            flagstr += " or ";
                        }
                        flagstr += &format!("{} == 1", flag.to_uppercase());
                        self.current_flow_file().flow_in_vars.insert(flag.to_uppercase(), "-1".to_string());
                    }
                    flagstr += ") {";
                    self.current_flow_file().push_body(&flagstr);
                    self.current_flow_file().indent += 1;
                    if matches!(cond, FlowCondition::IfFlag(_)) {
                        node.process_children(self)?;
                    } else {
                        if let Some(else_node) = else_node {
                            else_node.process_children(self)?;
                        }
                    }
                    self.current_flow_file().indent -= 1;
                    self.current_flow_file().push_body("} else {");
                    self.current_flow_file().indent += 1;
                    if matches!(cond, FlowCondition::UnlessFlag(_)) {
                        node.process_children(self)?;
                    } else {
                        if let Some(else_node) = else_node {
                            else_node.process_children(self)?;
                        }
                    }
                    self.current_flow_file().indent -= 1;
                    self.current_flow_file().push_body("}");
                    Return::None
                }
                _ => Return::ProcessChildren,
            }
            PGM::SetFlag(flag, state, _is_auto_generated) => {
                let curr_flow = self.current_flow_file();
                if *state {
                    curr_flow.push_body(&format!("{} = 1;", flag.to_uppercase()));
                } else {
                    curr_flow.push_body(&format!("{} = 0;", flag.to_uppercase()));
                }
                curr_flow.flow_control_vars.push(flag.to_uppercase());
                Return::None
            }
            PGM::Bin(bin, _softbin, _kind) => {
                self.current_flow_file().push_body(&format!("addBin({});", bin));
                Return::None
            }
            PGM::Render(text) => {
                self.current_flow_file().push_body(&format!(r#"{}"#, text));
                Return::None
            }
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