use crate::ast::{Node, Processor, Return};
use crate::prog_gen::advantest::smt8::processors::create_flow_data::FlowData;
use crate::prog_gen::config::SMT8Config;
use crate::prog_gen::model::TestCollectionItem;
use crate::prog_gen::{BinType, FlowCondition, GroupType, LoopCount, Model, ParamValue, PGM};
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
    /// Render an else branch which has no content, see Config::set_smt8_empty_else
    empty_else: bool,
    /// Format the flows as Origen 1 writes them, see Config::set_smt8_o1_formatting
    o1_formatting: bool,
    /// Keep the case of sub-flow instance names, see Config::set_smt8_keep_sub_flow_instance_case
    keep_sub_flow_instance_case: bool,
    writer: FileWriter,
}

/// Contains the data for a .flow file
#[derive(Default)]
struct FlowFile {
    /// The name of the flow, its file and its definition
    name: String,
    /// The name of the instance of the flow which its parent calls, the same as its name unless
    /// the case of instance names is kept
    instance: String,
    path: PathBuf,
    indent: usize,
    execute_lines: Vec<String>,
    execute_line_buffers: Vec<Vec<String>>,
    render_bins: bool,
    test_ids: Vec<(String, usize)>,
    existing_test_counter: HashMap<String, usize>,
    existing_flow_counter: HashMap<String, usize>,
    /// The sub-flows which this flow calls, (instance name, flow name)
    sub_flows: Vec<(String, String)>,
    /// Sub-flows which are called with bypass enabled
    bypassed_sub_flows: HashSet<String>,
    flow_data: FlowData,
}

impl FlowFile {
    /// Returns an error if a sub-flow instance has the same name as a test suite of this flow,
    /// both are declared and executed by their names. Sub-flow instances are upper case unless
    /// their case is kept, which can make one the same as a suite.
    fn check_instance_names(&self) -> Result<()> {
        for (instance, _) in &self.sub_flows {
            if self
                .test_ids
                .iter()
                .any(|(test_name, _)| test_name == instance)
            {
                bail!(
                    "In SMT8 flow '{}', the test suite '{}' and the sub-flow (or flow group) \
                     instance '{}' have the same name, rename one of them",
                    self.name,
                    instance,
                    instance
                );
            }
        }
        Ok(())
    }

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
    run_with_options(ast, output_dir, model, &Options::from_config())
}

/// The rendering options which are taken from the config
struct Options {
    /// See Config::set_smt8_empty_else
    empty_else: bool,
    /// See Config::set_smt8_o1_formatting
    o1_formatting: bool,
    /// See Config::set_smt8_keep_sub_flow_instance_case
    keep_sub_flow_instance_case: bool,
}

impl Default for Options {
    /// The default options of the config
    fn default() -> Self {
        Options {
            empty_else: true,
            o1_formatting: false,
            keep_sub_flow_instance_case: false,
        }
    }
}

impl Options {
    fn from_config() -> Options {
        Options {
            empty_else: crate::PROG_GEN_CONFIG.smt8_empty_else(),
            o1_formatting: crate::PROG_GEN_CONFIG.smt8_o1_formatting(),
            keep_sub_flow_instance_case: crate::PROG_GEN_CONFIG.smt8_keep_sub_flow_instance_case(),
        }
    }
}

/// As run, with the option to render an empty else given, rather than taken from the config
#[cfg(test)]
fn run_with_empty_else(
    ast: &Node<PGM>,
    output_dir: &Path,
    model: Model,
    empty_else: bool,
) -> Result<(Model, Vec<PathBuf>)> {
    let options = Options {
        empty_else,
        ..Options::from_config()
    };
    run_with_options(ast, output_dir, model, &options)
}

