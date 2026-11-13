use crate::ast::{Node, Processor, Return};
use crate::prog_gen::advantest::smt8::processors::create_flow_data::FlowData;
use crate::prog_gen::config::SMT8Config;
use crate::prog_gen::{BinType, FlowCondition, GroupType, Model, ParamValue, PGM};
use crate::Result;
use indexmap::IndexMap;
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread::JoinHandle;

/// Writes the flow files on another thread while the flows are generated, creating their
/// directories: file system operations, creating a directory in particular, can be slow
/// compared to generating a file's contents. The queue of files is bounded, so a slow file
/// system holds up the generation instead of the files accumulating in memory. When dropped
/// without finish(), e.g. when the generation panics, it still waits for the queued files, so
/// none is written after the generation has returned.
struct FileWriter {
    sender: Option<mpsc::SyncSender<(PathBuf, Vec<u8>)>>,
    thread: Option<JoinHandle<std::io::Result<()>>>,
}

impl FileWriter {
    /// The number of files which can be waiting to be written
    const QUEUE_LENGTH: usize = 64;

    fn new() -> Self {
        let (sender, receiver) = mpsc::sync_channel::<(PathBuf, Vec<u8>)>(Self::QUEUE_LENGTH);
        let thread = std::thread::spawn(move || -> std::io::Result<()> {
            let mut directories: HashSet<PathBuf> = HashSet::new();
            for (path, contents) in receiver {
                if let Some(directory) = path.parent() {
                    if !directories.contains(directory) {
                        if !directory.exists() {
                            std::fs::create_dir_all(directory)?;
                        }
                        directories.insert(directory.to_owned());
                    }
                }
                std::fs::write(&path, &contents)?;
            }
            Ok(())
        });
        FileWriter {
            sender: Some(sender),
            thread: Some(thread),
        }
    }

    /// Write a file, creating its directory if needed. Any error is returned by `finish()`.
    fn write(&self, path: PathBuf, contents: Vec<u8>) {
        if let Some(sender) = &self.sender {
            // The thread only stops receiving after an error, which finish() returns
            let _ = sender.send((path, contents));
        }
    }

    /// Wait for every file to be written, returning the first error
    fn finish(&mut self) -> Result<()> {
        self.sender = None;
        if let Some(thread) = self.thread.take() {
            match thread.join() {
                Ok(written) => written?,
                Err(panic) => std::panic::resume_unwind(panic),
            }
        }
        Ok(())
    }
}

impl Drop for FileWriter {
    fn drop(&mut self) {
        self.sender = None;
        if let Some(thread) = self.thread.take() {
            // Only reached without finish(), which returns the outcome
            let _ = thread.join();
        }
    }
}

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
    limits_file: Option<BufWriter<std::fs::File>>,
    namespaces: Vec<String>,
    options: SMT8Config,
    writer: FileWriter,
}

/// Contains the data for a .flow file
#[derive(Default)]
struct FlowFile {
    name: String,
    path: PathBuf,
    indent: usize,
    execute_lines: Vec<String>,
    execute_line_buffers: Vec<Vec<String>>,
    render_bins: bool,
    test_ids: Vec<(String, usize)>,
    existing_test_counter: HashMap<String, usize>,
    existing_flow_counter: HashMap<String, usize>,
    sub_flows: Vec<String>,
    /// Sub-flows which are called with bypass enabled
    bypassed_sub_flows: HashSet<String>,
    flow_data: FlowData,
}

impl FlowFile {
    /// Adds a line to the execute section of the flow file, automatically
    /// indenting it appropriately
    fn execute_line(&mut self, line: String) {
        let indent = "    ".repeat(self.indent);
        self.push_execute_line(format!("{}{}", indent, line));
    }

    fn push_execute_line(&mut self, line: String) {
        if let Some(buffer) = self.execute_line_buffers.last_mut() {
            buffer.push(line);
        } else {
            self.execute_lines.push(line);
        }
    }

    fn start_execute_buffer(&mut self) {
        self.execute_line_buffers.push(vec![]);
    }

    fn finish_execute_buffer(&mut self) -> Vec<String> {
        self.execute_line_buffers.pop().unwrap_or_default()
    }

    /// Flushes captured execute lines to the active output target.
    /// If a parent result branch is being buffered, these lines are appended to
    /// that parent buffer rather than directly to the final execute section.
    fn flush_buffered_execute_lines(&mut self, lines: Vec<String>, indent: usize) {
        for line in lines {
            self.push_execute_line(format!("{}{}", "    ".repeat(indent), line));
        }
    }
}

pub fn run(ast: &Node<PGM>, output_dir: &Path, model: Model) -> Result<(Model, Vec<PathBuf>)> {
    // For debugging
    //ast.to_file("smt8_flow.txt")?;

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
        namespaces: vec![],
        options: crate::PROG_GEN_CONFIG.smt8_options(),
        writer: FileWriter::new(),
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
    let generated = ast.process(&mut p);
    // The files are complete once all of them have been written
    let written = p.writer.finish();
    generated?;
    written?;
    if let Some(limits_file) = p.limits_file.as_mut() {
        limits_file.flush()?;
    }
    Ok((p.model, p.generated_files))
}

impl FlowGenerator {
    fn alpha_cmp(a: &str, b: &str) -> Ordering {
        a.to_ascii_lowercase()
            .cmp(&b.to_ascii_lowercase())
            .then_with(|| a.cmp(b))
    }

    fn natural_cmp(a: &str, b: &str) -> Ordering {
        let mut left = a.chars().peekable();
        let mut right = b.chars().peekable();

        loop {
            match (left.peek(), right.peek()) {
                (None, None) => return Ordering::Equal,
                (None, Some(_)) => return Ordering::Less,
                (Some(_), None) => return Ordering::Greater,
                (Some(lc), Some(rc)) => {
                    if lc.is_ascii_digit() && rc.is_ascii_digit() {
                        let mut l_digits = String::new();
                        let mut r_digits = String::new();
                        while let Some(c) = left.peek() {
                            if c.is_ascii_digit() {
                                l_digits.push(*c);
                                left.next();
                            } else {
                                break;
                            }
                        }
                        while let Some(c) = right.peek() {
                            if c.is_ascii_digit() {
                                r_digits.push(*c);
                                right.next();
                            } else {
                                break;
                            }
                        }
                        let l_trimmed = l_digits.trim_start_matches('0');
                        let r_trimmed = r_digits.trim_start_matches('0');
                        let number_cmp = l_trimmed
                            .len()
                            .cmp(&r_trimmed.len())
                            .then_with(|| l_trimmed.cmp(r_trimmed))
                            .then_with(|| l_digits.len().cmp(&r_digits.len()));
                        if number_cmp != Ordering::Equal {
                            return number_cmp;
                        }
                    } else {
                        let l = left.next().unwrap();
                        let r = right.next().unwrap();
                        let char_cmp = l
                            .to_ascii_lowercase()
                            .cmp(&r.to_ascii_lowercase())
                            .then_with(|| l.cmp(&r));
                        if char_cmp != Ordering::Equal {
                            return char_cmp;
                        }
                    }
                }
            }
        }
    }