/// As run, with the options given, rather than taken from the config
fn run_with_options(
    ast: &Node<PGM>,
    output_dir: &Path,
    model: Model,
    options: &Options,
) -> Result<(Model, Vec<PathBuf>)> {
    // For debugging
    //ast.to_file("smt8_flow.txt")?;

    let mut p = FlowGenerator::new(output_dir, model, options);

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
    fn new(output_dir: &Path, model: Model, options: &Options) -> FlowGenerator {
        FlowGenerator {
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
            empty_else: options.empty_else,
            o1_formatting: options.o1_formatting,
            keep_sub_flow_instance_case: options.keep_sub_flow_instance_case,
            writer: FileWriter::new(),
        }
    }

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

    /// Renders a condition as `if (<expression>) { ... } else { ... }`. The node's content is the
    /// first branch when positive (an if_* condition), else the second (an unless_* condition), and
    /// the content of its Else child, if any, is the other branch. When the option to render an
    /// empty else is off, the else is left out when it has no content.
    fn render_condition(
        &mut self,
        node: &Node<PGM>,
        expression: &str,
        positive: bool,
    ) -> Result<()> {
        let else_node = node.children.iter().find(|n| matches!(n.attrs, PGM::Else));
        {
            let current_flow = self.flow_stack.last_mut().unwrap();
            current_flow.execute_line(format!("if ({}) {{", expression));
            current_flow.indent += 1;
        }
        if positive {
            node.process_children(self)?;
        } else if let Some(else_node) = else_node {
            else_node.process_children(self)?;
        }
        if self.empty_else {
            {
                let current_flow = self.flow_stack.last_mut().unwrap();
                current_flow.indent -= 1;
                current_flow.execute_line("} else {".to_string());
                current_flow.indent += 1;
            }
            self.render_else_branch(node, else_node, positive)?;
            let current_flow = self.flow_stack.last_mut().unwrap();
            current_flow.indent -= 1;
            current_flow.execute_line("}".to_string());
        } else {
            self.flow_stack.last_mut().unwrap().start_execute_buffer();
            let rendered = self.render_else_branch(node, else_node, positive);
            let current_flow = self.flow_stack.last_mut().unwrap();
            let lines = current_flow.finish_execute_buffer();
            rendered?;
            current_flow.indent -= 1;
            if !lines.is_empty() {
                current_flow.execute_line("} else {".to_string());
                current_flow.flush_buffered_execute_lines(lines, 0);
            }
            current_flow.execute_line("}".to_string());
        }
        Ok(())
    }

    /// Renders the second branch of a condition, see render_condition
    fn render_else_branch(
        &mut self,
        node: &Node<PGM>,
        else_node: Option<&Box<Node<PGM>>>,
        positive: bool,
    ) -> Result<()> {
        if !positive {
            node.process_children(self)?;
        } else if let Some(else_node) = else_node {
            else_node.process_children(self)?;
        }
        Ok(())
    }

    /// Renders the content of the given flow-type group or sub-flow node into the flow file which
    /// has just been opened for it, and closes it. A flow kept within a resources block is a flow
    /// definition, its content is rendered as it is outside of resources.
    fn render_flow_content(&mut self, node: &Node<PGM>) -> Result<FlowFile> {
        let orig_resources_block = self.resources_block;
        self.resources_block = false;
        // Bypass applies to the call of this flow, not to the flows that it calls
        let orig_bypass = self.bypass_sub_flows;
        self.bypass_sub_flows = false;
        let processed = node.process_children(self);
        self.bypass_sub_flows = orig_bypass;
        self.resources_block = orig_resources_block;
        processed?;
        self.close_flow_file(self.namespaces.last().cloned())
    }

    /// Declares the given flow, which has just been rendered, in the current flow and executes it
    /// there, passing its inputs in and its outputs back. A flow kept within a resources block is
    /// declared but not executed. Returns true if the flow is executed.
    fn call_flow(&mut self, flow: &FlowFile) -> bool {
        let executed = !self.resources_block;
        let o1_formatting = self.o1_formatting;
        let current_flow = self.flow_stack.last_mut().unwrap();
        let inputs = flow.flow_data.sorted_input_vars();
        let outputs: Vec<String> = flow
            .flow_data
            .sorted_output_flags()
            .into_iter()
            .filter(|v| {
                current_flow.flow_data.output_flags.contains(v)
                    || current_flow.flow_data.referenced_flags.contains(v)
                    || current_flow.flow_data.downstream_flags.contains(v)
            })
            .collect();
        // Origen 1 groups the assignments of each kind in a block headed by a comment
        let open_block = |current_flow: &mut FlowFile, kind: &str| {
            current_flow.execute_line(format!("// {} sub-flow {} variables", flow.instance, kind));
            current_flow.execute_line("{".to_string());
            current_flow.indent += 1;
        };
        let close_block = |current_flow: &mut FlowFile| {
            current_flow.indent -= 1;
            current_flow.execute_line("}".to_string());
        };
        if executed {
            let grouped = o1_formatting && !inputs.is_empty();
            if grouped {
                open_block(current_flow, "input");
            }
            for v in &inputs {
                current_flow.execute_line(format!("{}.{} = {};", flow.instance, v, v));
            }
            if grouped {
                close_block(current_flow);
            }
            current_flow.execute_line(format!("{}.execute();", flow.instance));
        }
        current_flow
            .sub_flows
            .push((flow.instance.clone(), flow.name.clone()));
        if self.bypass_sub_flows {
            current_flow
                .bypassed_sub_flows
                .insert(flow.instance.clone());
        }
        if executed {
            let grouped = o1_formatting && !outputs.is_empty();
            if grouped {
                open_block(current_flow, "output");
            }
            for v in &outputs {
                current_flow.execute_line(format!("{} = {}.{};", v, flow.instance, v));
            }
            if grouped {
                close_block(current_flow);
            }
        }
        executed
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

    /// Renders the parameters of a test suite, or of the given collection item within it
    fn render_sorted_contents<W: Write>(
        &self,
        f: &mut W,
        indent: usize,
        item: Option<&TestCollectionItem>,
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
                        .is_some_and(|default| default.is_same_value(value))
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
        match (self.o1_formatting, item) {
            // Origen 1 orders a test suite's parameters by the byte order of their names
            (true, None) => names.sort(),
            // and a collection item's parameters in the order of their definition, followed here
            // by its collections in the order of theirs, anything else follows those by name.
            // A template defines a collection's parameters and its collections in separate maps,
            // so a collection defined between two parameters, which Origen 1 writes in between them,
            // is written after them here.
            (true, Some(item)) => {
                let position = |name: &String| {
                    item.params
                        .get_index_of(name)
                        .or_else(|| {
                            item.collection_defs
                                .get_index_of(name)
                                .map(|i| item.params.len() + i)
                        })
                        .unwrap_or(usize::MAX)
                };
                names.sort_by(|a, b| {
                    position(a)
                        .cmp(&position(b))
                        .then_with(|| Self::alpha_cmp(a, b))
                });
            }
            (false, _) => names.sort_by(|a, b| Self::alpha_cmp(a, b)),
        }
        names.dedup();

        for name in names {
            if let Some(value) = values.get(&name) {
                if !self.options.render_default_tmparams
                    && default_values
                        .get(&name)
                        .is_some_and(|default| default.is_same_value(value))
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
                        Some(item),
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
                    p = f.instance.to_string();
                } else {
                    p = format!("{}.{}", p, f.instance);
                }
            }
            Some(p)
        }
    }

    fn open_flow_file(&mut self, name: &str, flow_data: FlowData) -> Result<()> {
        // Create a clean name where each run of spaces is an underscore, keeping any underscores
        // of the name itself, e.g. a MAIN__RETEST sub-flow. The flow's name is upper case, its
        // instance name keeps its case when asked.
        let mut instance = name.split_whitespace().collect::<Vec<_>>().join("_");
        let mut name = instance.to_uppercase();
        if !self.keep_sub_flow_instance_case {
            instance = name.clone();
        }
        if let Some(current_flow) = self.flow_stack.last_mut() {
            if current_flow.existing_flow_counter.contains_key(&name) {
                let count = current_flow.existing_flow_counter.get(&name).unwrap() + 1;
                current_flow
                    .existing_flow_counter
                    .insert(name.to_owned(), count);
                name = format!("{}_{}", name, count);
                instance = format!("{}_{}", instance, count);
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
            instance,
            path: flow_path,
            render_bins: true,
            flow_data,
            ..Default::default()
        });
        Ok(())
    }

    fn close_flow_file(&mut self, namespace: Option<String>) -> Result<FlowFile> {
        let flow_file = self.flow_stack.pop().unwrap();
        if self.keep_sub_flow_instance_case {
            flow_file.check_instance_names()?;
        }
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
                    None,
                    &test.values,
                    &test.definition.default_values,
                    &test.collections,
                )?;
                writeln!(&mut f, "        }}")?;
            }
            writeln!(&mut f, "")?;
        }
        for (instance, sub_flow) in &flow_file.sub_flows {
            let relative_path = flow_file
                .path
                .strip_prefix(&self.output_dir)
                .unwrap()
                .parent()
                .unwrap()
                .join(flow_file.name.to_lowercase());
            let call = format!(
                "        flow {} calls {}flows.{}.{} {{",
                instance,
                &namespace,
                relative_path
                    .to_str()
                    .unwrap()
                    .replace("\\", ".")
                    .replace("/", "."),
                sub_flow
            );
            if flow_file.bypassed_sub_flows.contains(instance) {
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
        if self.o1_formatting {
            // Only the flags which this flow sets itself are initialized, as Origen 1 does, not those
            // which it only declares as outputs, which a test method sets natively
            let mut flags: Vec<&String> = flow_file.flow_data.set_flags.iter().collect();
            flags.sort();
            for v in &flags {
                writeln!(&mut f, "        {} = -1;", v)?;
            }
            if !flags.is_empty() {
                writeln!(&mut f, "")?;
            }
        } else {
            for v in flow_file.flow_data.sorted_modified_flags() {
                writeln!(&mut f, "        {} = -1;", v)?;
            }
            if !flow_file.flow_data.output_flags.is_empty() {
                writeln!(&mut f, "")?;
            }
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
                let flow = self.render_flow_content(node)?;
                self.sub_flow_open = orig;
                self.call_flow(&flow);
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
                    let flow = self.render_flow_content(node)?;
                    if self.call_flow(&flow) {
                        self.render_result_branches(&node.children, &flow.instance, false)?;
                    }
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
                    let expression = jobs
                        .iter()
                        .map(|j| {
                            if jobs.len() > 1 {
                                format!("(JOB == \"{}\")", j.to_uppercase())
                            } else {
                                format!("JOB == \"{}\"", j.to_uppercase())
                            }
                        })
                        .collect::<Vec<String>>()
                        .join(" || ");
                    self.render_condition(
                        node,
                        &expression,
                        matches!(cond, FlowCondition::IfJob(_)),
                    )?;
                    Return::None
                }
                FlowCondition::IfEnable(flags)
                | FlowCondition::UnlessEnable(flags)
                | FlowCondition::IfFlag(flags)
                | FlowCondition::UnlessFlag(flags) => {
                    let expression = flags
                        .iter()
                        .map(|f| {
                            if flags.len() > 1 {
                                format!("({} == 1)", f.to_uppercase())
                            } else {
                                format!("{} == 1", f.to_uppercase())
                            }
                        })
                        .collect::<Vec<String>>()
                        .join(" || ");
                    let positive =
                        matches!(cond, FlowCondition::IfEnable(_) | FlowCondition::IfFlag(_));
                    self.render_condition(node, &expression, positive)?;
                    Return::None
                }
                FlowCondition::IfExpr(expression) | FlowCondition::UnlessExpr(expression) => {
                    self.render_condition(
                        node,
                        expression,
                        matches!(cond, FlowCondition::IfExpr(_)),
                    )?;
                    Return::None
                }
                _ => Return::ProcessChildren,
            },
            // A loop within resources only holds the flows which are kept there, it is not
            // executed
            PGM::Loop(..) if self.resources_block => {
                node.process_children(self)?;
                Return::None
            }
            PGM::Loop(count, variable, descending, start) => {
                // The first and last values of the loop variable, joined like this to match
                // hand-written SMT8, e.g. 1..3 and 1 .. count. Computed as i128 so that no start
                // and count can overflow.
                let start = *start as i128;
                let (first, last, sep) = match count {
                    // SMT8 counts down a range whose first value is greater, so 1..0 would run
                    // twice
                    Some(LoopCount::Fixed(0)) => {
                        bail!("A V93K SMT8 loop must repeat at least once, its count is 0")
                    }
                    Some(LoopCount::Fixed(c)) => (
                        start.to_string(),
                        (start + *c as i128 - 1).to_string(),
                        "..",
                    ),
                    Some(LoopCount::Variable(name)) => {
                        let last = match start - 1 {
                            0 => name.to_string(),
                            offset if offset > 0 => format!("{} + {}", name, offset),
                            offset => format!("{} - {}", name, -offset),
                        };
                        (start.to_string(), last, " .. ")
                    }
                    None => bail!(
                        "V93K SMT8 flow generation only supports loops with an iteration count"
                    ),
                };
                let variable = variable.clone().unwrap_or_else(|| "index".to_string());
                {
                    let current_flow = self.flow_stack.last_mut().unwrap();
                    if *descending {
                        current_flow.execute_line(format!(
                            "for ({} : {}{}{}) {{",
                            variable, last, sep, first
                        ));
                    } else {
                        current_flow.execute_line(format!(
                            "for ({} : {}{}{}) {{",
                            variable, first, sep, last
                        ));
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
            PGM::OutputVariable(_)
            | PGM::InputVariable(_)
            | PGM::LocalFlag(_)
            | PGM::LocalVariable(_) => Return::None,
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
            // Only analysed to declare the flow's variables
            PGM::FlowDataOnly => Return::None,
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
    use super::{run, run_with_empty_else, run_with_options, FileWriter, FlowGenerator, Options};
    use crate::prog_gen::{
        process_flow, FlowCondition, FlowID, GroupType, LoopCount, Model, ParamValue,
        SupportedTester, PGM,
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
    fn opposite_conditions_within_a_loop_are_rendered_as_if_else() -> crate::Result<()> {
        let flow = node!(PGM::Flow, "combined".to_string() =>
            node!(PGM::Loop, Some(LoopCount::Fixed(2)), Some("i".to_string()), false, 1 =>
                node!(PGM::Condition, FlowCondition::IfExpr("t.pass".to_string()) =>
                    node!(PGM::SetFlag, "FLAG_B".to_string(), true, false)
                ),
                node!(PGM::Condition, FlowCondition::UnlessExpr("t.pass".to_string()) =>
                    node!(PGM::TestStr, "t_b".to_string(), FlowID::from_str("t_b"), None, None, None)
                )
            )
        );
        let flow = render_flow_file(flow, "COMBINED.flow")?;
        assert!(
            flow.contains(
                "        for (i : 1..2) {\n\
                \x20           if (t.pass) {\n\
                \x20               FLAG_B = 1;\n\
                \x20           } else {\n\
                \x20               t_b.execute();\n\
                \x20           }\n\
                \x20       }\n"
            ),
            "{}",
            flow
        );
        Ok(())
    }

    #[test]
    fn loops_are_rendered_ascending_and_descending() -> crate::Result<()> {
        let flow = node!(PGM::Flow, "loops".to_string() =>
            node!(PGM::Loop, Some(LoopCount::Fixed(3)), Some("attempt".to_string()), false, 1 =>
                node!(PGM::TestStr, "looped".to_string(), FlowID::from_str("looped"), None, None, None)
            ),
            node!(PGM::Loop, Some(LoopCount::Fixed(9)), None, true, 1 =>
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
    fn loops_can_start_at_any_index() -> crate::Result<()> {
        let looped = |name: &str| {
            node!(
                PGM::TestStr,
                name.to_string(),
                FlowID::from_str(name),
                None,
                None,
                None
            )
        };
        let flow = node!(PGM::Flow, "starts".to_string() =>
            node!(PGM::Loop, Some(LoopCount::Fixed(6)), Some("i".to_string()), false, 0 =>
                looped("from_0")
            ),
            node!(PGM::Loop, Some(LoopCount::Fixed(6)), Some("j".to_string()), true, 0 =>
                looped("down_to_0")
            ),
            node!(PGM::Loop, Some(LoopCount::Fixed(2)), Some("k".to_string()), false, 5 =>
                looped("from_5")
            )
        );
        let flow = render_flow_file(flow, "STARTS.flow")?;
        assert!(flow.contains("for (i : 0..5) {"), "{}", flow);
        assert!(flow.contains("for (j : 5..0) {"), "{}", flow);
        assert!(flow.contains("for (k : 5..6) {"), "{}", flow);
        Ok(())
    }

    #[test]
    fn loops_can_repeat_to_a_count_held_in_a_flow_variable() -> crate::Result<()> {
        let looped = |name: &str| {
            node!(
                PGM::TestStr,
                name.to_string(),
                FlowID::from_str(name),
                None,
                None,
                None
            )
        };
        let count = |name: &str| Some(LoopCount::Variable(name.to_string()));
        let flow = node!(PGM::Flow, "counted".to_string() =>
            node!(PGM::Loop, count("retryCount"), Some("attempt".to_string()), false, 1 =>
                looped("up")
            ),
            node!(PGM::Loop, count("retryCount"), Some("attempt".to_string()), true, 1 =>
                looped("down")
            ),
            node!(PGM::Loop, count("retryCount"), Some("attempt".to_string()), false, 0 =>
                looped("from_0")
            ),
            node!(PGM::Loop, count("retryCount"), Some("attempt".to_string()), false, 3 =>
                looped("from_3")
            ),
            // A nested loop may count to the variable of the loop around it
            node!(PGM::Loop, Some(LoopCount::Fixed(3)), Some("outer".to_string()), false, 1 =>
                node!(PGM::Loop, count("outer"), Some("inner".to_string()), false, 1 =>
                    looped("nested")
                )
            )
        );
        let flow = render_flow_file(flow, "COUNTED.flow")?;
        assert!(
            flow.contains("for (attempt : 1 .. retryCount) {"),
            "{}",
            flow
        );
        assert!(
            flow.contains("for (attempt : retryCount .. 1) {"),
            "{}",
            flow
        );
        assert!(
            flow.contains("for (attempt : 0 .. retryCount - 1) {"),
            "{}",
            flow
        );
        assert!(
            flow.contains("for (attempt : 3 .. retryCount + 2) {"),
            "{}",
            flow
        );
        assert!(flow.contains("for (inner : 1 .. outer) {"), "{}", flow);
        // The count is an input of the flow, keeping its case, the loop variables are not
        assert!(flow.contains("    in retryCount = -1;"), "{}", flow);
        assert!(!flow.contains("in outer"), "{}", flow);
        assert!(!flow.contains("RETRYCOUNT"), "{}", flow);
        Ok(())
    }

    #[test]
    fn a_loop_count_declared_as_an_output_is_not_an_input() -> crate::Result<()> {
        let flow = node!(PGM::Flow, "counted".to_string() =>
            node!(PGM::SubFlow, "sub".to_string(), None =>
                node!(PGM::TestStr, "set_count".to_string(), FlowID::from_str("set_count"), None, None, None),
                node!(PGM::Loop, Some(LoopCount::Variable("retryCount".to_string())), Some("attempt".to_string()), false, 1 =>
                    node!(PGM::TestStr, "step".to_string(), FlowID::from_str("step"), None, None, None)
                ),
                node!(PGM::OutputVariable, "retryCount".to_string())
            )
        );
        let sub = render_flow_file(flow.clone(), "SUB.flow")?;
        assert!(sub.contains("    out retryCount = -1;"), "{}", sub);
        assert!(!sub.contains("in retryCount"), "{}", sub);
        assert!(sub.contains("for (attempt : 1 .. retryCount) {"), "{}", sub);
        let top = render_flow_file(flow, "COUNTED.flow")?;
        assert!(!top.contains("in retryCount"), "{}", top);
        Ok(())
    }

    #[test]
    fn a_loop_without_a_count_is_an_error() {
        let flow = node!(PGM::Flow, "unbounded".to_string() =>
            node!(PGM::Loop, None, None, false, 1 =>
                node!(PGM::TestStr, "t".to_string(), FlowID::from_str("t"), None, None, None)
            )
        );
        assert!(render_flow_file(flow, "UNBOUNDED.flow").is_err());
    }

    #[test]
    fn a_loop_with_a_fixed_count_of_0_is_an_error() {
        // SMT8 would run 1..0 twice, counting down
        let flow = node!(PGM::Flow, "never".to_string() =>
            node!(PGM::Loop, Some(LoopCount::Fixed(0)), None, false, 1 =>
                node!(PGM::TestStr, "t".to_string(), FlowID::from_str("t"), None, None, None)
            )
        );
        assert!(render_flow_file(flow, "NEVER.flow").is_err());
    }

    #[test]
    fn loop_ranges_at_the_limits_of_the_start_index_do_not_overflow() -> crate::Result<()> {
        let flow = node!(PGM::Flow, "limits".to_string() =>
            node!(PGM::Loop, Some(LoopCount::Fixed(2)), Some("i".to_string()), false, i64::MAX =>
                node!(PGM::TestStr, "high".to_string(), FlowID::from_str("high"), None, None, None)
            ),
            node!(PGM::Loop, Some(LoopCount::Variable("count".to_string())), Some("j".to_string()), false, i64::MIN =>
                node!(PGM::TestStr, "low".to_string(), FlowID::from_str("low"), None, None, None)
            )
        );
        let flow = render_flow_file(flow, "LIMITS.flow")?;
        assert!(
            flow.contains("for (i : 9223372036854775807..9223372036854775808) {"),
            "{}",
            flow
        );
        assert!(
            flow.contains("for (j : -9223372036854775808 .. count - 9223372036854775809) {"),
            "{}",
            flow
        );
        Ok(())
    }

    #[test]
    fn a_loop_counter_read_by_a_flow_called_within_the_loop_is_passed_down_to_it(
    ) -> crate::Result<()> {
        let flow = node!(PGM::Flow, "top".to_string() =>
            node!(PGM::Loop, Some(LoopCount::Fixed(3)), Some("outer".to_string()), false, 1 =>
                flow_group("grp", vec![
                    node!(PGM::Loop, Some(LoopCount::Variable("outer".to_string())), Some("inner".to_string()), false, 1 =>
                        test_str("in_grp")
                    ),
                    flow_group("nested", vec![
                        node!(PGM::Condition, FlowCondition::IfExpr("outer > 1".to_string()) =>
                            test_str("in_nested")
                        )
                    ]),
                ])
            )
        );
        // The caller's loop counter is passed down from within its loop, it's not an input of
        // the caller
        let top = render_flow_file(flow.clone(), "TOP.flow")?;
        assert!(!top.contains("in outer"), "{}", top);
        assert!(
            top.contains(
                "for (outer : 1..3) {\n            GRP.outer = outer;\n            GRP.execute();"
            ),
            "{}",
            top
        );
        let grp = render_flow_file(flow.clone(), "GRP.flow")?;
        assert!(grp.contains("    in outer = -1;"), "{}", grp);
        assert!(grp.contains("for (inner : 1 .. outer) {"), "{}", grp);
        assert!(grp.contains("NESTED.outer = outer;"), "{}", grp);
        let nested = render_flow_file(flow, "NESTED.flow")?;
        assert!(nested.contains("    in outer = -1;"), "{}", nested);
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
                node!(PGM::Loop, Some(LoopCount::Fixed(9)), Some("per_tile_index".to_string()), true, 1 =>
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

    /// A flow with a parent flow containing a sub-flow which sets FLAG and a sub-flow which
    /// reads it, in the given order
    fn setter_and_reader(setter_first: bool) -> crate::ast::Node<PGM> {
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
        let setter = node!(PGM::Group, "setter".to_string(), None, GroupType::Flow, Some(FlowID::from_str("setter")) =>
            t("check"),
            node!(PGM::SetFlag, "flag".to_string(), true, false)
        );
        let reader = node!(PGM::Group, "reader".to_string(), None, GroupType::Flow, Some(FlowID::from_str("reader")) =>
            node!(PGM::Condition, FlowCondition::UnlessFlag(vec!["flag".to_string()]) =>
                t("gated")
            )
        );
        let (first, second) = if setter_first {
            (setter, reader)
        } else {
            (reader, setter)
        };
        node!(PGM::Flow, "top".to_string() =>
            node!(PGM::Group, "parent".to_string(), None, GroupType::Flow, Some(FlowID::from_str("parent")) =>
                first,
                second
            )
        )
    }

    #[test]
    fn a_flag_set_by_an_earlier_sub_flow_is_not_an_input_of_the_enclosing_flows(
    ) -> crate::Result<()> {
        let flow = setter_and_reader(true);
        let parent = render_flow_file(flow.clone(), "PARENT.flow")?;
        assert!(parent.contains("    out FLAG = -1;"), "{}", parent);
        assert!(!parent.contains("in FLAG"), "{}", parent);
        assert!(
            parent.contains(
                "FLAG = SETTER.FLAG;\n        READER.FLAG = FLAG;\n        READER.execute();"
            ),
            "{}",
            parent
        );
        let top = render_flow_file(flow.clone(), "TOP.flow")?;
        assert!(top.contains("    out FLAG = -1;"), "{}", top);
        assert!(!top.contains("in FLAG"), "{}", top);
        let reader = render_flow_file(flow, "READER.flow")?;
        assert!(reader.contains("    in FLAG = -1;"), "{}", reader);
        Ok(())
    }

    #[test]
    fn a_flag_read_before_a_sub_flow_sets_it_is_an_input_of_the_enclosing_flows(
    ) -> crate::Result<()> {
        let flow = setter_and_reader(false);
        let parent = render_flow_file(flow.clone(), "PARENT.flow")?;
        assert!(parent.contains("    in FLAG = -1;"), "{}", parent);
        assert!(parent.contains("READER.FLAG = FLAG;"), "{}", parent);
        let top = render_flow_file(flow, "TOP.flow")?;
        assert!(top.contains("    in FLAG = -1;"), "{}", top);
        assert!(top.contains("PARENT.FLAG = FLAG;"), "{}", top);
        Ok(())
    }

    fn flow_group(name: &str, children: Vec<crate::ast::Node<PGM>>) -> crate::ast::Node<PGM> {
        let mut group = node!(
            PGM::Group,
            name.to_string(),
            None,
            GroupType::Flow,
            Some(FlowID::from_str(name))
        );
        for child in children {
            group.add_child(child);
        }
        group
    }

    fn test_str(name: &str) -> crate::ast::Node<PGM> {
        node!(
            PGM::TestStr,
            name.to_string(),
            FlowID::from_str(name),
            None,
            None,
            None
        )
    }

    fn unless_flag(flag: &str, test: &str) -> crate::ast::Node<PGM> {
        node!(PGM::Condition, FlowCondition::UnlessFlag(vec![flag.to_string()]) =>
            test_str(test)
        )
    }

    fn set_flag(flag: &str) -> crate::ast::Node<PGM> {
        node!(PGM::SetFlag, flag.to_string(), true, false)
    }

    fn local_flag(flag: &str) -> crate::ast::Node<PGM> {
        node!(PGM::LocalFlag, flag.to_string())
    }

    #[test]
    fn a_local_flag_which_no_other_flow_reads_stays_in_the_flow_which_sets_it() -> crate::Result<()>
    {
        let flow = node!(PGM::Flow, "top".to_string() =>
            flow_group("sub", vec![
                local_flag("flag"),
                test_str("check"),
                set_flag("flag"),
                unless_flag("flag", "on_fail"),
                // Read by a sub-flow of the flow which sets it, it is passed down
                flow_group("nested", vec![test_str("first"), unless_flag("flag", "nested_test")]),
            ])
        );
        let sub = render_flow_file(flow.clone(), "SUB.flow")?;
        assert!(sub.contains("    out FLAG = -1;"), "{}", sub);
        assert!(sub.contains("NESTED.FLAG = FLAG;"), "{}", sub);
        let top = render_flow_file(flow, "TOP.flow")?;
        assert!(!top.contains("FLAG"), "{}", top);
        Ok(())
    }

    #[test]
    fn a_local_flag_is_passed_out_as_far_as_the_highest_flow_with_another_reader(
    ) -> crate::Result<()> {
        // Declared by the parent, for the sub-flow which sets it
        let flow = node!(PGM::Flow, "top".to_string() =>
            flow_group("parent", vec![
                local_flag("FLAG"),
                flow_group("mid", vec![
                    flow_group("setter", vec![test_str("check"), set_flag("flag")]),
                ]),
                flow_group("reader", vec![unless_flag("flag", "gated")]),
            ])
        );
        let mid = render_flow_file(flow.clone(), "MID.flow")?;
        assert!(mid.contains("    out FLAG = -1;"), "{}", mid);
        assert!(mid.contains("FLAG = SETTER.FLAG;"), "{}", mid);
        // The parent holds it, initializing it, copying it up from the setter's flow and passing
        // it down to the reader, but it isn't an output or an input of the parent
        let parent = render_flow_file(flow.clone(), "PARENT.flow")?;
        assert!(!parent.contains("out FLAG"), "{}", parent);
        assert!(!parent.contains("in FLAG"), "{}", parent);
        assert!(parent.contains("        FLAG = -1;"), "{}", parent);
        assert!(
            parent.contains(
                "FLAG = MID.FLAG;\n        READER.FLAG = FLAG;\n        READER.execute();"
            ),
            "{}",
            parent
        );
        let top = render_flow_file(flow, "TOP.flow")?;
        assert!(!top.contains("FLAG"), "{}", top);
        Ok(())
    }

    #[test]
    fn a_local_flag_read_by_an_expression_stays_in_the_flow_which_holds_it() -> crate::Result<()> {
        let flow = node!(PGM::Flow, "top".to_string() =>
            flow_group("parent", vec![
                local_flag("FLAG"),
                flow_group("setter", vec![test_str("check"), set_flag("flag")]),
                flow_group("reader", vec![
                    node!(PGM::Condition, FlowCondition::IfExpr("FLAG == 1".to_string()) =>
                        test_str("gated")
                    )
                ]),
            ])
        );
        let parent = render_flow_file(flow.clone(), "PARENT.flow")?;
        assert!(!parent.contains("in FLAG"), "{}", parent);
        assert!(!parent.contains("out FLAG"), "{}", parent);
        assert!(
            parent.contains("FLAG = SETTER.FLAG;\n        READER.FLAG = FLAG;"),
            "{}",
            parent
        );
        let reader = render_flow_file(flow.clone(), "READER.flow")?;
        assert!(reader.contains("    in FLAG = -1;"), "{}", reader);
        let top = render_flow_file(flow, "TOP.flow")?;
        assert!(!top.contains("FLAG"), "{}", top);
        Ok(())
    }

    #[test]
    fn a_local_flag_read_by_an_enclosing_flow_is_passed_out_to_it() -> crate::Result<()> {
        let flow = node!(PGM::Flow, "top".to_string() =>
            flow_group("mid", vec![
                flow_group("setter", vec![local_flag("flag"), set_flag("flag")]),
            ]),
            unless_flag("flag", "gated")
        );
        let mid = render_flow_file(flow.clone(), "MID.flow")?;
        assert!(mid.contains("    out FLAG = -1;"), "{}", mid);
        let top = render_flow_file(flow, "TOP.flow")?;
        assert!(!top.contains("out FLAG"), "{}", top);
        assert!(!top.contains("in FLAG"), "{}", top);
        assert!(top.contains("FLAG = MID.FLAG;"), "{}", top);
        Ok(())
    }

    #[test]
    fn a_flag_passed_up_to_a_flow_which_reads_it_with_an_expression_is_not_its_input(
    ) -> crate::Result<()> {
        let reads = || {
            node!(PGM::Condition, FlowCondition::IfExpr("FLAG == 1".to_string()) =>
                test_str("gated")
            )
        };
        // The holder of a local flag reads it, and a flag passed up to every enclosing flow is
        // read by one of them
        for local in [true, false] {
            let mut parent = vec![flow_group(
                "setter",
                vec![test_str("check"), set_flag("flag")],
            )];
            if local {
                parent.insert(0, local_flag("FLAG"));
            }
            parent.push(reads());
            let flow = node!(PGM::Flow, "top".to_string() => flow_group("parent", parent));
            let parent = render_flow_file(flow.clone(), "PARENT.flow")?;
            assert!(!parent.contains("in FLAG"), "{}", parent);
            assert!(
                parent.contains("FLAG = SETTER.FLAG;\n        if (FLAG == 1) {"),
                "{}",
                parent
            );
            let top = render_flow_file(flow, "TOP.flow")?;
            assert!(!top.contains("in FLAG"), "{}", top);
            assert_eq!(top.contains("out FLAG"), !local, "{}", top);
        }
        Ok(())
    }

    #[test]
    fn a_flag_declared_local_and_as_an_output_is_output() -> crate::Result<()> {
        let flow = node!(PGM::Flow, "top".to_string() =>
            flow_group("sub", vec![
                local_flag("flag"),
                node!(PGM::OutputVariable, "FLAG".to_string()),
                set_flag("flag"),
            ])
        );
        let top = render_flow_file(flow, "TOP.flow")?;
        assert!(top.contains("    out FLAG = -1;"), "{}", top);
        assert!(top.contains("FLAG = SUB.FLAG;"), "{}", top);
        Ok(())
    }

    #[test]
    fn a_flag_which_is_not_declared_local_is_passed_out_to_every_enclosing_flow(
    ) -> crate::Result<()> {
        // Declared local by a flow which doesn't enclose the flow which sets it
        let flow = node!(PGM::Flow, "top".to_string() =>
            flow_group("other", vec![local_flag("flag")]),
            flow_group("sub", vec![set_flag("flag"), unless_flag("flag", "gated")])
        );
        let top = render_flow_file(flow, "TOP.flow")?;
        assert!(top.contains("    out FLAG = -1;"), "{}", top);
        assert!(top.contains("FLAG = SUB.FLAG;"), "{}", top);
        Ok(())
    }

    #[test]
    fn a_flag_set_by_a_flow_is_not_its_input_when_its_sub_flow_reads_it() -> crate::Result<()> {
        let flow = node!(PGM::Flow, "top".to_string() =>
            node!(PGM::SetFlag, "flag".to_string(), true, false),
            node!(PGM::Group, "reader".to_string(), None, GroupType::Flow, Some(FlowID::from_str("reader")) =>
                node!(PGM::Condition, FlowCondition::IfFlag(vec!["flag".to_string()]) =>
                    node!(PGM::TestStr, "gated".to_string(), FlowID::from_str("gated"), None, None, None)
                )
            )
        );
        let top = render_flow_file(flow, "TOP.flow")?;
        assert!(!top.contains("in FLAG"), "{}", top);
        assert!(
            top.contains("FLAG = 1;\n        READER.FLAG = FLAG;"),
            "{}",
            top
        );
        Ok(())
    }

    #[test]
    fn declared_input_variables_keep_their_case_and_are_passed_down() -> crate::Result<()> {
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
        let flow = node!(PGM::Flow, "inputs".to_string() =>
            // Declared by a flow which only passes it down, and by the sub-flow which receives it
            node!(PGM::Group, "outer".to_string(), None, GroupType::Flow, Some(FlowID::from_str("outer")) =>
                node!(PGM::InputVariable, "tileMask".to_string()),
                node!(PGM::Group, "sub".to_string(), None, GroupType::Flow, Some(FlowID::from_str("sub")) =>
                    node!(PGM::InputVariable, "tileMask".to_string()),
                    t("in_sub")
                ),
                node!(PGM::Group, "other".to_string(), None, GroupType::Flow, Some(FlowID::from_str("other")) =>
                    t("in_other")
                )
            ),
            // Declared only by a flow nested in a flow which doesn't declare it
            node!(PGM::Group, "middle".to_string(), None, GroupType::Flow, Some(FlowID::from_str("middle")) =>
                node!(PGM::Group, "deep".to_string(), None, GroupType::Flow, Some(FlowID::from_str("deep")) =>
                    node!(PGM::InputVariable, "DEPTH".to_string()),
                    t("in_deep")
                )
            )
        );

        let top = render_flow_file(flow.clone(), "INPUTS.flow")?;
        assert!(top.contains("    in tileMask = -1;"), "{}", top);
        assert!(top.contains("    in DEPTH = -1;"), "{}", top);
        assert!(
            top.contains("OUTER.tileMask = tileMask;\n        OUTER.execute();"),
            "{}",
            top
        );
        assert!(top.contains("MIDDLE.DEPTH = DEPTH;"), "{}", top);
        assert!(!top.contains("out "), "{}", top);

        let outer = render_flow_file(flow.clone(), "OUTER.flow")?;
        assert!(outer.contains("    in tileMask = -1;"), "{}", outer);
        assert!(outer.contains("SUB.tileMask = tileMask;"), "{}", outer);
        // Not passed to a sub-flow which neither reads nor declares it
        assert!(!outer.contains("OTHER.tileMask"), "{}", outer);
        assert!(!outer.contains("TILEMASK"), "{}", outer);

        let sub = render_flow_file(flow.clone(), "SUB.flow")?;
        assert!(sub.contains("    in tileMask = -1;"), "{}", sub);
        let other = render_flow_file(flow.clone(), "OTHER.flow")?;
        assert!(!other.contains("tileMask"), "{}", other);

        let middle = render_flow_file(flow.clone(), "MIDDLE.flow")?;
        assert!(middle.contains("    in DEPTH = -1;"), "{}", middle);
        assert!(middle.contains("DEEP.DEPTH = DEPTH;"), "{}", middle);
        let deep = render_flow_file(flow, "DEEP.flow")?;
        assert!(deep.contains("    in DEPTH = -1;"), "{}", deep);
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

    fn set_variable(name: &str, value: &str) -> crate::ast::Node<PGM> {
        node!(PGM::SetVariable, name.to_string(), value.to_string())
    }

    fn if_expr_node(
        expression: &str,
        children: Vec<crate::ast::Node<PGM>>,
    ) -> crate::ast::Node<PGM> {
        let mut node = node!(
            PGM::Condition,
            FlowCondition::IfExpr(expression.to_string())
        );
        for child in children {
            node.add_child(child);
        }
        node
    }

    fn local_variable(name: &str) -> crate::ast::Node<PGM> {
        node!(PGM::LocalVariable, name.to_string())
    }

    /// The flow of the issue: a counter which assigns and reads its own variable
    fn counter(declare_local: bool) -> crate::ast::Node<PGM> {
        let mut children = vec![];
        if declare_local {
            children.push(local_variable("passed_count"));
        }
        children.extend(vec![
            set_variable("passed_count", "-1"),
            test_str("count"),
            if_expr_node("count.pass", vec![set_variable("passed_count", "1")]),
            if_expr_node("passed_count == 1", vec![test_str("counted")]),
        ]);
        flow_group("counter", children)
    }

    #[test]
    fn a_local_variable_assigned_and_read_by_one_flow_stays_in_it() -> crate::Result<()> {
        let flow = node!(PGM::Flow, "top".to_string() =>
            flow_group("outer", vec![counter(true)])
        );
        let counter = render_flow_file(flow.clone(), "COUNTER.flow")?;
        assert!(
            counter.contains("    out passed_count = -1;"),
            "{}",
            counter
        );
        assert!(!counter.contains("in passed_count"), "{}", counter);
        assert!(
            counter.contains("    execute {\n        passed_count = -1;\n\n        passed_count = -1;\n        count.execute();"),
            "{}",
            counter
        );
        for file in ["OUTER.flow", "TOP.flow"] {
            let parent = render_flow_file(flow.clone(), file)?;
            assert!(!parent.contains("passed_count"), "{}: {}", file, parent);
        }
        Ok(())
    }

    #[test]
    fn a_variable_declared_local_by_an_enclosing_flow_is_local() -> crate::Result<()> {
        let flow = node!(PGM::Flow, "top".to_string() =>
            flow_group("outer", vec![local_variable("passed_count"), counter(false)])
        );
        let counter = render_flow_file(flow.clone(), "COUNTER.flow")?;
        assert!(
            counter.contains("    out passed_count = -1;"),
            "{}",
            counter
        );
        assert!(!counter.contains("in passed_count"), "{}", counter);
        for file in ["OUTER.flow", "TOP.flow"] {
            let parent = render_flow_file(flow.clone(), file)?;
            assert!(!parent.contains("passed_count"), "{}: {}", file, parent);
        }
        Ok(())
    }

    #[test]
    fn a_local_variable_is_passed_out_as_far_as_another_reader() -> crate::Result<()> {
        let flow = node!(PGM::Flow, "top".to_string() =>
            flow_group("outer", vec![
                local_variable("v"),
                flow_group("setter", vec![set_variable("v", "1")]),
                flow_group("reader", vec![if_expr_node("v == 1", vec![test_str("t")])]),
            ])
        );
        let setter = render_flow_file(flow.clone(), "SETTER.flow")?;
        assert!(setter.contains("    out v = -1;"), "{}", setter);
        assert!(!setter.contains("in v"), "{}", setter);
        let outer = render_flow_file(flow.clone(), "OUTER.flow")?;
        assert!(!outer.contains("in v"), "{}", outer);
        assert!(!outer.contains("out v"), "{}", outer);
        assert!(outer.contains("v = SETTER.v;"), "{}", outer);
        assert!(outer.contains("READER.v = v;"), "{}", outer);
        let top = render_flow_file(flow, "TOP.flow")?;
        assert!(!top.contains(" v "), "{}", top);
        Ok(())
    }

    #[test]
    fn a_local_variable_declared_as_an_output_is_passed_out_to_every_enclosing_flow(
    ) -> crate::Result<()> {
        let flow = node!(PGM::Flow, "top".to_string() =>
            flow_group("outer", vec![flow_group("setter", vec![
                local_variable("v"),
                node!(PGM::OutputVariable, "v".to_string()),
                set_variable("v", "1"),
            ])])
        );
        for file in ["SETTER.flow", "OUTER.flow", "TOP.flow"] {
            let rendered = render_flow_file(flow.clone(), file)?;
            assert!(
                rendered.contains("    out v = -1;"),
                "{}: {}",
                file,
                rendered
            );
            assert!(!rendered.contains("in v"), "{}: {}", file, rendered);
        }
        Ok(())
    }

    #[test]
    fn an_assigned_variable_which_is_not_declared_local_is_an_input_as_before() -> crate::Result<()>
    {
        let flow = node!(PGM::Flow, "top".to_string() =>
            flow_group("outer", vec![counter(false)])
        );
        let counter = render_flow_file(flow.clone(), "COUNTER.flow")?;
        assert!(counter.contains("    in passed_count = -1;"), "{}", counter);
        let outer = render_flow_file(flow.clone(), "OUTER.flow")?;
        assert!(
            outer.contains("COUNTER.passed_count = passed_count;"),
            "{}",
            outer
        );
        let top = render_flow_file(flow, "TOP.flow")?;
        assert!(top.contains("    in passed_count = -1;"), "{}", top);
        Ok(())
    }

    #[test]
    fn a_local_variable_is_passed_down_to_a_sub_flow_which_reads_it() -> crate::Result<()> {
        let flow = node!(PGM::Flow, "top".to_string() =>
            flow_group("outer", vec![
                local_variable("cnt"),
                set_variable("cnt", "1"),
                flow_group("child", vec![if_expr_node("cnt > 0", vec![set_flag("CHILD_SAW")])]),
            ])
        );
        let outer = render_flow_file(flow.clone(), "OUTER.flow")?;
        assert!(outer.contains("    out cnt = -1;"), "{}", outer);
        assert!(!outer.contains("in cnt"), "{}", outer);
        assert!(outer.contains("CHILD.cnt = cnt;"), "{}", outer);
        let child = render_flow_file(flow.clone(), "CHILD.flow")?;
        assert!(child.contains("    in cnt = -1;"), "{}", child);
        let top = render_flow_file(flow, "TOP.flow")?;
        assert!(!top.contains("cnt"), "{}", top);
        Ok(())
    }

    fn if_enable_node(enable: &str, children: Vec<crate::ast::Node<PGM>>) -> crate::ast::Node<PGM> {
        let mut node = node!(
            PGM::Condition,
            FlowCondition::IfEnable(vec![enable.to_string()])
        );
        for child in children {
            node.add_child(child);
        }
        node
    }

    #[test]
    fn a_local_flag_assigned_as_a_variable_by_another_flow_is_routed_as_before() -> crate::Result<()>
    {
        let flow = node!(PGM::Flow, "top".to_string() =>
            local_flag("X"),
            flow_group("setter", vec![set_flag("X")]),
            flow_group("writer", vec![if_enable_node("w", vec![set_variable("X", "2")])])
        );
        let top = render_flow_file(flow, "TOP.flow")?;
        assert!(top.contains("X = SETTER.X;"), "{}", top);
        assert!(top.contains("WRITER.X = X;"), "{}", top);
        assert!(!top.contains("in X = -1;"), "{}", top);
        Ok(())
    }

    #[test]
    fn a_local_variable_declaration_does_not_apply_to_a_flag_of_its_name() -> crate::Result<()> {
        let flow = |declare: bool| {
            let mut top = node!(PGM::Flow, "top".to_string());
            if declare {
                top.add_child(local_variable("X"));
            }
            top.add_child(flow_group("setter", vec![set_flag("X")]));
            top
        };
        let undeclared = render_flow_file(flow(false), "TOP.flow")?;
        assert!(undeclared.contains("    out X = -1;"), "{}", undeclared);
        assert!(undeclared.contains("X = SETTER.X;"), "{}", undeclared);
        let declared = render_flow_file(flow(true), "TOP.flow")?;
        assert_eq!(declared, undeclared);
        Ok(())
    }

    #[test]
    fn a_local_variable_named_as_an_enable_of_its_flow_is_an_error() {
        let flow = node!(PGM::Flow, "top".to_string() =>
            flow_group("child", vec![
                local_variable("GATE"),
                if_enable_node("gate", vec![set_variable("GATE", "0")]),
            ])
        );
        let error = render_flow_file(flow, "CHILD.flow")
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("The local variable 'GATE' is held by a flow which also needs an input"),
            "{}",
            error
        );
    }

    #[test]
    fn a_local_variable_named_as_an_input_of_a_sub_flow_is_an_error() {
        let consumer = |condition: FlowCondition| {
            let mut node = node!(PGM::Condition, condition);
            node.add_child(test_str("t"));
            flow_group("consumer", vec![node])
        };
        for (name, condition) in [
            ("GATE", FlowCondition::IfEnable(vec!["gate".to_string()])),
            ("JOB", FlowCondition::IfJob(vec!["ft".to_string()])),
        ] {
            let flow = node!(PGM::Flow, "top".to_string() =>
                local_variable(name),
                set_variable(name, "1"),
                consumer(condition)
            );
            let error = render_flow_file(flow, "TOP.flow").unwrap_err().to_string();
            assert!(
                error.contains(&format!(
                    "The local variable '{}' is held by a flow which also needs an input",
                    name
                )),
                "{}",
                error
            );
        }
    }

    #[test]
    fn a_flag_condition_reads_a_local_variable_of_its_name() -> crate::Result<()> {
        let flow = node!(PGM::Flow, "top".to_string() =>
            local_variable("READY"),
            flow_group("setter", vec![set_variable("READY", "1")]),
            flow_group("reader", vec![node!(PGM::Condition, FlowCondition::IfFlag(vec!["READY".to_string()]) =>
                test_str("t")
            )])
        );
        let top = render_flow_file(flow, "TOP.flow")?;
        assert!(top.contains("READY = SETTER.READY;"), "{}", top);
        assert!(top.contains("READER.READY = READY;"), "{}", top);
        assert!(!top.contains("in READY"), "{}", top);
        Ok(())
    }

    fn o1(o1_formatting: bool) -> Options {
        Options {
            o1_formatting,
            ..Default::default()
        }
    }

    /// Renders the given flow AST for SMT8, formatted as Origen 1 does or not, and returns the content
    /// of every flow file by its name
    fn render_flows(
        flow: crate::ast::Node<PGM>,
        o1_formatting: bool,
    ) -> crate::Result<std::collections::HashMap<String, String>> {
        let mut flow_ast = crate::ast::AST::new();
        flow_ast.start(flow);
        let (ast, model) = process_flow(
            &flow_ast,
            Model::new(SupportedTester::V93KSMT8),
            SupportedTester::V93KSMT8,
            true,
        )?;
        let output_dir = tempdir()?;
        let (_model, files) = run_with_options(&ast, output_dir.path(), model, &o1(o1_formatting))?;
        let mut flows = std::collections::HashMap::new();
        for path in files {
            let name = path.file_name().unwrap().to_str().unwrap().to_string();
            if name.ends_with(".flow") {
                flows.insert(name, fs::read_to_string(&path)?);
            }
        }
        Ok(flows)
    }

    fn o1_example() -> crate::ast::Node<PGM> {
        let test = |name: &str| {
            node!(
                PGM::TestStr,
                name.to_string(),
                FlowID::from_str(name),
                None,
                None,
                None
            )
        };
        node!(PGM::Flow, "main".to_string() =>
            node!(PGM::SubFlow, "vmin".to_string(), None =>
                node!(PGM::Condition, FlowCondition::IfEnable(vec!["CONTINUE_ON_FAIL".to_string()]) =>
                    test("t1")
                ),
                node!(PGM::SetFlag, "RUN_FALLBACK_TRUE".to_string(), true, false)
            ),
            node!(PGM::Group, "plain".to_string(), None, GroupType::Flow, Some(FlowID::from_str("plain")) =>
                test("t2")
            ),
            node!(PGM::Condition, FlowCondition::IfFlag(vec!["RUN_FALLBACK_TRUE".to_string()]) =>
                test("t3")
            )
        )
    }

    #[test]
    fn flows_are_formatted_as_origen_1_does_when_asked() -> crate::Result<()> {
        let flows = render_flows(o1_example(), true)?;
        let main = &flows["MAIN.flow"];
        assert!(
            main.contains(
                "    execute {\n\
                \x20       // VMIN sub-flow input variables\n\
                \x20       {\n\
                \x20           VMIN.CONTINUE_ON_FAIL = CONTINUE_ON_FAIL;\n\
                \x20       }\n\
                \x20       VMIN.execute();\n\
                \x20       // VMIN sub-flow output variables\n\
                \x20       {\n\
                \x20           RUN_FALLBACK_TRUE = VMIN.RUN_FALLBACK_TRUE;\n\
                \x20       }\n\
                \x20       PLAIN.execute();\n"
            ),
            "{}",
            main
        );
        // A flag which only a sub-flow sets is declared, but not initialized
        assert!(main.contains("    out RUN_FALLBACK_TRUE = -1;"), "{}", main);
        assert!(
            !main.contains("        RUN_FALLBACK_TRUE = -1;"),
            "{}",
            main
        );
        // The sub-flow which sets it initializes it
        let vmin = &flows["VMIN.flow"];
        assert!(
            vmin.contains("    execute {\n        RUN_FALLBACK_TRUE = -1;\n\n"),
            "{}",
            vmin
        );
        Ok(())
    }

    #[test]
    fn flows_are_formatted_as_before_by_default() -> crate::Result<()> {
        let flows = render_flows(o1_example(), false)?;
        let main = &flows["MAIN.flow"];
        assert!(
            main.contains(
                "    execute {\n\
                \x20       RUN_FALLBACK_TRUE = -1;\n\
                \n\
                \x20       VMIN.CONTINUE_ON_FAIL = CONTINUE_ON_FAIL;\n\
                \x20       VMIN.execute();\n\
                \x20       RUN_FALLBACK_TRUE = VMIN.RUN_FALLBACK_TRUE;\n\
                \x20       PLAIN.execute();\n"
            ),
            "{}",
            main
        );
        assert!(!main.contains("sub-flow"), "{}", main);
        Ok(())
    }

    #[test]
    fn suite_parameters_are_ordered_by_byte_order_when_formatted_as_origen_1_does(
    ) -> crate::Result<()> {
        let mut values = indexmap::IndexMap::new();
        for name in [
            "scan.SSN",
            "logging",
            "outputType",
            "Zed",
            "alpha2",
            "alpha10",
        ] {
            values.insert(name.to_string(), ParamValue::Int(1));
        }
        let render = |o1_formatting: bool| -> crate::Result<Vec<String>> {
            let output_dir = tempdir()?;
            let generator = FlowGenerator::new(
                output_dir.path(),
                Model::new(SupportedTester::V93KSMT8),
                &o1(o1_formatting),
            );
            let mut rendered: Vec<u8> = vec![];
            generator.render_sorted_contents(
                &mut rendered,
                0,
                None,
                &values,
                &indexmap::IndexMap::new(),
                &indexmap::IndexMap::new(),
            )?;
            Ok(String::from_utf8(rendered)
                .unwrap()
                .lines()
                .map(|l| l.split(' ').next().unwrap().to_string())
                .collect())
        };
        assert_eq!(
            render(true)?,
            vec![
                "Zed",
                "alpha10",
                "alpha2",
                "logging",
                "outputType",
                "scan.SSN"
            ]
        );
        assert_eq!(
            render(false)?,
            vec![
                "alpha10",
                "alpha2",
                "logging",
                "outputType",
                "scan.SSN",
                "Zed"
            ]
        );
        Ok(())
    }

    #[test]
    fn collection_parameters_are_in_definition_order_when_formatted_as_origen_1_does(
    ) -> crate::Result<()> {
        use crate::prog_gen::model::{TestCollection, TestCollectionItem};
        use crate::prog_gen::ParamType;
        let mut collection = TestCollection::new("variable");
        for name in ["expression", "outputVariable", "outputType", "logging"] {
            collection
                .params
                .insert(name.to_string(), ParamType::String);
        }
        let mut item = TestCollectionItem::from_collection(100, 1, "param0", &collection);
        for name in ["logging", "outputType", "outputVariable", "expression"] {
            item.values
                .insert(name.to_string(), ParamValue::String(name.to_string()));
        }
        let mut suite_values = indexmap::IndexMap::new();
        for name in ["testDescription.a", "binning.binnable"] {
            suite_values.insert(name.to_string(), ParamValue::Int(1));
        }
        let mut collections = indexmap::IndexMap::new();
        collections.insert("variable".to_string(), vec![100]);

        let render = |o1_formatting: bool| -> crate::Result<Vec<String>> {
            let output_dir = tempdir()?;
            let mut model = Model::new(SupportedTester::V93KSMT8);
            model.test_collection_items.insert(100, item.clone());
            let generator = FlowGenerator::new(output_dir.path(), model, &o1(o1_formatting));
            let mut rendered: Vec<u8> = vec![];
            generator.render_sorted_contents(
                &mut rendered,
                0,
                None,
                &suite_values,
                &indexmap::IndexMap::new(),
                &collections,
            )?;
            Ok(String::from_utf8(rendered)
                .unwrap()
                .lines()
                .map(|l| l.trim().split(' ').next().unwrap().to_string())
                .collect())
        };
        assert_eq!(
            render(true)?,
            vec![
                "binning.binnable",
                "testDescription.a",
                "variable[param0]",
                "expression",
                "outputVariable",
                "outputType",
                "logging",
                "};"
            ]
        );
        assert_eq!(
            render(false)?,
            vec![
                "binning.binnable",
                "testDescription.a",
                "variable[param0]",
                "expression",
                "logging",
                "outputType",
                "outputVariable",
                "};"
            ]
        );
        Ok(())
    }

    #[test]
    fn a_declared_output_is_not_initialized_when_formatted_as_origen_1_does() -> crate::Result<()> {
        let flow = node!(PGM::Flow, "main".to_string() =>
            node!(PGM::OutputVariable, "nativeOut".to_string()),
            node!(PGM::OutputVariable, "BOTH".to_string()),
            node!(PGM::SetFlag, "BOTH".to_string(), true, false),
            node!(PGM::SetFlag, "EXPLICIT".to_string(), true, false)
        );
        let main = render_flows(flow.clone(), true)?
            .remove("MAIN.flow")
            .unwrap();
        assert!(main.contains("    out nativeOut = -1;"), "{}", main);
        assert!(
            main.contains("    execute {\n        BOTH = -1;\n        EXPLICIT = -1;\n\n"),
            "{}",
            main
        );
        assert!(!main.contains("        nativeOut = -1;"), "{}", main);
        // By default, it is initialized as before
        let main = render_flows(flow, false)?.remove("MAIN.flow").unwrap();
        assert!(main.contains("        nativeOut = -1;"), "{}", main);
        Ok(())
    }

    #[test]
    fn a_flag_which_is_only_cleared_is_not_initialized_when_formatted_as_origen_1_does(
    ) -> crate::Result<()> {
        let flow = node!(PGM::Flow, "main".to_string() =>
            node!(PGM::OutputVariable, "CLEARED".to_string()),
            node!(PGM::Condition, FlowCondition::IfEnable(vec!["clear".to_string()]) =>
                node!(PGM::SetFlag, "CLEARED".to_string(), false, false),
                node!(PGM::SetFlag, "BOTH".to_string(), false, false)
            ),
            node!(PGM::SetFlag, "BOTH".to_string(), true, false)
        );
        let main = render_flows(flow.clone(), true)?
            .remove("MAIN.flow")
            .unwrap();
        assert!(
            main.contains("    execute {\n        BOTH = -1;\n\n        if (CLEAR == 1) {"),
            "{}",
            main
        );
        assert!(!main.contains("        CLEARED = -1;"), "{}", main);
        assert!(main.contains("CLEARED = 0;"), "{}", main);
        let main = render_flows(flow, false)?.remove("MAIN.flow").unwrap();
        assert!(main.contains("        CLEARED = -1;"), "{}", main);
        Ok(())
    }

    #[test]
    fn the_flags_a_flow_sets_are_not_serialized_with_its_data() {
        let mut data = super::FlowData::default();
        data.set_flags.insert("X".to_string());
        let serialized = serde_json::to_string(&data).unwrap();
        assert!(!serialized.contains("set_flags"), "{}", serialized);
        assert!(serialized.contains("modified_flags"), "{}", serialized);
    }

    /// Renders the given flow AST for SMT8, with or without empty else branches, and returns the
    /// execute section of its top-level flow file
    fn render_execute(flow: crate::ast::Node<PGM>, empty_else: bool) -> crate::Result<String> {
        let mut flow_ast = crate::ast::AST::new();
        flow_ast.start(flow);
        let (ast, model) = process_flow(
            &flow_ast,
            Model::new(SupportedTester::V93KSMT8),
            SupportedTester::V93KSMT8,
            true,
        )?;
        let output_dir = tempdir()?;
        let (_model, files) = run_with_empty_else(&ast, output_dir.path(), model, empty_else)?;
        let flow = fs::read_to_string(
            files
                .iter()
                .find(|path| path.file_name().and_then(|name| name.to_str()) == Some("MAIN.flow"))
                .expect("expected generated SMT8 flow file"),
        )?;
        Ok(flow.split_once("    execute {\n").unwrap().1.to_string())
    }

    fn condition(
        condition: FlowCondition,
        children: Vec<crate::ast::Node<PGM>>,
    ) -> crate::ast::Node<PGM> {
        let mut node = node!(PGM::Condition, condition);
        for child in children {
            node.add_child(child);
        }
        node
    }

    #[test]
    fn an_empty_else_is_rendered_unless_turned_off() -> crate::Result<()> {
        let flow = node!(PGM::Flow, "main".to_string() =>
            node!(PGM::Loop, Some(LoopCount::Fixed(2)), Some("i".to_string()), false, 1 =>
                condition(FlowCondition::IfExpr("x.pass".to_string()), vec![test_str("t1")])
            ),
            condition(FlowCondition::IfJob(vec!["ws".to_string()]), vec![test_str("t2")]),
            condition(FlowCondition::IfEnable(vec!["e".to_string()]), vec![test_str("t3")]),
            condition(FlowCondition::IfFlag(vec!["f".to_string()]), vec![test_str("t4")])
        );
        let rendered = render_execute(flow.clone(), true)?;
        assert_eq!(
            rendered,
            "        for (i : 1..2) {\n\
            \x20           if (x.pass) {\n\
            \x20               t1.execute();\n\
            \x20           } else {\n\
            \x20           }\n\
            \x20       }\n\
            \x20       if (JOB == \"WS\") {\n\
            \x20           t2.execute();\n\
            \x20       } else {\n\
            \x20       }\n\
            \x20       if (E == 1) {\n\
            \x20           t3.execute();\n\
            \x20       } else {\n\
            \x20       }\n\
            \x20       if (F == 1) {\n\
            \x20           t4.execute();\n\
            \x20       } else {\n\
            \x20       }\n\
            \x20   }\n\
            }\n"
        );
        let rendered = render_execute(flow, false)?;
        assert_eq!(
            rendered,
            "        for (i : 1..2) {\n\
            \x20           if (x.pass) {\n\
            \x20               t1.execute();\n\
            \x20           }\n\
            \x20       }\n\
            \x20       if (JOB == \"WS\") {\n\
            \x20           t2.execute();\n\
            \x20       }\n\
            \x20       if (E == 1) {\n\
            \x20           t3.execute();\n\
            \x20       }\n\
            \x20       if (F == 1) {\n\
            \x20           t4.execute();\n\
            \x20       }\n\
            \x20   }\n\
            }\n"
        );
        Ok(())
    }

    #[test]
    fn an_else_with_content_is_rendered_when_empty_else_is_off() -> crate::Result<()> {
        let flow = node!(PGM::Flow, "main".to_string() =>
            // Content only in the else branch
            condition(FlowCondition::UnlessFlag(vec!["f".to_string()]), vec![test_str("t1")]),
            // Content in both branches, with an empty else nested in the else branch
            condition(FlowCondition::IfExpr("a".to_string()), vec![
                test_str("t2"),
                node!(PGM::Else =>
                    condition(FlowCondition::IfExpr("b".to_string()), vec![test_str("t3")])
                ),
            ])
        );
        let rendered = render_execute(flow, false)?;
        assert_eq!(
            rendered,
            "        if (F == 1) {\n\
            \x20       } else {\n\
            \x20           t1.execute();\n\
            \x20       }\n\
            \x20       if (a) {\n\
            \x20           t2.execute();\n\
            \x20       } else {\n\
            \x20           if (b) {\n\
            \x20               t3.execute();\n\
            \x20           }\n\
            \x20       }\n\
            \x20   }\n\
            }\n"
        );
        Ok(())
    }

    /// Renders the given flow AST for SMT8 and returns the name and content of every flow file
    fn render_flow_files(flow: crate::ast::Node<PGM>) -> crate::Result<Vec<(String, String)>> {
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
        let mut flows = vec![];
        for path in files {
            let name = path.file_name().unwrap().to_str().unwrap().to_string();
            if name.ends_with(".flow") {
                flows.push((name, fs::read_to_string(&path)?));
            }
        }
        Ok(flows)
    }

    fn resources(keep_flows: bool, children: Vec<crate::ast::Node<PGM>>) -> crate::ast::Node<PGM> {
        let mut block = if keep_flows {
            node!(PGM::ResourcesKeepFlows)
        } else {
            node!(PGM::Resources)
        };
        for child in children {
            block.add_child(child);
        }
        block
    }

    fn main_flow(children: Vec<crate::ast::Node<PGM>>) -> crate::ast::Node<PGM> {
        let mut flow = node!(PGM::Flow, "main".to_string());
        for child in children {
            flow.add_child(child);
        }
        flow
    }

    fn sub_flow(name: &str, children: Vec<crate::ast::Node<PGM>>) -> crate::ast::Node<PGM> {
        let mut flow = node!(PGM::SubFlow, name.to_string(), None);
        for child in children {
            flow.add_child(child);
        }
        flow
    }

    fn if_enable(enable: &str, children: Vec<crate::ast::Node<PGM>>) -> crate::ast::Node<PGM> {
        let mut condition = node!(
            PGM::Condition,
            FlowCondition::IfEnable(vec![enable.to_string()])
        );
        for child in children {
            condition.add_child(child);
        }
        condition
    }

    fn if_flag(flag: &str, children: Vec<crate::ast::Node<PGM>>) -> crate::ast::Node<PGM> {
        let mut condition = node!(
            PGM::Condition,
            FlowCondition::IfFlag(vec![flag.to_string()])
        );
        for child in children {
            condition.add_child(child);
        }
        condition
    }

    fn if_expr(expression: &str, children: Vec<crate::ast::Node<PGM>>) -> crate::ast::Node<PGM> {
        let mut condition = node!(
            PGM::Condition,
            FlowCondition::IfExpr(expression.to_string())
        );
        for child in children {
            condition.add_child(child);
        }
        condition
    }

    fn flow_file<'a>(flows: &'a [(String, String)], name: &str) -> &'a str {
        &flows
            .iter()
            .find(|(file, _)| file == name)
            .unwrap_or_else(|| panic!("expected flow file {} in {:?}", name, flows))
            .1
    }

    #[test]
    fn a_flow_within_resources_is_flattened_unless_flows_are_kept() -> crate::Result<()> {
        for kept in [
            flow_group("shared", vec![test_str("t1")]),
            sub_flow("shared", vec![test_str("t1")]),
        ] {
            let flows = render_flow_files(main_flow(vec![resources(false, vec![kept.clone()])]))?;
            assert_eq!(flows.len(), 1, "{:?}", flows);
            let main = flow_file(&flows, "MAIN.flow");
            assert!(!main.contains("SHARED"), "{}", main);
            assert!(!main.contains("t1"), "{}", main);

            let flows = render_flow_files(main_flow(vec![resources(true, vec![kept])]))?;
            assert_eq!(flows.len(), 2, "{:?}", flows);
            let main = flow_file(&flows, "MAIN.flow");
            assert!(
                main.contains(
                    "    setup {\n        flow SHARED calls flows.main.SHARED { }\n    }"
                ),
                "{}",
                main
            );
            assert!(main.contains("    execute {\n    }"), "{}", main);
            assert!(!main.contains("t1"), "{}", main);
            let shared = flow_file(&flows, "SHARED.flow");
            assert!(
                shared.contains("    execute {\n        t1.execute();\n    }"),
                "{}",
                shared
            );
        }
        Ok(())
    }

    #[test]
    fn a_kept_flow_is_declared_but_not_executed_by_the_enclosing_flow() -> crate::Result<()> {
        // The kept flow is within conditions and a non-flow group, it reads an enable and sets a
        // flag which the enclosing flow reads, and it has a result branch
        let mut kept = flow_group(
            "shared",
            vec![if_enable("E", vec![test_str("t1")]), set_flag("F")],
        );
        kept.add_child(node!(PGM::OnFailed, FlowID::from_str("shared") =>
            node!(PGM::Log, "shared failed".to_string())
        ));
        let flows = render_flow_files(main_flow(vec![
            resources(
                true,
                vec![if_flag(
                    "G",
                    vec![
                        node!(PGM::Group, "tests".to_string(), None, GroupType::Test, None =>
                            node!(PGM::BypassSubFlows => kept)
                        ),
                    ],
                )],
            ),
            if_flag("F", vec![test_str("t2")]),
        ]))?;
        let main = flow_file(&flows, "MAIN.flow");
        // Its interface is as if it were executed
        assert!(main.contains("    in E = -1;"), "{}", main);
        assert!(main.contains("    out F = -1;"), "{}", main);
        assert!(
            main.contains(
                "        flow SHARED calls flows.main.SHARED {\n            bypass = true;\n        }"
            ),
            "{}",
            main
        );
        // Nothing executes it, passes it inputs, reads its outputs or tests its result
        assert!(!main.contains("SHARED."), "{}", main);
        assert!(!main.contains("shared failed"), "{}", main);
        assert!(!main.contains("t1"), "{}", main);
        assert!(
            main.contains("if (F == 1) {\n            t2.execute();\n        }"),
            "{}",
            main
        );

        let shared = flow_file(&flows, "SHARED.flow");
        assert!(shared.contains("    in E = -1;"), "{}", shared);
        assert!(shared.contains("    out F = -1;"), "{}", shared);
        assert!(
            shared.contains("if (E == 1) {\n            t1.execute();\n        }"),
            "{}",
            shared
        );
        assert!(shared.contains("F = 1;"), "{}", shared);
        Ok(())
    }

    #[test]
    fn a_flow_within_a_kept_flow_is_executed_by_it() -> crate::Result<()> {
        let flows = render_flow_files(main_flow(vec![resources(
            true,
            vec![flow_group(
                "shared",
                vec![flow_group("inner", vec![test_str("t1")])],
            )],
        )]))?;
        let shared = flow_file(&flows, "SHARED.flow");
        assert!(
            shared.contains("flow INNER calls flows.main.shared.INNER { }"),
            "{}",
            shared
        );
        assert!(shared.contains("INNER.execute();"), "{}", shared);
        let inner = flow_file(&flows, "INNER.flow");
        assert!(inner.contains("t1.execute();"), "{}", inner);
        Ok(())
    }

    #[test]
    fn the_variables_read_within_kept_resources_are_inputs_unless_the_flow_holds_them(
    ) -> crate::Result<()> {
        let block = || {
            vec![
                if_flag("F", vec![test_str("t1")]),
                if_enable("E", vec![test_str("t2")]),
                if_expr("mode == 6", vec![test_str("t3")]),
                node!(PGM::Loop, Some(LoopCount::Fixed(3)), Some("attempt".to_string()), true, 1 =>
                    if_expr("attempt > 1", vec![test_str("t4")])
                ),
            ]
        };
        let flows = render_flow_files(main_flow(vec![resources(true, block())]))?;
        let main = flow_file(&flows, "MAIN.flow");
        assert!(main.contains("    in E = -1;"), "{}", main);
        assert!(main.contains("    in F = -1;"), "{}", main);
        assert!(main.contains("    in mode = -1;"), "{}", main);
        assert!(!main.contains("attempt"), "{}", main);
        // Nothing within the block is rendered
        assert!(main.contains("    execute {\n    }"), "{}", main);

        // A flag which the flow sets is not an input
        let flows = render_flow_files(main_flow(vec![set_flag("F"), resources(true, block())]))?;
        let main = flow_file(&flows, "MAIN.flow");
        assert!(!main.contains("in F"), "{}", main);
        assert!(main.contains("    in E = -1;"), "{}", main);

        // Nor is it when the block sets it, as when the block is executed
        let mut content = block();
        content.push(set_flag("F"));
        let flows = render_flow_files(main_flow(vec![resources(true, content)]))?;
        let main = flow_file(&flows, "MAIN.flow");
        assert!(!main.contains("in F"), "{}", main);
        assert!(main.contains("    out F = -1;"), "{}", main);
        assert!(!main.contains("F = 1;"), "{}", main);

        // Without keeping flows, the content of the block is not analysed
        let flows = render_flow_files(main_flow(vec![resources(false, block())]))?;
        let main = flow_file(&flows, "MAIN.flow");
        assert!(!main.contains(" in "), "{}", main);
        Ok(())
    }

    #[test]
    fn a_test_result_condition_within_kept_resources_does_not_flag_the_test() -> crate::Result<()> {
        let flows = render_flow_files(main_flow(vec![
            test_str("t1"),
            resources(
                true,
                vec![
                    node!(PGM::Condition, FlowCondition::IfFailed(vec![FlowID::from_str("t1")]) =>
                        test_str("t2")
                    ),
                ],
            ),
        ]))?;
        let main = flow_file(&flows, "MAIN.flow");
        assert!(
            main.contains("    execute {\n        t1.execute();\n    }"),
            "{}",
            main
        );
        Ok(())
    }

    #[test]
    fn the_counter_of_a_loop_around_a_kept_flow_is_not_an_input_of_the_enclosing_flow(
    ) -> crate::Result<()> {
        let flows = render_flow_files(main_flow(vec![resources(
            true,
            vec![
                node!(PGM::Loop, Some(LoopCount::Fixed(3)), Some("attempt".to_string()), false, 1 =>
                    flow_group("shared", vec![if_expr("attempt > 1", vec![test_str("t1")])])
                ),
                node!(PGM::Loop, Some(LoopCount::Variable("retryCount".to_string())), Some("retry".to_string()), false, 1 =>
                    flow_group("retried", vec![if_expr("retry > 1", vec![test_str("t2")])])
                ),
            ],
        )]))?;
        let main = flow_file(&flows, "MAIN.flow");
        assert!(!main.contains("attempt"), "{}", main);
        assert!(!main.contains("in retry ="), "{}", main);
        // The count is read by the loop, as when it is executed
        assert!(main.contains("    in retryCount = -1;"), "{}", main);
        assert!(
            main.contains("flow SHARED calls flows.main.SHARED { }"),
            "{}",
            main
        );
        assert!(
            main.contains("flow RETRIED calls flows.main.RETRIED { }"),
            "{}",
            main
        );
        assert!(main.contains("    execute {\n    }"), "{}", main);
        let shared = flow_file(&flows, "SHARED.flow");
        assert!(shared.contains("    in attempt = -1;"), "{}", shared);
        let retried = flow_file(&flows, "RETRIED.flow");
        assert!(retried.contains("    in retry = -1;"), "{}", retried);
        Ok(())
    }

    #[test]
    fn the_conditions_around_kept_resources_do_not_remove_those_within_a_kept_flow(
    ) -> crate::Result<()> {
        let guarded = || vec![if_flag("F", vec![set_flag("DID_RUN")])];
        // Adjacent guards are combined, and the combined node is processed again
        let adjacent = || {
            vec![
                if_flag("F", vec![set_flag("DID_RUN")]),
                if_flag("F", vec![set_flag("DID_RUN_ALSO")]),
            ]
        };
        let one = "if (F == 1) {\n            DID_RUN = 1;\n        }";
        let both =
            "if (F == 1) {\n            DID_RUN = 1;\n            DID_RUN_ALSO = 1;\n        }";
        for (kept, expected) in [
            (flow_group("shared", guarded()), one),
            (sub_flow("shared", guarded()), one),
            (flow_group("shared", adjacent()), both),
            (sub_flow("shared", adjacent()), both),
        ] {
            let flows = render_flow_files(main_flow(vec![if_flag(
                "F",
                vec![resources(true, vec![kept])],
            )]))?;
            let shared = flow_file(&flows, "SHARED.flow");
            assert!(shared.contains("    in F = -1;"), "{}", shared);
            assert!(shared.contains(expected), "{}", shared);
        }
        Ok(())
    }

    #[test]
    fn resources_which_keep_flows_are_plain_resources_on_other_testers() -> crate::Result<()> {
        let content = || {
            vec![if_flag(
                "F",
                vec![
                    flow_group("shared", vec![test_str("t1")]),
                    sub_flow("child", vec![if_enable("E", vec![test_str("t2")])]),
                ],
            )]
        };
        for tester in [
            SupportedTester::V93KSMT7,
            SupportedTester::ULTRAFLEX,
            SupportedTester::J750,
        ] {
            let processed = |keep_flows: bool| -> crate::Result<String> {
                let mut flow_ast = crate::ast::AST::new();
                flow_ast.start(main_flow(vec![resources(keep_flows, content())]));
                let (ast, _model) = process_flow(&flow_ast, Model::new(tester), tester, true)?;
                Ok(ast.to_string())
            };
            assert_eq!(processed(true)?, processed(false)?, "{:?}", tester);
        }
        Ok(())
    }

    #[test]
    fn a_kept_flow_gets_no_o1_variable_blocks_as_it_is_not_executed() -> crate::Result<()> {
        let flow = main_flow(vec![resources(
            true,
            vec![flow_group(
                "shared",
                vec![if_enable("E", vec![test_str("t1")]), set_flag("F")],
            )],
        )]);
        let flows = render_flows(flow, true)?;
        let main = &flows["MAIN.flow"];
        assert!(
            main.contains("flow SHARED calls flows.main.SHARED { }"),
            "{}",
            main
        );
        assert!(!main.contains("sub-flow"), "{}", main);
        assert!(!main.contains("SHARED."), "{}", main);
        let shared = &flows["SHARED.flow"];
        assert!(
            shared.contains("    execute {\n        F = -1;\n"),
            "{}",
            shared
        );
        Ok(())
    }

    fn instance_case_example() -> crate::ast::Node<PGM> {
        let group = |name: &str, id: &str| {
            node!(
                PGM::Group,
                name.to_string(),
                None,
                GroupType::Flow,
                Some(FlowID::from_str(id))
            )
        };
        let mut vmin = group("Vmin 800mV", "vmin");
        vmin.add_child(
            node!(PGM::Condition, FlowCondition::IfEnable(vec!["E".to_string()]) =>
                node!(PGM::TestStr, "t1".to_string(), FlowID::from_str("t1"), None, None, None)
            ),
        );
        vmin.add_child(node!(PGM::SetFlag, "OUT".to_string(), true, false));
        vmin.add_child(node!(PGM::OnFailed, FlowID::from_str("vmin") =>
            node!(PGM::SetFlag, "VMIN_FAILED".to_string(), true, false)
        ));
        let mut again = group("VMIN_800MV", "again");
        again.add_child(node!(
            PGM::TestStr,
            "t2".to_string(),
            FlowID::from_str("t2"),
            None,
            None,
            None
        ));
        let mut bypassed = group("Vmin_800mV", "bypassed");
        bypassed.add_child(node!(
            PGM::TestStr,
            "t3".to_string(),
            FlowID::from_str("t3"),
            None,
            None,
            None
        ));
        node!(PGM::Flow, "main".to_string() =>
            vmin,
            again,
            node!(PGM::BypassSubFlows => bypassed),
            node!(PGM::Condition, FlowCondition::IfFlag(vec!["OUT".to_string()]) =>
                node!(PGM::TestStr, "t4".to_string(), FlowID::from_str("t4"), None, None, None)
            )
        )
    }

    fn render_with(
        flow: crate::ast::Node<PGM>,
        options: &Options,
    ) -> crate::Result<std::collections::HashMap<String, String>> {
        let mut flow_ast = crate::ast::AST::new();
        flow_ast.start(flow);
        let (ast, model) = process_flow(
            &flow_ast,
            Model::new(SupportedTester::V93KSMT8),
            SupportedTester::V93KSMT8,
            true,
        )?;
        let output_dir = tempdir()?;
        let (_model, files) = run_with_options(&ast, output_dir.path(), model, options)?;
        let mut flows = std::collections::HashMap::new();
        for path in files {
            let name = path.file_name().unwrap().to_str().unwrap().to_string();
            if name.ends_with(".flow") {
                flows.insert(name, fs::read_to_string(&path)?);
            }
        }
        Ok(flows)
    }

    #[test]
    fn sub_flow_instance_names_keep_their_case_when_asked() -> crate::Result<()> {
        let flows = render_with(
            instance_case_example(),
            &Options {
                keep_sub_flow_instance_case: true,
                ..Default::default()
            },
        )?;
        let mut files: Vec<&String> = flows.keys().collect();
        files.sort();
        assert_eq!(
            files,
            vec![
                "MAIN.flow",
                "VMIN_800MV.flow",
                "VMIN_800MV_1.flow",
                "VMIN_800MV_2.flow"
            ]
        );
        let main = &flows["MAIN.flow"];
        for line in [
            "        flow Vmin_800mV calls flows.main.VMIN_800MV { }\n",
            "        flow VMIN_800MV_1 calls flows.main.VMIN_800MV_1 { }\n",
            "        flow Vmin_800mV_2 calls flows.main.VMIN_800MV_2 {\n            bypass = true;\n",
            "        Vmin_800mV.E = E;\n        Vmin_800mV.execute();\n",
            "        OUT = Vmin_800mV.OUT;\n",
            "        if (!Vmin_800mV.pass) {\n",
            "        VMIN_800MV_1.execute();\n",
            "        Vmin_800mV_2.execute();\n",
        ] {
            assert!(main.contains(line), "{}: {}", line, main);
        }
        assert!(!main.contains("VMIN_800MV.execute"), "{}", main);
        // The flows' definitions are upper case
        assert!(flows["VMIN_800MV.flow"].starts_with("flow VMIN_800MV {"));
        assert!(flows["VMIN_800MV_2.flow"].starts_with("flow VMIN_800MV_2 {"));

        // And with Origen 1 formatting, its comments name the instance
        let flows = render_with(
            instance_case_example(),
            &Options {
                keep_sub_flow_instance_case: true,
                o1_formatting: true,
                ..Default::default()
            },
        )?;
        let main = &flows["MAIN.flow"];
        assert!(
            main.contains("        // Vmin_800mV sub-flow input variables\n"),
            "{}",
            main
        );
        assert!(
            main.contains("        // Vmin_800mV sub-flow output variables\n"),
            "{}",
            main
        );
        Ok(())
    }

    #[test]
    fn sub_flow_instance_names_are_upper_case_by_default() -> crate::Result<()> {
        let flows = render_with(instance_case_example(), &Options::default())?;
        let main = &flows["MAIN.flow"];
        assert!(
            main.contains("        flow VMIN_800MV calls flows.main.VMIN_800MV { }\n"),
            "{}",
            main
        );
        assert!(
            main.contains("        flow VMIN_800MV_2 calls flows.main.VMIN_800MV_2 {\n"),
            "{}",
            main
        );
        assert!(
            main.contains("        VMIN_800MV.E = E;\n        VMIN_800MV.execute();\n"),
            "{}",
            main
        );
        assert!(
            main.contains("        if (!VMIN_800MV.pass) {\n"),
            "{}",
            main
        );
        assert!(!main.contains("Vmin"), "{}", main);
        Ok(())
    }

    #[test]
    fn a_test_path_is_made_of_the_sub_flow_instance_names() -> crate::Result<()> {
        let output_dir = tempdir()?;
        for (keep_case, expected) in [(false, "VMIN_800MV"), (true, "Vmin_800mV")] {
            let mut generator = FlowGenerator::new(
                output_dir.path(),
                Model::new(SupportedTester::V93KSMT8),
                &Options {
                    keep_sub_flow_instance_case: keep_case,
                    ..Default::default()
                },
            );
            generator.open_flow_file("main", Default::default())?;
            generator.open_flow_file("Vmin 800mV", Default::default())?;
            assert_eq!(generator.current_flow_path().as_deref(), Some(expected));
        }
        Ok(())
    }

    #[test]
    fn a_sub_flow_instance_named_as_a_suite_is_an_error() {
        let mut flow = super::FlowFile {
            name: "MAIN".to_string(),
            ..Default::default()
        };
        flow.test_ids.push(("Vmin".to_string(), 1));
        flow.sub_flows
            .push(("VMIN".to_string(), "VMIN".to_string()));
        assert!(flow.check_instance_names().is_ok());
        flow.sub_flows
            .push(("Vmin".to_string(), "VMIN_1".to_string()));
        let error = flow.check_instance_names().unwrap_err().to_string();
        assert!(
            error
                .contains("the test suite 'Vmin' and the sub-flow (or flow group) instance 'Vmin'"),
            "{}",
            error
        );
        assert!(error.contains("SMT8 flow 'MAIN'"), "{}", error);
    }

    #[test]
    fn a_suite_and_a_group_with_the_same_kept_case_name_are_an_error() -> crate::Result<()> {
        // The test method is added to the model directly, rather than from a template on the
        // load path, which is global
        let model = || {
            let mut model = Model::new(SupportedTester::V93KSMT8);
            let mut method =
                crate::prog_gen::model::Test::new("vmin_tm", 1, SupportedTester::V93KSMT8);
            method.class_name = Some("instance_case_lib.T".to_string());
            model.tests.insert(1, method);
            model
        };
        let flow = || {
            node!(PGM::Flow, "main".to_string() =>
                node!(PGM::DefTestInv, 2, "Vmin".to_string(), SupportedTester::V93KSMT8),
                node!(PGM::AssignTestToInv, 2, 1),
                node!(PGM::SetAttr, 2, "name".to_string(), Some(ParamValue::Any("Vmin".to_string())), false),
                node!(PGM::Test, 2, FlowID::from_str("vmin_suite")),
                node!(PGM::Group, "Vmin".to_string(), None, GroupType::Flow, Some(FlowID::from_str("child")) =>
                    node!(PGM::TestStr, "t1".to_string(), FlowID::from_str("t1"), None, None, None)
                )
            )
        };
        let render = |options: &Options| -> crate::Result<String> {
            let mut flow_ast = crate::ast::AST::new();
            flow_ast.start(flow());
            let (ast, model) = process_flow(&flow_ast, model(), SupportedTester::V93KSMT8, true)?;
            let output_dir = tempdir()?;
            let (_model, files) = run_with_options(&ast, output_dir.path(), model, options)?;
            let main = files
                .iter()
                .find(|path| path.file_name().and_then(|n| n.to_str()) == Some("MAIN.flow"))
                .unwrap();
            Ok(fs::read_to_string(main)?)
        };

        let error = render(&Options {
            keep_sub_flow_instance_case: true,
            ..Default::default()
        })
        .unwrap_err()
        .to_string();
        assert!(
            error
                .contains("the test suite 'Vmin' and the sub-flow (or flow group) instance 'Vmin'"),
            "{}",
            error
        );
        let main = render(&Options::default())?;
        assert!(
            main.contains("        suite Vmin calls instance_case_lib.T {"),
            "{}",
            main
        );
        assert!(
            main.contains("        flow VMIN calls flows.main.VMIN { }"),
            "{}",
            main
        );
        Ok(())
    }

    /// The flow 'top' with the given content, preceded by the definition of the test suite
    /// 'check', see check_test
    fn check_flow(children: Vec<crate::ast::Node<PGM>>) -> crate::ast::Node<PGM> {
        let mut nodes = vec![
            node!(
                PGM::DefTestInv,
                2,
                "check".to_string(),
                SupportedTester::V93KSMT8
            ),
            node!(PGM::AssignTestToInv, 2, 1),
            node!(
                PGM::SetAttr,
                2,
                "name".to_string(),
                Some(ParamValue::Any("check".to_string())),
                false
            ),
        ];
        nodes.extend(children);
        crate::ast::Node::new_with_children(PGM::Flow("top".to_string()), nodes)
    }

    /// Runs the test suite 'check', continuing when it fails
    fn check_test() -> Vec<crate::ast::Node<PGM>> {
        vec![
            node!(PGM::Test, 2, FlowID::from_str("check")),
            node!(PGM::OnFailed, FlowID::from_str("check") => node!(PGM::Continue)),
        ]
    }

    /// Sets the flags RESULT_PASSED and RESULT_FAILED from the result of the test 'check',
    /// optionally declaring them local first
    fn check_result_reader(local: bool) -> Vec<crate::ast::Node<PGM>> {
        let mut nodes = result_declarations(local);
        nodes.push(
            node!(PGM::Condition, FlowCondition::IfPassed(vec![FlowID::from_str("check")]) =>
                node!(PGM::SetFlag, "result_passed".to_string(), true, false)
            ),
        );
        nodes.push(
            node!(PGM::Condition, FlowCondition::IfFailed(vec![FlowID::from_str("check")]) =>
                node!(PGM::SetFlag, "result_failed".to_string(), true, false)
            ),
        );
        nodes
    }

    /// Declares the flags RESULT_PASSED and RESULT_FAILED local, when local
    fn result_declarations(local: bool) -> Vec<crate::ast::Node<PGM>> {
        if local {
            vec![
                node!(PGM::LocalFlag, "result_passed".to_string()),
                node!(PGM::LocalFlag, "result_failed".to_string()),
            ]
        } else {
            vec![]
        }
    }

    /// Renders the given flow for SMT8, with a test method for the suite 'check', and returns the
    /// content of every flow file by its name
    fn render_check_flows(
        flow: crate::ast::Node<PGM>,
        o1_formatting: bool,
    ) -> crate::Result<std::collections::HashMap<String, String>> {
        let mut model = Model::new(SupportedTester::V93KSMT8);
        let mut method =
            crate::prog_gen::model::Test::new("check_tm", 1, SupportedTester::V93KSMT8);
        method.class_name = Some("result_lib.T".to_string());
        model.tests.insert(1, method);
        let mut flow_ast = crate::ast::AST::new();
        flow_ast.start(flow);
        let (ast, model) = process_flow(&flow_ast, model, SupportedTester::V93KSMT8, true)?;
        let output_dir = tempdir()?;
        let (_model, files) = run_with_options(&ast, output_dir.path(), model, &o1(o1_formatting))?;
        let mut flows = std::collections::HashMap::new();
        for path in files {
            let name = path.file_name().unwrap().to_str().unwrap().to_string();
            if name.ends_with(".flow") {
                flows.insert(name, fs::read_to_string(&path)?);
            }
        }
        Ok(flows)
    }

    #[test]
    fn a_test_result_read_in_the_flow_of_the_test_is_one_branch_on_it() -> crate::Result<()> {
        for local in [false, true] {
            // Declared at the start of the flow, as a flow declares its local flags
            let mut inner = result_declarations(local);
            inner.extend(check_test());
            inner.extend(check_result_reader(false));
            let flow = check_flow(vec![flow_group("outer", vec![flow_group("inner", inner)])]);
            let flows = render_check_flows(flow, false)?;
            let inner = &flows["INNER.flow"];
            assert!(
                inner.contains(
                    "        check.execute();\n        if (check.pass) {\n            RESULT_PASSED = 1;\n        } else {\n            RESULT_FAILED = 1;\n        }\n"
                ),
                "{}",
                inner
            );
            for name in ["TOP.flow", "OUTER.flow", "INNER.flow"] {
                assert!(!flows[name].contains("CHECK_"), "{}", flows[name]);
            }
            if local {
                assert!(
                    !flows["OUTER.flow"].contains("RESULT_"),
                    "{}",
                    flows["OUTER.flow"]
                );
                assert!(
                    !flows["TOP.flow"].contains("RESULT_"),
                    "{}",
                    flows["TOP.flow"]
                );
            }
        }
        Ok(())
    }

    #[test]
    fn a_test_result_read_by_a_sibling_flow_is_passed_to_it() -> crate::Result<()> {
        for local in [false, true] {
            for o1_formatting in [false, true] {
                let flow = check_flow(vec![flow_group(
                    "outer",
                    vec![
                        flow_group("inner", check_test()),
                        flow_group("setter", check_result_reader(local)),
                    ],
                )]);
                let flows = render_check_flows(flow, o1_formatting)?;
                let inner = &flows["INNER.flow"];
                assert!(
                    inner.contains("    out CHECK_FAILED = -1;\n    out CHECK_PASSED = -1;\n"),
                    "{}",
                    inner
                );
                let outer = &flows["OUTER.flow"];
                let (copy, pass) = if o1_formatting {
                    (
                        "        // INNER sub-flow output variables\n        {\n            CHECK_FAILED = INNER.CHECK_FAILED;\n            CHECK_PASSED = INNER.CHECK_PASSED;\n        }\n",
                        "        // SETTER sub-flow input variables\n        {\n            SETTER.CHECK_FAILED = CHECK_FAILED;\n            SETTER.CHECK_PASSED = CHECK_PASSED;\n        }\n",
                    )
                } else {
                    (
                        "        CHECK_FAILED = INNER.CHECK_FAILED;\n        CHECK_PASSED = INNER.CHECK_PASSED;\n",
                        "        SETTER.CHECK_FAILED = CHECK_FAILED;\n        SETTER.CHECK_PASSED = CHECK_PASSED;\n",
                    )
                };
                assert!(
                    outer.contains(&format!("        INNER.execute();\n{}{}", copy, pass)),
                    "{}",
                    outer
                );
                assert!(!outer.contains("in CHECK_"), "{}", outer);
                assert!(
                    !flows["TOP.flow"].contains("CHECK_"),
                    "{}",
                    flows["TOP.flow"]
                );
                let setter = &flows["SETTER.flow"];
                assert!(setter.contains("    in CHECK_PASSED = -1;\n"), "{}", setter);
                assert!(
                    setter.contains("        if (CHECK_PASSED == 1) {\n"),
                    "{}",
                    setter
                );
                if local {
                    assert!(!outer.contains("RESULT_"), "{}", outer);
                    assert!(
                        !flows["TOP.flow"].contains("RESULT_"),
                        "{}",
                        flows["TOP.flow"]
                    );
                }
            }
        }
        Ok(())
    }

    #[test]
    fn a_test_result_read_within_a_sibling_flow_is_passed_down_to_its_reader() -> crate::Result<()>
    {
        let flow = check_flow(vec![flow_group(
            "outer",
            vec![
                flow_group("inner", check_test()),
                flow_group(
                    "wrapper",
                    vec![flow_group("setter", check_result_reader(false))],
                ),
            ],
        )]);
        let flows = render_check_flows(flow, false)?;
        assert!(
            flows["INNER.flow"].contains("    out CHECK_PASSED = -1;\n"),
            "{}",
            flows["INNER.flow"]
        );
        let outer = &flows["OUTER.flow"];
        assert!(
            outer.contains("        CHECK_PASSED = INNER.CHECK_PASSED;\n"),
            "{}",
            outer
        );
        assert!(
            outer.contains("        WRAPPER.CHECK_PASSED = CHECK_PASSED;\n"),
            "{}",
            outer
        );
        assert!(!outer.contains("in CHECK_"), "{}", outer);
        let wrapper = &flows["WRAPPER.flow"];
        assert!(
            wrapper.contains("    in CHECK_PASSED = -1;\n"),
            "{}",
            wrapper
        );
        assert!(
            wrapper.contains("        SETTER.CHECK_PASSED = CHECK_PASSED;\n"),
            "{}",
            wrapper
        );
        assert!(
            !flows["TOP.flow"].contains("CHECK_"),
            "{}",
            flows["TOP.flow"]
        );
        Ok(())
    }

    #[test]
    fn a_test_result_is_held_by_the_highest_flow_with_another_reader() -> crate::Result<()> {
        let flow = check_flow(vec![
            flow_group("producer", vec![flow_group("inner", check_test())]),
            flow_group(
                "consumer",
                vec![flow_group("setter", check_result_reader(false))],
            ),
        ]);
        let flows = render_check_flows(flow, false)?;
        assert!(
            flows["INNER.flow"].contains("    out CHECK_PASSED = -1;\n"),
            "{}",
            flows["INNER.flow"]
        );
        let producer = &flows["PRODUCER.flow"];
        assert!(
            producer.contains("    out CHECK_PASSED = -1;\n"),
            "{}",
            producer
        );
        assert!(
            producer.contains("        CHECK_PASSED = INNER.CHECK_PASSED;\n"),
            "{}",
            producer
        );
        let top = &flows["TOP.flow"];
        assert!(
            top.contains("        CHECK_PASSED = PRODUCER.CHECK_PASSED;\n"),
            "{}",
            top
        );
        assert!(
            top.contains("        CONSUMER.CHECK_PASSED = CHECK_PASSED;\n"),
            "{}",
            top
        );
        assert!(!top.contains("in CHECK_"), "{}", top);
        assert!(
            flows["CONSUMER.flow"].contains("    in CHECK_PASSED = -1;\n"),
            "{}",
            flows["CONSUMER.flow"]
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