    fn format_number(value: f64) -> String {
        if value.is_infinite() {
            if value.is_sign_positive() {
                "Infinity".to_string()
            } else {
                "-Infinity".to_string()
            }
        } else {
            format!("{}", value)
        }
    }

    fn format_plain_float(value: f64) -> String {
        if value.is_infinite() {
            Self::format_number(value)
        } else if value.fract() == 0.0 {
            format!("{:.1}", value)
        } else {
            format!("{}", value)
        }
    }

    /// Renders the content of a test's OnPassed or OnFailed branch into a buffer
    fn render_result_branch(
        &mut self,
        branch: &Node<PGM>,
        suppress_bins: bool,
    ) -> Result<Vec<String>> {
        let original_render_bins = {
            let current_flow = self.flow_stack.last_mut().unwrap();
            let original_render_bins = current_flow.render_bins;
            current_flow.start_execute_buffer();
            if suppress_bins {
                current_flow.render_bins = false;
            }
            original_render_bins
        };
        branch.process_children(self)?;
        let current_flow = self.flow_stack.last_mut().unwrap();
        current_flow.render_bins = original_render_bins;
        Ok(current_flow.finish_execute_buffer())
    }

    /// Renders a test's result branches. A test with both passed and failed content renders
    /// them as `if (<test>.pass) { ... } else { ... }`.
    fn render_result_branches(
        &mut self,
        children: &Vec<Box<Node<PGM>>>,
        subject_name: &str,
        suppress_bins: bool,
    ) -> Result<()> {
        let mut passed: Vec<String> = vec![];
        let mut failed: Vec<String> = vec![];
        for branch in children {
            match &branch.attrs {
                PGM::OnPassed(_) => {
                    passed.extend(self.render_result_branch(branch, suppress_bins)?)
                }
                PGM::OnFailed(_) => {
                    failed.extend(self.render_result_branch(branch, suppress_bins)?)
                }
                _ => {}
            }
        }
        let current_flow = self.flow_stack.last_mut().unwrap();
        if !passed.is_empty() {
            current_flow.execute_line(format!("if ({}.pass) {{", subject_name));
            current_flow.flush_buffered_execute_lines(passed, 1);
            if !failed.is_empty() {
                current_flow.execute_line("} else {".to_string());
                current_flow.flush_buffered_execute_lines(failed, 1);
            }
            current_flow.execute_line("}".to_string());
        } else if !failed.is_empty() {
            current_flow.execute_line(format!("if (!{}.pass) {{", subject_name));
            current_flow.flush_buffered_execute_lines(failed, 1);
            current_flow.execute_line("}".to_string());
        }
        Ok(())
    }

    fn write_param_value<W: Write>(
        f: &mut W,
        indent: usize,
        name: &str,
        value: &ParamValue,
    ) -> Result<()> {
        let indent = "    ".repeat(indent);
        match value {
            ParamValue::String(v) | ParamValue::Any(v) => {
                writeln!(f, "{}{} = \"{}\";", indent, name, v)?;
            }
            ParamValue::Class(v) => {
                writeln!(f, "{}{} = {};", indent, name, v)?;
            }
            ParamValue::List(values) => {
                let values = values
                    .iter()
                    .map(Self::format_list_item)
                    .collect::<Result<Vec<String>>>()?
                    .join(", ");
                writeln!(f, "{}{} = #[{}];", indent, name, values)?;
            }
            ParamValue::Int(v) => {
                writeln!(f, "{}{} = {};", indent, name, v)?;
            }
            ParamValue::UInt(v) => {
                writeln!(f, "{}{} = {};", indent, name, v)?;
            }
            ParamValue::Float(v) => {
                writeln!(f, "{}{} = {};", indent, name, Self::format_plain_float(*v))?;
            }
            ParamValue::Current(v) => {
                writeln!(
                    f,
                    "{}{} = \"{}[A]\";",
                    indent,
                    name,
                    Self::format_number(*v)
                )?;
            }
            ParamValue::Voltage(v) => {
                writeln!(
                    f,
                    "{}{} = \"{}[V]\";",
                    indent,
                    name,
                    Self::format_number(*v)
                )?;
            }
            ParamValue::Time(v) => {
                writeln!(
                    f,
                    "{}{} = \"{}[s]\";",
                    indent,
                    name,
                    Self::format_number(*v)
                )?;
            }
            ParamValue::Frequency(v) => {
                writeln!(
                    f,
                    "{}{} = \"{}[Hz]\";",
                    indent,
                    name,
                    Self::format_number(*v)
                )?;
            }
            ParamValue::Bool(v) => {
                writeln!(
                    f,
                    "{}{} = {};",
                    indent,
                    name,
                    if *v { "true" } else { "false" }
                )?;
            }
        }
        Ok(())
    }

    fn format_list_item(value: &ParamValue) -> Result<String> {
        Ok(match value {
            ParamValue::String(v) | ParamValue::Any(v) => format!("\"{}\"", v),
            ParamValue::Class(v) => v.clone(),
            ParamValue::Int(v) => v.to_string(),
            ParamValue::UInt(v) => v.to_string(),
            ParamValue::Float(v) => Self::format_plain_float(*v),
            ParamValue::Current(v) => format!("\"{}[A]\"", Self::format_number(*v)),
            ParamValue::Voltage(v) => format!("\"{}[V]\"", Self::format_number(*v)),
            ParamValue::Time(v) => format!("\"{}[s]\"", Self::format_number(*v)),
            ParamValue::Frequency(v) => format!("\"{}[Hz]\"", Self::format_number(*v)),
            ParamValue::Bool(v) => if *v { "true" } else { "false" }.to_string(),
            ParamValue::List(_) => bail!("Nested parameter lists cannot be rendered"),
        })
    }

    fn render_sorted_contents<W: Write>(
        &self,
        f: &mut W,
        indent: usize,
        values: &IndexMap<String, ParamValue>,
        default_values: &IndexMap<String, ParamValue>,
        collections: &IndexMap<String, Vec<usize>>,
    ) -> Result<()> {
        let mut names: Vec<String> = values
            .iter()
            .filter_map(|(name, value)| {
                if !self.options.render_default_tmparams
                    && default_values
                        .get(name)
                        .is_some_and(|default| default == value)
                {
                    None
                } else {
                    Some(name.clone())
                }
            })
            .collect();
        for (collection_name, ids) in collections {
            let has_renderable_items = ids.iter().any(|id| {
                self.model
                    .test_collection_items
                    .get(id)
                    .is_some_and(|item| item.available)
            });
            if has_renderable_items {
                names.push(collection_name.clone());
            }
        }
        names.sort_by(|a, b| Self::alpha_cmp(a, b));
        names.dedup();

        for name in names {
            if let Some(value) = values.get(&name) {
                if !self.options.render_default_tmparams
                    && default_values
                        .get(&name)
                        .is_some_and(|default| default == value)
                {
                    continue;
                }
                Self::write_param_value(f, indent, &name, value)?;
                continue;
            }

            if let Some(ids) = collections.get(&name) {
                let mut items = ids
                    .iter()
                    .filter_map(|id| self.model.test_collection_items.get(id))
                    .filter(|item| item.available)
                    .collect::<Vec<_>>();
                items.sort_by(|a, b| Self::natural_cmp(&a.instance_id, &b.instance_id));
                for item in items {
                    let indent_str = "    ".repeat(indent);
                    writeln!(
                        f,
                        "{}{}[{}] = {{",
                        indent_str, item.collection_name, item.instance_id
                    )?;
                    self.render_sorted_contents(
                        f,
                        indent + 1,
                        &item.values,
                        &item.default_values,
                        &item.collections,
                    )?;
                    writeln!(f, "{}}};", indent_str)?;
                }
            }
        }
        Ok(())
    }

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
        // Create a clean upper case name where each run of spaces is an underscore, keeping any
        // underscores of the name itself, e.g. a MAIN__RETEST sub-flow
        let mut name = name
            .split_whitespace()
            .collect::<Vec<_>>()
            .join("_")
            .to_uppercase();
        if let Some(current_flow) = self.flow_stack.last_mut() {
            if current_flow.existing_flow_counter.contains_key(&name) {
                let count = current_flow.existing_flow_counter.get(&name).unwrap() + 1;
                current_flow
                    .existing_flow_counter
                    .insert(name.to_owned(), count);
                name = format!("{}_{}", name, count);
            } else {
                current_flow
                    .existing_flow_counter
                    .insert(name.to_owned(), 0);
            }
        }
        let flow_path = match self.flow_stack.last() {
            // The directory is created when the file is written, see FileWriter
            Some(f) => f
                .path
                .parent()
                .unwrap()
                .join(f.name.to_lowercase())
                .join(format!("{}.flow", name)),
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

    fn close_flow_file(&mut self, namespace: Option<String>) -> Result<FlowFile> {
        let flow_file = self.flow_stack.pop().unwrap();
        let namespace = {
            match namespace {
                Some(n) => format!("{}.", n),
                None => "".to_string(),
            }
        };
        // Written by self.writer once complete
        let mut f: Vec<u8> = Vec::new();
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
                writeln!(
                    &mut f,
                    "        suite {} calls {} {{",
                    test_name,
                    test.class_name.as_ref().unwrap()
                )?;
                if let Some(pattern) = test_invocation.get("pattern")?.map(|p| p.to_string()) {
                    writeln!(
                        &mut f,
                        "            measurement.pattern = setupRef({}patterns.{});",
                        &namespace, pattern
                    )?;
                }
                if let Some(spec) = test_invocation.get("spec")?.map(|p| p.to_string()) {
                    writeln!(
                        &mut f,
                        "            measurement.specification = setupRef({}specs.{});",
                        &namespace, spec
                    )?;
                }
                self.render_sorted_contents(
                    &mut f,
                    3,
                    &test.values,
                    &test.definition.default_values,
                    &test.collections,
                )?;
                writeln!(&mut f, "        }}")?;
            }
            writeln!(&mut f, "")?;
        }
        for sub_flow in &flow_file.sub_flows {
            let relative_path = flow_file
                .path
                .strip_prefix(&self.output_dir)
                .unwrap()
                .parent()
                .unwrap()
                .join(flow_file.name.to_lowercase());
            let call = format!(
                "        flow {} calls {}flows.{}.{} {{",
                sub_flow,
                &namespace,
                relative_path
                    .to_str()
                    .unwrap()
                    .replace("\\", ".")
                    .replace("/", "."),
                sub_flow
            );
            if flow_file.bypassed_sub_flows.contains(sub_flow) {
                writeln!(&mut f, "{}", call)?;
                writeln!(&mut f, "            bypass = true;")?;
                writeln!(&mut f, "        }}")?;
            } else {
                writeln!(&mut f, "{} }}", call)?;
            }
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
        self.writer.write(flow_file.path.clone(), f);
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
            PGM::Namespace(namespace) => {
                self.namespaces.push(namespace.to_owned());
                Return::None
            }
            PGM::Flow(name) => {
                log_debug!("Rendering flow '{}' for V93k SMT8", name);

                if self.options.create_limits_file {
                    let limits_dir = self.output_dir.parent().unwrap().join("limits");
                    if !limits_dir.exists() {
                        std::fs::create_dir_all(&limits_dir)?;
                    }
                    let limits_file = limits_dir.join(format!(
                        "Main.{}_Tests.csv",
                        name.replace(" ", "_").to_uppercase()
                    ));

                    let file = std::fs::File::create(&limits_file)?;
                    let mut f = BufWriter::new(file);
                    self.generated_files.push(limits_file.clone());
                    writeln!(
                        &mut f,
                        "Test Suite,Test,Test Number,Test Text,Low Limit,High Limit,Unit,Soft Bin"
                    )?;
                    writeln!(&mut f, ",,,,default,default")?;
                    self.limits_file = Some(f);
                }

                self.name = name.to_owned();
                self.model.select_flow(name)?;
                let flow_data = if let PGM::FlowData(fdata) = &node.children[0].attrs {
                    (**fdata).clone()
                } else {
                    FlowData::default()
                };
                self.open_flow_file(name, flow_data)?;
                node.process_children(self)?;
                self.close_flow_file(self.namespaces.last().cloned())?;

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
                    (**fdata).clone()
                } else {
                    FlowData::default()
                };
                self.open_flow_file(name, flow_data)?;
                let orig = self.sub_flow_open;
                self.sub_flow_open = true;
                // Bypass applies to the call of this flow, not to the flows that it calls
                let orig_bypass = self.bypass_sub_flows;
                self.bypass_sub_flows = false;
                node.process_children(self)?;
                self.bypass_sub_flows = orig_bypass;
                self.sub_flow_open = orig;
                let flow = self.close_flow_file(self.namespaces.last().cloned())?;
                let current_flow = self.flow_stack.last_mut().unwrap();
                for v in flow.flow_data.sorted_input_vars() {
                    current_flow.execute_line(format!("{}.{} = {};", flow.name, v, v));
                }
                current_flow.execute_line(format!("{}.execute();", flow.name));
                current_flow.sub_flows.push(flow.name.clone());
                if self.bypass_sub_flows {
                    current_flow.bypassed_sub_flows.insert(flow.name.clone());
                }
                for v in flow.flow_data.sorted_output_flags() {
                    if current_flow.flow_data.output_flags.contains(&v)
                        || current_flow.flow_data.referenced_flags.contains(&v)
                    {
                        current_flow.execute_line(format!("{} = {}.{};", v, flow.name, v));
                    }
                }
                Return::None
            }
            PGM::Group(name, _, kind, _) => {
                if kind == &GroupType::Flow {
                    log_debug!("Rendering group '{}'", name);
                    let flow_data = if let PGM::FlowData(fdata) = &node.children[0].attrs {
                        (**fdata).clone()
                    } else {
                        FlowData::default()
                    };
                    self.open_flow_file(name, flow_data)?;
                    // Bypass applies to the call of this flow, not to the flows that it calls
                    let orig_bypass = self.bypass_sub_flows;
                    self.bypass_sub_flows = false;
                    node.process_children(self)?;
                    self.bypass_sub_flows = orig_bypass;
                    let flow = self.close_flow_file(self.namespaces.last().cloned())?;
                    let current_flow = self.flow_stack.last_mut().unwrap();
                    for v in flow.flow_data.sorted_input_vars() {
                        current_flow.execute_line(format!("{}.{} = {};", flow.name, v, v));
                    }
                    current_flow.execute_line(format!("{}.execute();", flow.name));
                    current_flow.sub_flows.push(flow.name.clone());
                    if self.bypass_sub_flows {
                        current_flow.bypassed_sub_flows.insert(flow.name.clone());
                    }
                    for v in flow.flow_data.sorted_output_flags() {
                        if current_flow.flow_data.output_flags.contains(&v)
                            || current_flow.flow_data.referenced_flags.contains(&v)
                        {
                            current_flow.execute_line(format!("{} = {}.{};", v, flow.name, v));
                        }
                    }
                    self.render_result_branches(&node.children, &flow.name, false)?;
                } else {
                    node.process_children(self)?;
                }
                Return::None
            }
            PGM::Log(msg) => {
                self.flow_stack
                    .last_mut()
                    .unwrap()
                    .execute_line(format!("println(\"{}\");", msg));
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
                                current_flow
                                    .existing_test_counter
                                    .get(&orig_test_name)
                                    .unwrap()
                            );
                            let count = current_flow
                                .existing_test_counter
                                .get(&orig_test_name)
                                .unwrap()
                                + 1;
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
                            test_invocation.tname.clone(),
                        )
                    }
                };
                if !self.resources_block && self.options.create_limits_file {
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
                    self.flow_stack
                        .last_mut()
                        .unwrap()
                        .execute_line(format!("{}.execute();", &test_name));
                    self.render_result_branches(&node.children, &test_name, true)?;
                }
                Return::ProcessChildren
            }
            PGM::TestStr(name, _flow_id, _bin, softbin, number) => {
                if !self.resources_block && self.options.create_limits_file {
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
                self.flow_stack
                    .last_mut()
                    .unwrap()
                    .execute_line(format!("{}.execute();", name));
                self.render_result_branches(&node.children, name, true)?;
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
                            jobs.iter()
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
                FlowCondition::IfExpr(expression) | FlowCondition::UnlessExpr(expression) => {
                    let else_node = node.children.iter().find(|n| matches!(n.attrs, PGM::Else));
                    {
                        let current_flow = self.flow_stack.last_mut().unwrap();
                        current_flow.execute_line(format!("if ({}) {{", expression));
                        current_flow.indent += 1;
                    }
                    if matches!(cond, FlowCondition::IfExpr(_)) {
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
                    if matches!(cond, FlowCondition::UnlessExpr(_)) {
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
            PGM::Loop(count, variable, descending) => {
                let count = match count {
                    Some(c) => *c,
                    None => bail!("V93K SMT8 flow generation only supports loops with a fixed iteration count"),
                };
                let variable = variable.clone().unwrap_or_else(|| "index".to_string());
                {
                    let current_flow = self.flow_stack.last_mut().unwrap();
                    if *descending {
                        current_flow.execute_line(format!("for ({} : {}..1) {{", variable, count));
                    } else {
                        current_flow.execute_line(format!("for ({} : 1..{}) {{", variable, count));
                    }
                    current_flow.indent += 1;
                }
                node.process_children(self)?;
                let current_flow = self.flow_stack.last_mut().unwrap();
                current_flow.indent -= 1;
                current_flow.execute_line("}".to_string());
                Return::None
            }
            PGM::SetVariable(name, value) => {
                let current_flow = self.flow_stack.last_mut().unwrap();
                current_flow.execute_line(format!("{} = {};", name, value));
                Return::None
            }
            // Only declares the variable, see create_flow_data
            PGM::OutputVariable(_) => Return::None,
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
                            current_flow
                                .execute_line(format!("addBin({});", softbin.unwrap_or(*bin)));
                        }
                    }
                    BinType::Good => {
                        current_flow.execute_line(format!("addBin({});", softbin.unwrap_or(*bin)));
                    }
                };
                Return::None
            }
            PGM::Render(text) => {
                let current_flow = self.flow_stack.last_mut().unwrap();
                // Use split to preserve empty and trailing lines in the rendered text.
                for line in text.split('\n') {
                    current_flow.execute_line(line.to_owned());
                }
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

    fn on_processed_node(&mut self, node: &Node<PGM>) -> Result<Return<PGM>> {
        match &node.attrs {
            PGM::Namespace(_) => {
                self.namespaces.pop();
            }
            _ => {}
        }
        // Rendering consumes the input AST for side effects only. Dropping each
        // processed node avoids rebuilding and then discarding a duplicate tree.
        Ok(Return::None)
    }
}

#[cfg(test)]
mod tests {
    use super::{run, FileWriter, FlowGenerator};
    use crate::prog_gen::{
        process_flow, FlowCondition, FlowID, GroupType, Model, ParamValue, SupportedTester, PGM,
    };
    use std::cmp::Ordering;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn files_are_written_with_their_directories() -> crate::Result<()> {
        let dir = tempdir()?;
        let mut writer = FileWriter::new();
        let first = dir.path().join("a").join("b").join("FIRST.flow");
        let second = dir.path().join("a").join("SECOND.flow");
        writer.write(first.clone(), b"first\n".to_vec());
        writer.write(second.clone(), b"second\n".to_vec());
        // The last write of a file wins, as when written in order
        writer.write(second.clone(), b"second again\n".to_vec());
        writer.finish()?;
        assert_eq!(fs::read_to_string(&first)?, "first\n");
        assert_eq!(fs::read_to_string(&second)?, "second again\n");
        Ok(())
    }

    #[test]
    fn file_write_errors_are_returned_when_finished() -> crate::Result<()> {
        let dir = tempdir()?;
        let blocking_file = dir.path().join("not_a_directory");
        fs::write(&blocking_file, "")?;
        let mut writer = FileWriter::new();
        writer.write(blocking_file.join("FLOW.flow"), b"flow\n".to_vec());
        // Writes after an error are not made
        writer.write(dir.path().join("LATER.flow"), b"later\n".to_vec());
        assert!(writer.finish().is_err());
        assert!(!dir.path().join("LATER.flow").exists());
        // Finishing again is fine
        writer.finish()?;
        Ok(())
    }

    /// Makes a FIFO, a write to it is held up until it is read
    #[cfg(unix)]
    fn fifo(path: &std::path::Path) {
        assert!(std::process::Command::new("mkfifo")
            .arg(path)
            .status()
            .unwrap()
            .success());
    }

    #[cfg(unix)]
    #[test]
    fn a_held_up_file_system_holds_up_the_generation() -> crate::Result<()> {
        use std::sync::atomic::{AtomicBool, Ordering};
        let dir = tempdir()?;
        let gate = dir.path().join("GATE.flow");
        fifo(&gate);
        let queued = AtomicBool::new(false);
        std::thread::scope(|scope| -> crate::Result<()> {
            let generation = scope.spawn(|| -> crate::Result<()> {
                let mut writer = FileWriter::new();
                writer.write(gate.clone(), b"gate\n".to_vec());
                for i in 0..FileWriter::QUEUE_LENGTH + 2 {
                    writer.write(dir.path().join(format!("{}.flow", i)), vec![]);
                }
                queued.store(true, Ordering::SeqCst);
                writer.finish()
            });
            std::thread::sleep(std::time::Duration::from_millis(200));
            let queued_while_held_up = queued.load(Ordering::SeqCst);
            assert_eq!(fs::read_to_string(&gate)?, "gate\n");
            generation.join().unwrap()?;
            assert!(!queued_while_held_up);
            assert!(queued.load(Ordering::SeqCst));
            Ok(())
        })
    }

    #[cfg(unix)]
    #[test]
    fn the_files_are_written_before_an_unfinished_writer_is_dropped() -> crate::Result<()> {
        let dir = tempdir()?;
        let gate = dir.path().join("GATE.flow");
        fifo(&gate);
        let late = dir.path().join("LATE.flow");
        let reader = {
            let gate = gate.clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(200));
                fs::read(&gate).unwrap()
            })
        };
        let writer = FileWriter::new();
        writer.write(gate.clone(), b"gate\n".to_vec());
        writer.write(late.clone(), b"late\n".to_vec());
        // As when the generation panics
        drop(writer);
        assert!(late.exists());
        assert_eq!(reader.join().unwrap(), b"gate\n");
        Ok(())
    }

    /// Renders the given flow AST for SMT8 and returns the content of the named flow file
    fn render_flow_file(flow: crate::ast::Node<PGM>, file_name: &str) -> crate::Result<String> {
        let mut flow_ast = crate::ast::AST::new();
        flow_ast.start(flow);
        let (ast, model) = process_flow(
            &flow_ast,
            Model::new(SupportedTester::V93KSMT8),
            SupportedTester::V93KSMT8,
            true,
        )?;
        let output_dir = tempdir()?;
        let (_model, files) = run(&ast, output_dir.path(), model)?;
        let flow_path = files
            .iter()
            .find(|path| path.file_name().and_then(|name| name.to_str()) == Some(file_name))
            .expect("expected generated SMT8 flow file");
        Ok(fs::read_to_string(flow_path)?)
    }

    #[test]
    fn loops_are_rendered_ascending_and_descending() -> crate::Result<()> {
        let flow = node!(PGM::Flow, "loops".to_string() =>
            node!(PGM::Loop, Some(3), Some("attempt".to_string()), false =>
                node!(PGM::TestStr, "looped".to_string(), FlowID::from_str("looped"), None, None, None)
            ),
            node!(PGM::Loop, Some(9), None, true =>
                node!(PGM::TestStr, "countdown".to_string(), FlowID::from_str("countdown"), None, None, None)
            )
        );
        let flow = render_flow_file(flow, "LOOPS.flow")?;

        assert!(flow.contains("for (attempt : 1..3) {\n            looped.execute();\n        }"));
        assert!(flow.contains("for (index : 9..1) {\n            countdown.execute();\n        }"));
        // Loop counters are local to the flow, not inputs
        assert!(!flow.contains("in attempt"));
        assert!(!flow.contains("in index"));
        Ok(())
    }

    #[test]
    fn a_failure_tested_by_several_conditions_is_flagged_once() -> crate::Result<()> {
        let t = |name: &str| {
            node!(
                PGM::TestStr,
                name.to_string(),
                FlowID::from_str(name),
                None,
                None,
                None
            )
        };
        let failed = |name: &str| FlowCondition::IfFailed(vec![FlowID::from_str(name)]);
        let flow = node!(PGM::Flow, "flagged".to_string() =>
            t("t_first"),
            node!(PGM::Condition, failed("t_first") =>
                t("t_second"),
                node!(PGM::Condition, failed("t_second") => t("t_second_check"))
            ),
            node!(PGM::Condition, failed("t_second") => t("t_after_second"))
        );
        let flow = render_flow_file(flow, "FLAGGED.flow")?;
        assert_eq!(flow.matches("T_SECOND_FAILED = 1;").count(), 1, "{}", flow);
        assert_eq!(flow.matches("T_FIRST_FAILED = 1;").count(), 1, "{}", flow);
        Ok(())
    }

    #[test]
    fn expression_conditions_are_rendered_and_their_variables_input() -> crate::Result<()> {
        let flow = node!(PGM::Flow, "expressions".to_string() =>
            node!(PGM::Condition, FlowCondition::IfExpr("mode == 6".to_string()) =>
                node!(PGM::TestStr, "when_6".to_string(), FlowID::from_str("when_6"), None, None, None)
            ),
            node!(PGM::Condition, FlowCondition::UnlessExpr("mode == 6".to_string()) =>
                node!(PGM::TestStr, "unless_6".to_string(), FlowID::from_str("unless_6"), None, None, None)
            )
        );
        let flow = render_flow_file(flow, "EXPRESSIONS.flow")?;

        assert!(flow.contains("in mode = -1;"));
        // Like flag conditions, adjacent opposite expressions are combined into an if/else
        assert!(flow.contains(
            "if (mode == 6) {\n            when_6.execute();\n        } else {\n            unless_6.execute();\n        }"
        ));
        Ok(())
    }

    #[test]
    fn expression_conditions_render_both_branches() -> crate::Result<()> {
        let flow = node!(PGM::Flow, "branches".to_string() =>
            node!(PGM::Condition, FlowCondition::IfExpr("mode == 6".to_string()) =>
                node!(PGM::TestStr, "when_6".to_string(), FlowID::from_str("when_6"), None, None, None)
            ),
            node!(PGM::TestStr, "between".to_string(), FlowID::from_str("between"), None, None, None),
            node!(PGM::Condition, FlowCondition::UnlessExpr("mode == 6".to_string()) =>
                node!(PGM::TestStr, "unless_6".to_string(), FlowID::from_str("unless_6"), None, None, None)
            )
        );
        let flow = render_flow_file(flow, "BRANCHES.flow")?;

        // Like flag conditions, an expression condition always renders both branches
        assert!(flow.contains(
            "if (mode == 6) {\n            when_6.execute();\n        } else {\n        }"
        ));
        assert!(flow.contains(
            "if (mode == 6) {\n        } else {\n            unless_6.execute();\n        }"
        ));
        Ok(())
    }

    #[test]
    fn flags_read_by_expressions_are_not_input_variables() -> crate::Result<()> {
        // The flag is set before and after the expression that reads it
        for set_first in [true, false] {
            let set_flag = node!(PGM::SetFlag, "RETRY".to_string(), true, false);
            let condition = node!(PGM::Condition, FlowCondition::IfExpr("RETRY == 1".to_string()) =>
                node!(PGM::TestStr, "retry".to_string(), FlowID::from_str("retry"), None, None, None)
            );
            let (first, second) = if set_first {
                (set_flag, condition)
            } else {
                (condition, set_flag)
            };
            let flow = node!(PGM::Flow, "flags".to_string() => first, second);
            let flow = render_flow_file(flow, "FLAGS.flow")?;

            assert!(flow.contains("out RETRY = -1;"), "{}", flow);
            assert!(!flow.contains("in RETRY"), "{}", flow);
        }
        Ok(())
    }

    #[test]
    fn variables_are_assigned_and_passed_down() -> crate::Result<()> {
        let flow = node!(PGM::Flow, "variables".to_string() =>
            node!(PGM::SetVariable, "per_tile_passed".to_string(), "0".to_string()),
            node!(PGM::Group, "inner".to_string(), None, GroupType::Flow, Some(FlowID::from_str("inner")) =>
                node!(PGM::Loop, Some(9), Some("per_tile_index".to_string()), true =>
                    node!(PGM::Condition, FlowCondition::IfExpr("per_tile_passed == 0 && per_tile_index > 1".to_string()) =>
                        node!(PGM::SetVariable, "per_tile_passed".to_string(), "1".to_string())
                    )
                )
            )
        );
        let top = render_flow_file(flow.clone(), "VARIABLES.flow")?;
        assert!(top.contains("in per_tile_passed = -1;"));
        assert!(top.contains("per_tile_passed = 0;"));
        assert!(top.contains("INNER.per_tile_passed = per_tile_passed;\n        INNER.execute();"));

        let inner = render_flow_file(flow, "INNER.flow")?;
        assert!(inner.contains("in per_tile_passed = -1;"));
        assert!(!inner.contains("in per_tile_index"));
        assert!(inner.contains("for (per_tile_index : 9..1) {"));
        assert!(inner.contains("if (per_tile_passed == 0 && per_tile_index > 1) {"));
        assert!(inner.contains("per_tile_passed = 1;"));
        Ok(())
    }

    #[test]
    fn declared_output_variables_keep_their_case_and_are_passed_up() -> crate::Result<()> {
        let flow = node!(PGM::Flow, "outputs".to_string() =>
            node!(PGM::SubFlow, "sub".to_string(), None =>
                node!(PGM::TestStr, "set_params".to_string(), FlowID::from_str("set_params"), None, None, None),
                node!(PGM::Condition, FlowCondition::IfExpr("failCaptureModeVarStd == 6".to_string()) =>
                    node!(PGM::TestStr, "std".to_string(), FlowID::from_str("std"), None, None, None)
                ),
                node!(PGM::OutputVariable, "failCaptureModeVarStd".to_string())
            ),
            node!(PGM::Group, "grp".to_string(), None, GroupType::Flow, Some(FlowID::from_str("grp")) =>
                node!(PGM::OutputVariable, "BITMAP".to_string())
            ),
            node!(PGM::Group, "reader".to_string(), None, GroupType::Flow, Some(FlowID::from_str("reader")) =>
                node!(PGM::Condition, FlowCondition::IfExpr("failCaptureModeVarStd == 6".to_string()) =>
                    node!(PGM::TestStr, "reads".to_string(), FlowID::from_str("reads"), None, None, None)
                )
            )
        );

        // Declared, initialized and read in the declaring flow, but never assigned
        let sub = render_flow_file(flow.clone(), "SUB.flow")?;
        assert!(
            sub.contains("    out failCaptureModeVarStd = -1;"),
            "{}",
            sub
        );
        assert!(!sub.contains("in failCaptureModeVarStd"), "{}", sub);
        assert!(
            sub.contains("        failCaptureModeVarStd = -1;"),
            "{}",
            sub
        );
        assert!(sub.contains("if (failCaptureModeVarStd == 6) {"), "{}", sub);
        assert_eq!(
            sub.matches("failCaptureModeVarStd = ").count(),
            2,
            "{}",
            sub
        );

        let grp = render_flow_file(flow.clone(), "GRP.flow")?;
        assert!(grp.contains("    out BITMAP = -1;"), "{}", grp);
        assert_eq!(grp.matches("BITMAP = ").count(), 2, "{}", grp);

        // Passed up from the declaring flows, and down to a flow that reads it
        let top = render_flow_file(flow.clone(), "OUTPUTS.flow")?;
        assert!(
            top.contains("    out failCaptureModeVarStd = -1;"),
            "{}",
            top
        );
        assert!(top.contains("    out BITMAP = -1;"), "{}", top);
        assert!(
            top.contains("failCaptureModeVarStd = SUB.failCaptureModeVarStd;"),
            "{}",
            top
        );
        assert!(top.contains("BITMAP = GRP.BITMAP;"), "{}", top);
        assert!(
            top.contains("READER.failCaptureModeVarStd = failCaptureModeVarStd;"),
            "{}",
            top
        );
        assert!(!top.contains("FAILCAPTUREMODEVARSTD"), "{}", top);

        let reader = render_flow_file(flow, "READER.flow")?;
        assert!(
            reader.contains("    in failCaptureModeVarStd = -1;"),
            "{}",
            reader
        );
        Ok(())
    }

    #[test]
    fn declared_output_variables_read_as_flags_are_outputs() -> crate::Result<()> {
        let flow = node!(PGM::Flow, "control".to_string() =>
            node!(PGM::TestStr, "set_params".to_string(), FlowID::from_str("set_params"), None, None, None),
            node!(PGM::Condition, FlowCondition::UnlessFlag(vec!["DEBUG".to_string()]) =>
                node!(PGM::TestStr, "not_debug".to_string(), FlowID::from_str("not_debug"), None, None, None)
            ),
            node!(PGM::OutputVariable, "DEBUG".to_string()),
            // Declared and also set by the user, a single output
            node!(PGM::OutputVariable, "RETRY".to_string()),
            node!(PGM::SetFlag, "retry".to_string(), true, false)
        );
        let flow = render_flow_file(flow, "CONTROL.flow")?;
        assert!(flow.contains("    out DEBUG = -1;"), "{}", flow);
        assert!(!flow.contains("in DEBUG"), "{}", flow);
        assert_eq!(flow.matches("out RETRY = -1;").count(), 1, "{}", flow);
        assert!(flow.contains("RETRY = 1;"), "{}", flow);
        Ok(())
    }

    #[test]
    fn declared_output_variables_are_output_when_a_generated_flag_has_their_name(
    ) -> crate::Result<()> {
        // T1_RAN is also the flag that Origen generates for an if_ran("t1") condition
        let flow = node!(PGM::Flow, "outer".to_string() =>
            node!(PGM::SubFlow, "sub".to_string(), None =>
                node!(PGM::TestStr, "t1".to_string(), FlowID::from_str("t1"), None, None, None),
                node!(PGM::SetFlag, "T1_RAN".to_string(), true, true),
                node!(PGM::Condition, FlowCondition::IfFlag(vec!["T1_RAN".to_string()]) =>
                    node!(PGM::TestStr, "t2".to_string(), FlowID::from_str("t2"), None, None, None)
                ),
                node!(PGM::OutputVariable, "T1_RAN".to_string())
            )
        );
        let sub = render_flow_file(flow.clone(), "SUB.flow")?;
        assert!(sub.contains("    out T1_RAN = -1;"), "{}", sub);
        let outer = render_flow_file(flow, "OUTER.flow")?;
        assert!(outer.contains("    out T1_RAN = -1;"), "{}", outer);
        assert!(outer.contains("T1_RAN = SUB.T1_RAN;"), "{}", outer);
        Ok(())
    }

    #[test]
    fn bypassed_groups_are_called_with_bypass() -> crate::Result<()> {
        let flow = node!(PGM::Flow, "bypassing".to_string() =>
            node!(PGM::BypassSubFlows =>
                node!(PGM::Group, "skipped".to_string(), None, GroupType::Flow, Some(FlowID::from_str("skipped")) =>
                    node!(PGM::Group, "nested".to_string(), None, GroupType::Flow, Some(FlowID::from_str("nested")) =>
                        node!(PGM::TestStr, "in_nested".to_string(), FlowID::from_str("in_nested"), None, None, None)
                    )
                )
            ),
            node!(PGM::Group, "kept".to_string(), None, GroupType::Flow, Some(FlowID::from_str("kept")) =>
                node!(PGM::TestStr, "in_kept".to_string(), FlowID::from_str("in_kept"), None, None, None)
            )
        );
        let top = render_flow_file(flow.clone(), "BYPASSING.flow")?;
        assert!(top.contains("flow SKIPPED calls"));
        assert!(top.contains(".SKIPPED {\n            bypass = true;\n        }"));
        assert!(top.contains(".KEPT { }"));

        // Bypass applies only to the call of the bypassed flow, not to the flows it calls
        let skipped = render_flow_file(flow, "SKIPPED.flow")?;
        assert!(skipped.contains(".NESTED { }"));
        assert!(!skipped.contains("bypass = true;"));
        Ok(())
    }

    #[test]
    fn natural_sort_orders_numeric_suffixes() {
        let mut ids = vec!["param10", "param2", "param1", "param11", "param3"];
        ids.sort_by(|a, b| FlowGenerator::natural_cmp(a, b));
        assert_eq!(
            ids,
            vec!["param1", "param2", "param3", "param10", "param11"]
        );
    }

    #[test]
    fn formats_smt8_list_items_by_scalar_type() {
        let values = vec![
            ParamValue::String("SLC, SDA".to_string()),
            ParamValue::Class("DEFECT_SCREEN".to_string()),
            ParamValue::Bool(true),
            ParamValue::Int(3),
        ];
        assert_eq!(
            values
                .iter()
                .map(FlowGenerator::format_list_item)
                .collect::<crate::Result<Vec<String>>>()
                .unwrap(),
            vec!["\"SLC, SDA\"", "DEFECT_SCREEN", "true", "3"]
        );
    }

    #[test]
    fn writes_class_and_list_values_with_smt8_syntax() -> crate::Result<()> {
        let output_dir = tempdir()?;
        let path = output_dir.path().join("params.txt");
        let mut file = std::fs::File::create(&path)?;
        FlowGenerator::write_param_value(
            &mut file,
            0,
            "decodingType",
            &ParamValue::Class("SMU14".to_string()),
        )?;
        FlowGenerator::write_param_value(
            &mut file,
            0,
            "categories",
            &ParamValue::List(vec![ParamValue::Class("DEFECT_SCREEN".to_string())]),
        )?;
        drop(file);
        assert_eq!(
            fs::read_to_string(path)?,
            "decodingType = SMU14;\ncategories = #[DEFECT_SCREEN];\n"
        );
        Ok(())
    }

    #[test]
    fn rejects_nested_lists_instead_of_panicking() -> crate::Result<()> {
        let output_dir = tempdir()?;
        let path = output_dir.path().join("params.txt");
        let mut file = std::fs::File::create(path)?;
        let error = FlowGenerator::write_param_value(
            &mut file,
            0,
            "nested",
            &ParamValue::List(vec![ParamValue::List(vec![])]),
        )
        .unwrap_err();
        assert!(error.to_string().contains("Nested parameter lists"));
        Ok(())
    }

    #[test]
    fn alpha_sort_is_case_insensitive() {
        assert_eq!(
            FlowGenerator::alpha_cmp("testName", "testerState"),
            Ordering::Greater
        );
        assert_eq!(
            FlowGenerator::alpha_cmp("Y1Variable", "zDataDeltaLimit"),
            Ordering::Less
        );
    }

    #[test]
    fn conditional_render_is_emitted() -> crate::Result<()> {
        let flow = node!(PGM::Flow, "conditional_render".to_string() =>
            node!(PGM::Render, "initialize();".to_string()),
            node!(PGM::TestStr, "test1".to_string(), FlowID::from_str("test1"), None, None, None),
            node!(PGM::Condition, FlowCondition::IfFailed(vec![FlowID::from_str("test1")]) =>
                node!(PGM::Render, "stop();".to_string())
            )
        );

        let mut flow_ast = crate::ast::AST::new();
        flow_ast.start(flow);
        let (ast, model) = process_flow(
            &flow_ast,
            Model::new(SupportedTester::V93KSMT8),
            SupportedTester::V93KSMT8,
            true,
        )?;
        let output_dir = tempdir()?;
        let (_model, files) = run(&ast, output_dir.path(), model)?;
        let flow_path = files
            .iter()
            .find(|path| {
                path.file_name().and_then(|name| name.to_str()) == Some("CONDITIONAL_RENDER.flow")
            })
            .expect("expected generated SMT8 flow file");
        let flow = fs::read_to_string(flow_path)?;

        assert!(flow.contains("        initialize();\n        test1.execute();"));
        assert!(flow.contains("if (TEST1_FAILED == 1) {\n            stop();\n        }"));
        Ok(())
    }

    #[test]
    fn passed_and_failed_branches_render_as_if_else() -> crate::Result<()> {
        let flow = node!(PGM::Flow, "branches".to_string() =>
            node!(PGM::TestStr, "t1".to_string(), FlowID::from_str("t1"), None, None, None =>
                node!(PGM::OnFailed, FlowID::from_str("t1") =>
                    node!(PGM::Render, "failed();".to_string())
                ),
                node!(PGM::OnPassed, FlowID::from_str("t1") =>
                    node!(PGM::Render, "passed();".to_string())
                )
            )
        );

        let mut flow_ast = crate::ast::AST::new();
        flow_ast.start(flow);
        let (ast, model) = process_flow(
            &flow_ast,
            Model::new(SupportedTester::V93KSMT8),
            SupportedTester::V93KSMT8,
            true,
        )?;
        let output_dir = tempdir()?;
        let (_model, files) = run(&ast, output_dir.path(), model)?;
        let flow_path = files
            .iter()
            .find(|path| path.file_name().and_then(|name| name.to_str()) == Some("BRANCHES.flow"))
            .expect("expected generated SMT8 flow file");
        let flow = fs::read_to_string(flow_path)?;

        assert!(
            flow.contains("t1.execute();\n        if (t1.pass) {\n            passed();\n        } else {\n            failed();\n        }"),
            "{}",
            flow
        );
        Ok(())
    }

    #[test]
    fn only_failed_branch_renders_as_if_not_passed() -> crate::Result<()> {
        let flow = node!(PGM::Flow, "failed_only".to_string() =>
            node!(PGM::TestStr, "t1".to_string(), FlowID::from_str("t1"), None, None, None =>
                node!(PGM::OnFailed, FlowID::from_str("t1") =>
                    node!(PGM::Render, "failed();".to_string())
                )
            )
        );

        let mut flow_ast = crate::ast::AST::new();
        flow_ast.start(flow);
        let (ast, model) = process_flow(
            &flow_ast,
            Model::new(SupportedTester::V93KSMT8),
            SupportedTester::V93KSMT8,
            true,
        )?;
        let output_dir = tempdir()?;
        let (_model, files) = run(&ast, output_dir.path(), model)?;
        let flow_path = files
            .iter()
            .find(|path| {
                path.file_name().and_then(|name| name.to_str()) == Some("FAILED_ONLY.flow")
            })
            .expect("expected generated SMT8 flow file");
        let flow = fs::read_to_string(flow_path)?;

        assert!(
            flow.contains(
                "t1.execute();\n        if (!t1.pass) {\n            failed();\n        }\n"
            ),
            "{}",
            flow
        );
        assert!(!flow.contains("else"), "{}", flow);
        Ok(())
    }

    #[test]
    fn flow_names_keep_their_underscores() -> crate::Result<()> {
        let flow = node!(PGM::Flow, "main__retest  final".to_string() =>
            node!(PGM::TestStr, "test1".to_string(), FlowID::from_str("test1"), None, None, None)
        );

        let mut flow_ast = crate::ast::AST::new();
        flow_ast.start(flow);
        let (ast, model) = process_flow(
            &flow_ast,
            Model::new(SupportedTester::V93KSMT8),
            SupportedTester::V93KSMT8,
            true,
        )?;
        let output_dir = tempdir()?;
        let (_model, files) = run(&ast, output_dir.path(), model)?;
        // A run of spaces is one underscore, the name's own underscores are kept
        assert!(
            files.iter().any(|path| {
                path.file_name().and_then(|name| name.to_str()) == Some("MAIN__RETEST_FINAL.flow")
            }),
            "{:?}",
            files
        );
        Ok(())
    }

    #[test]
    fn multiline_render_is_indented() -> crate::Result<()> {
        let flow = node!(PGM::Flow, "multiline_render".to_string() =>
            node!(PGM::Volatile, "render_enabled".to_string()),
            node!(PGM::Render, "initialize();\nconfigure();".to_string()),
            node!(PGM::Condition, FlowCondition::IfFlag(vec!["render_enabled".to_string()]) =>
                node!(PGM::Render, "stop();\ncleanup();".to_string())
            )
        );

        let mut flow_ast = crate::ast::AST::new();
        flow_ast.start(flow);
        let (ast, model) = process_flow(
            &flow_ast,
            Model::new(SupportedTester::V93KSMT8),
            SupportedTester::V93KSMT8,
            true,
        )?;
        let output_dir = tempdir()?;
        let (_model, files) = run(&ast, output_dir.path(), model)?;
        let flow_path = files
            .iter()
            .find(|path| {
                path.file_name().and_then(|name| name.to_str()) == Some("MULTILINE_RENDER.flow")
            })
            .expect("expected generated SMT8 flow file");
        let flow = fs::read_to_string(flow_path)?;

        assert!(flow.contains("        initialize();\n        configure();"));
        assert!(flow.contains(
            "if (RENDER_ENABLED == 1) {\n            stop();\n            cleanup();\n        }"
        ));
        Ok(())
    }

    #[test]
    fn nested_result_branch_keeps_parent_guard() -> crate::Result<()> {
        let flow = node!(PGM::Flow, "if_failed_retest".to_string() =>
            node!(PGM::Volatile, "$Alarm".to_string()),
            node!(PGM::TestStr, "primary_check".to_string(), FlowID::from_str("primary_check"), None, None, None),
            node!(PGM::Condition, FlowCondition::IfFailed(vec![FlowID::from_str("primary_check")]) =>
                node!(PGM::TestStr, "primary_check_retest".to_string(), FlowID::from_str("primary_check_retest_id"), None, None, None),
                node!(PGM::Condition, FlowCondition::IfFailed(vec![FlowID::from_str("primary_check_retest_id")]) =>
                    node!(PGM::Condition, FlowCondition::IfFlag(vec!["$Alarm".to_string()]))
                )
            )
        );

        let mut flow_ast = crate::ast::AST::new();
        flow_ast.start(flow);
        let (ast, model) = process_flow(
            &flow_ast,
            Model::new(SupportedTester::V93KSMT8),
            SupportedTester::V93KSMT8,
            true,
        )?;
        let output_dir = tempdir()?;
        let (_model, files) = run(&ast, output_dir.path(), model)?;
        let flow_path = files
            .iter()
            .find(|path| {
                path.file_name().and_then(|name| name.to_str()) == Some("IF_FAILED_RETEST.flow")
            })
            .expect("expected generated SMT8 flow file");
        let flow = fs::read_to_string(flow_path)?;

        assert!(flow.contains("primary_check.execute();"));
        assert!(flow.contains("if (!primary_check.pass) {"));
        assert!(flow.contains("primary_check_retest.execute();"));
        assert!(flow.contains("if (!primary_check_retest.pass) {"));
        assert!(
            !flow.contains("primary_check.execute();\n        if (!primary_check_retest.pass) {")
        );
        Ok(())
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
