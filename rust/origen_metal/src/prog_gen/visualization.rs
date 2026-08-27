use super::{BinType, FlowCondition, Model, ParamType, ParamValue, SupportedTester, Test, PGM};
use crate::ast::Node;
use crate::Result;
use indexmap::IndexMap;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

const SCHEMA_VERSION: u8 = 1;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct FlowGraph {
    pub schema_version: u8,
    pub name: String,
    pub tester: String,
    pub nodes: Vec<FlowGraphNode>,
    pub edges: Vec<FlowGraphEdge>,
    pub tests: IndexMap<String, TestDetails>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct FlowGraphNode {
    pub id: String,
    pub kind: String,
    pub label: String,
    pub detail_id: Option<String>,
    pub source: Option<String>,
    pub depth: usize,
    pub order: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct FlowGraphEdge {
    pub from: String,
    pub to: String,
    pub kind: String,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TestDetails {
    pub invocation: Option<String>,
    pub method: String,
    pub library: Option<String>,
    pub template: Option<String>,
    pub class_name: Option<String>,
    pub parameters: Vec<ParameterDetails>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ParameterDetails {
    pub scope: String,
    pub path: String,
    pub kind: String,
    pub value: Option<String>,
    pub default: Option<String>,
    pub configured: bool,
    pub aliases: Vec<String>,
    pub constraints: Vec<String>,
}

#[derive(Default)]
struct SequenceResult {
    entry: Option<String>,
    exits: Vec<String>,
}

struct GraphBuilder<'a> {
    graph: FlowGraph,
    model: &'a Model,
    next_node: usize,
}

impl FlowGraph {
    pub fn from_processed_ast(
        name: &str,
        tester: SupportedTester,
        ast: &Node<PGM>,
        model: &Model,
    ) -> Self {
        let mut builder = GraphBuilder {
            graph: FlowGraph {
                schema_version: SCHEMA_VERSION,
                name: name.to_string(),
                tester: tester.to_string(),
                nodes: Vec::new(),
                edges: Vec::new(),
                tests: IndexMap::new(),
            },
            model,
            next_node: 0,
        };
        if matches!(ast.attrs, PGM::Flow(_)) {
            builder.build_sequence(&ast.children, 0);
        } else {
            builder.build_node(ast, 0);
        }
        builder.graph
    }
}

impl<'a> GraphBuilder<'a> {
    fn build_sequence(&mut self, nodes: &[Box<Node<PGM>>], depth: usize) -> SequenceResult {
        let mut result = SequenceResult::default();
        let mut previous_exits: Vec<String> = Vec::new();

        for node in nodes {
            let current = self.build_node(node, depth);
            let Some(entry) = current.entry.clone() else {
                continue;
            };
            if result.entry.is_none() {
                result.entry = Some(entry.clone());
            }
            for previous in &previous_exits {
                self.push_edge(previous, &entry, "next", None);
            }
            previous_exits = current.exits;
        }
        result.exits = previous_exits;
        result
    }

    fn build_node(&mut self, node: &Node<PGM>, depth: usize) -> SequenceResult {
        let Some((kind, label, test_id)) = describe_node(&node.attrs, self.model) else {
            return self.build_sequence(&node.children, depth);
        };

        let id = format!("n{}", self.next_node);
        self.next_node += 1;
        let detail_id = test_id.map(|test_id| self.add_test_details(test_id));
        self.graph.nodes.push(FlowGraphNode {
            id: id.clone(),
            kind: kind.to_string(),
            label,
            detail_id,
            source: node.meta.as_ref().and_then(|meta| {
                meta.filename.as_ref().map(|filename| match meta.lineno {
                    Some(line) => format!("{}:{}", filename, line),
                    None => filename.clone(),
                })
            }),
            depth,
            order: self.graph.nodes.len(),
        });

        if node.children.is_empty() {
            return SequenceResult {
                entry: Some(id.clone()),
                exits: if is_terminal(&node.attrs) {
                    Vec::new()
                } else {
                    vec![id]
                },
            };
        }

        if matches!(node.attrs, PGM::Test(_, _) | PGM::Cz(_, _, _)) {
            let mut exits = vec![id.clone()];
            for child in &node.children {
                if let Some(branch_kind) = result_handler_kind(&child.attrs) {
                    let branch = self.build_node(child, depth + 1);
                    if let Some(entry) = branch.entry {
                        self.push_edge(&id, &entry, branch_kind, Some(branch_kind));
                    }
                    exits.extend(branch.exits);
                } else {
                    let branch = self.build_node(child, depth + 1);
                    if let Some(entry) = branch.entry {
                        self.push_edge(&id, &entry, "child", None);
                    }
                    exits.extend(branch.exits);
                }
            }
            return SequenceResult {
                entry: Some(id),
                exits,
            };
        }

        let children = self.build_sequence(&node.children, depth + 1);
        if let Some(entry) = children.entry {
            let label = block_edge_label(&node.attrs);
            self.push_edge(&id, &entry, block_edge_kind(&node.attrs), label);
        }

        let mut exits = children.exits;
        if has_fallthrough(&node.attrs) {
            exits.push(id.clone());
        }
        if matches!(node.attrs, PGM::Loop(_, _)) {
            for exit in &exits {
                if exit != &id {
                    self.push_edge(exit, &id, "loop", Some("repeat"));
                }
            }
            exits = vec![id.clone()];
        }
        if exits.is_empty() && !is_terminal(&node.attrs) {
            exits.push(id.clone());
        }
        SequenceResult {
            entry: Some(id),
            exits,
        }
    }

    fn add_test_details(&mut self, id: usize) -> String {
        let detail_id = format!("test-{}", id);
        if self.graph.tests.contains_key(&detail_id) {
            return detail_id;
        }

        let invocation = self.model.test_invocations.get(&id);
        let method = invocation
            .and_then(|invocation| invocation.test_id)
            .and_then(|test_id| self.model.tests.get(&test_id))
            .or_else(|| self.model.tests.get(&id));

        let details = match method {
            Some(method) => {
                let mut parameters = parameter_details("method", "", method);
                if let Some(invocation) = invocation {
                    parameters.extend(parameter_details("invocation", "", invocation));
                }
                TestDetails {
                    invocation: invocation.map(|invocation| invocation.name.clone()),
                    method: method.name.clone(),
                    library: method.template_library.clone(),
                    template: method.template_name.clone(),
                    class_name: method.class_name.clone(),
                    parameters,
                }
            }
            None => TestDetails {
                invocation: None,
                method: format!("Test {}", id),
                library: None,
                template: None,
                class_name: None,
                parameters: Vec::new(),
            },
        };
        self.graph.tests.insert(detail_id.clone(), details);
        detail_id
    }

    fn push_edge(&mut self, from: &str, to: &str, kind: &str, label: Option<&str>) {
        if from == to
            || self
                .graph
                .edges
                .iter()
                .any(|edge| edge.from == from && edge.to == to && edge.kind == kind)
        {
            return;
        }
        self.graph.edges.push(FlowGraphEdge {
            from: from.to_string(),
            to: to.to_string(),
            kind: kind.to_string(),
            label: label.map(str::to_string),
        });
    }
}

fn parameter_details(scope: &str, prefix: &str, test: &Test) -> Vec<ParameterDetails> {
    let mut parameters = Vec::new();
    for (name, kind) in &test.params {
        let value = test.values.get(name);
        let default = test.default_values.get(name);
        let configured = match (value, default) {
            (Some(value), Some(default)) => value != default,
            (Some(_), None) => true,
            _ => false,
        };
        parameters.push(ParameterDetails {
            scope: scope.to_string(),
            path: join_path(prefix, name),
            kind: parameter_kind(kind),
            value: if configured {
                value.map(parameter_value)
            } else {
                None
            },
            default: default.map(parameter_value),
            configured,
            aliases: aliases_for(name, &test.aliases),
            constraints: test
                .constraints
                .get(name)
                .map(|constraints| {
                    constraints
                        .iter()
                        .map(|constraint| format!("{:?}", constraint))
                        .collect()
                })
                .unwrap_or_default(),
        });
    }
    for (collection_name, collection) in &test.collection_defs {
        collect_collection_parameters(
            scope,
            &join_path(prefix, collection_name),
            collection,
            &mut parameters,
        );
    }
    parameters.sort_by(|a, b| a.scope.cmp(&b.scope).then(a.path.cmp(&b.path)));
    parameters
}

fn collect_collection_parameters(
    scope: &str,
    prefix: &str,
    collection: &super::TestCollection,
    parameters: &mut Vec<ParameterDetails>,
) {
    for (name, kind) in &collection.params {
        parameters.push(ParameterDetails {
            scope: scope.to_string(),
            path: join_path(prefix, name),
            kind: parameter_kind(kind),
            value: None,
            default: collection.default_values.get(name).map(parameter_value),
            configured: false,
            aliases: aliases_for(name, &collection.aliases),
            constraints: collection
                .constraints
                .get(name)
                .map(|constraints| {
                    constraints
                        .iter()
                        .map(|constraint| format!("{:?}", constraint))
                        .collect()
                })
                .unwrap_or_default(),
        });
    }
    for (name, nested) in &collection.collections {
        collect_collection_parameters(scope, &join_path(prefix, name), nested, parameters);
    }
}

fn aliases_for(name: &str, aliases: &IndexMap<String, String>) -> Vec<String> {
    let mut result: Vec<String> = aliases
        .iter()
        .filter_map(|(alias, target)| {
            if target == name {
                Some(alias.clone())
            } else {
                None
            }
        })
        .collect();
    result.sort();
    result
}

fn join_path(prefix: &str, name: &str) -> String {
    if prefix.is_empty() {
        name.to_string()
    } else {
        format!("{}.{}", prefix, name)
    }
}

fn parameter_kind(kind: &ParamType) -> String {
    format!("{:?}", kind)
}

fn parameter_value(value: &ParamValue) -> String {
    match value {
        ParamValue::String(value) | ParamValue::Class(value) | ParamValue::Any(value) => {
            value.clone()
        }
        ParamValue::Bool(value) => value.to_string(),
        ParamValue::Int(value) => value.to_string(),
        ParamValue::UInt(value) => value.to_string(),
        ParamValue::Float(value) => value.to_string(),
        ParamValue::Current(value) => format!("{} A", value),
        ParamValue::Voltage(value) => format!("{} V", value),
        ParamValue::Time(value) => format!("{} s", value),
        ParamValue::Frequency(value) => format!("{} Hz", value),
        ParamValue::List(values) => values
            .iter()
            .map(parameter_value)
            .collect::<Vec<_>>()
            .join(", "),
    }
}

fn describe_node(attrs: &PGM, model: &Model) -> Option<(&'static str, String, Option<usize>)> {
    Some(match attrs {
        PGM::Nil
        | PGM::DefTest(_, _, _, _, _)
        | PGM::DefTestInv(_, _, _)
        | PGM::AssignTestToInv(_, _)
        | PGM::DefTestCollectionItem(_, _, _, _, _)
        | PGM::SetAttr(_, _, _, _)
        | PGM::SetLimit(_, _, _, _)
        | PGM::DefSubTest(_, _, _, _, _)
        | PGM::PatternGroup(_, _, _, _)
        | PGM::PushPattern(_, _, _)
        | PGM::DefBin(_, _, _, _, _)
        | PGM::Resources
        | PGM::ResourcesFilename(_, _)
        | PGM::FlowDescription(_)
        | PGM::FlowNameOverride(_)
        | PGM::Namespace(_)
        | PGM::Uniqueness(_)
        | PGM::IGXLResource(_)
        | PGM::IGXLResourcesFilename(_, _)
        | PGM::Variable(_, _, _)
        | PGM::Parameter(_, _, _) => return None,
        PGM::Flow(name) => ("flow", name.clone(), None),
        PGM::SubFlow(name, _) => ("subflow", format!("Subflow: {}", name), None),
        PGM::Test(id, flow_id) => {
            let name = model
                .test_invocations
                .get(id)
                .or_else(|| model.tests.get(id))
                .map(|test| test.name.clone())
                .unwrap_or_else(|| format!("Test {}", id));
            ("test", format!("{}\n{}", name, flow_id.to_str()), Some(*id))
        }
        PGM::TestStr(name, flow_id, _, _, _) => {
            ("test", format!("{}\n{}", name, flow_id.to_str()), None)
        }
        PGM::Cz(id, setup, flow_id) => {
            let name = model
                .test_invocations
                .get(id)
                .or_else(|| model.tests.get(id))
                .map(|test| test.name.clone())
                .unwrap_or_else(|| format!("Test {}", id));
            (
                "test",
                format!("{}\nCZ: {}\n{}", name, setup, flow_id.to_str()),
                Some(*id),
            )
        }
        PGM::Render(text) => ("action", truncate(text, 72), None),
        PGM::Log(text) => ("action", format!("Log: {}", truncate(text, 64)), None),
        PGM::Group(name, _, kind, _) => ("group", format!("{:?} group: {}", kind, name), None),
        PGM::Condition(condition) => ("condition", condition_label(condition), None),
        PGM::Bin(number, soft, kind) => (
            match kind {
                BinType::Good => "bin-good",
                BinType::Bad => "bin-bad",
            },
            match soft {
                Some(soft) => format!("{:?} bin {} / soft {}", kind, number, soft),
                None => format!("{:?} bin {}", kind, number),
            },
            None,
        ),
        PGM::OnFailed(_) => ("handler-fail", "On fail".to_string(), None),
        PGM::OnPassed(_) => ("handler-pass", "On pass".to_string(), None),
        PGM::OnError(_) => ("handler-error", "On error".to_string(), None),
        PGM::TesterEq(testers) => ("condition", format!("Tester is {:?}", testers), None),
        PGM::TesterNeq(testers) => ("condition", format!("Tester is not {:?}", testers), None),
        PGM::Volatile(flag) => ("action", format!("Volatile flag: {}", flag), None),
        PGM::SetFlag(flag, value, _) => ("action", format!("Set {} = {}", flag, value), None),
        PGM::SetDefaultFlagState(flag, value) => {
            ("action", format!("Default {} = {}", flag, value), None)
        }
        PGM::Continue => ("action", "Continue on failure".to_string(), None),
        PGM::Delayed => ("action", "Delay binning".to_string(), None),
        PGM::Else => ("condition", "Else".to_string(), None),
        PGM::Whenever => ("condition", "Whenever".to_string(), None),
        PGM::WheneverAny => ("condition", "Whenever any".to_string(), None),
        PGM::WheneverAll => ("condition", "Whenever all".to_string(), None),
        PGM::Enable(flag) => ("action", format!("Enable {}", flag), None),
        PGM::Disable(flag) => ("action", format!("Disable {}", flag), None),
        PGM::BypassSubFlows => ("action", "Bypass subflows".to_string(), None),
        PGM::IGXLSetWaitFlags(id, flags) => (
            "action",
            format!("Wait flags {}: {}", id, flags.join(", ")),
            None,
        ),
        PGM::FlowData(_) => ("action", "Tester flow data".to_string(), None),
        PGM::Unknown(kind, raw) => ("action", format!("{}: {}", kind, truncate(raw, 56)), None),
        PGM::Comment(text) => ("note", truncate(text, 72), None),
        PGM::Wait(duration) => ("action", format!("Wait {}", duration), None),
        PGM::SetVariable(name, value) => ("action", format!("Set {} = {}", name, value), None),
        PGM::Synchronize => ("action", "Synchronize sites".to_string(), None),
        PGM::Label(label) => ("label", format!("Label: {}", label), None),
        PGM::Goto(label) => ("action", format!("Go to {}", label), None),
        PGM::Call(name, args) => (
            "action",
            format!("Call {}({})", name, args.join(", ")),
            None,
        ),
        PGM::Loop(count, variable) => (
            "condition",
            format!(
                "Loop {}{}",
                count
                    .map(|count| count.to_string())
                    .unwrap_or_else(|| "until complete".to_string()),
                variable
                    .as_ref()
                    .map(|variable| format!(" as {}", variable))
                    .unwrap_or_default()
            ),
            None,
        ),
        PGM::Report(category, message) => (
            "action",
            format!("Report {}: {}", category, truncate(message, 56)),
            None,
        ),
        PGM::Assertion(expression, message) => (
            "condition",
            format!("Assert {}: {}", expression, truncate(message, 48)),
            None,
        ),
        PGM::Callback(name, args) => (
            "action",
            format!("Callback {}({})", name, args.join(", ")),
            None,
        ),
    })
}

fn condition_label(condition: &FlowCondition) -> String {
    match condition {
        FlowCondition::IfJob(values) => format!("If job: {}", values.join(", ")),
        FlowCondition::UnlessJob(values) => format!("Unless job: {}", values.join(", ")),
        FlowCondition::IfEnable(values) => format!("If enabled: {}", values.join(", ")),
        FlowCondition::UnlessEnable(values) => format!("Unless enabled: {}", values.join(", ")),
        FlowCondition::IfFlag(values) => format!("If flag: {}", values.join(", ")),
        FlowCondition::UnlessFlag(values) => format!("Unless flag: {}", values.join(", ")),
        FlowCondition::IfExpr(expression) => format!("If {}", expression),
        FlowCondition::UnlessExpr(expression) => format!("Unless {}", expression),
        FlowCondition::IfVar(name, operator, value) => {
            format!("If {} {} {}", name, operator, value)
        }
        FlowCondition::UnlessVar(name, operator, value) => {
            format!("Unless {} {} {}", name, operator, value)
        }
        FlowCondition::IfSite(sites) => format!("If site: {:?}", sites),
        FlowCondition::UnlessSite(sites) => format!("Unless site: {:?}", sites),
        _ => format!("{:?}", condition),
    }
}

fn result_handler_kind(attrs: &PGM) -> Option<&'static str> {
    match attrs {
        PGM::OnPassed(_) => Some("pass"),
        PGM::OnFailed(_) => Some("fail"),
        PGM::OnError(_) => Some("error"),
        _ => None,
    }
}

fn block_edge_label(attrs: &PGM) -> Option<&'static str> {
    match attrs {
        PGM::Condition(_) | PGM::TesterEq(_) | PGM::TesterNeq(_) => Some("yes"),
        PGM::OnPassed(_) => Some("pass"),
        PGM::OnFailed(_) => Some("fail"),
        PGM::OnError(_) => Some("error"),
        PGM::Loop(_, _) => Some("body"),
        _ => None,
    }
}

fn block_edge_kind(attrs: &PGM) -> &'static str {
    match attrs {
        PGM::OnPassed(_) => "pass",
        PGM::OnFailed(_) => "fail",
        PGM::OnError(_) => "error",
        PGM::Loop(_, _) => "loop",
        _ => "branch",
    }
}

fn has_fallthrough(attrs: &PGM) -> bool {
    matches!(
        attrs,
        PGM::Condition(_)
            | PGM::TesterEq(_)
            | PGM::TesterNeq(_)
            | PGM::OnPassed(_)
            | PGM::OnFailed(_)
            | PGM::OnError(_)
            | PGM::Loop(_, _)
    )
}

fn is_terminal(attrs: &PGM) -> bool {
    matches!(attrs, PGM::Bin(_, _, _) | PGM::Goto(_))
}

fn truncate(value: &str, maximum: usize) -> String {
    let mut chars = value.chars();
    let shortened: String = chars.by_ref().take(maximum).collect();
    if chars.next().is_some() {
        format!("{}...", shortened)
    } else {
        shortened
    }
}

pub fn render_flow_visualization(
    name: &str,
    tester: SupportedTester,
    ast: &Node<PGM>,
    model: &Model,
    output_dir: &Path,
) -> Result<Vec<PathBuf>> {
    let graph = FlowGraph::from_processed_ast(name, tester, ast, model);
    let directory = output_dir.join("flow_visualizations");
    fs::create_dir_all(&directory)?;
    let stem = safe_filename(name);
    let json_path = directory.join(format!("{}.flow.json", stem));
    let html_path = directory.join(format!("{}.flow.html", stem));
    let json = serde_json::to_string_pretty(&graph).map_err(|error| {
        crate::Error::new(&format!("Failed to serialize flow graph: {}", error))
    })?;
    fs::write(&json_path, &json)?;
    fs::write(
        &html_path,
        FLOW_HTML
            .replace("__FLOW_TITLE__", &escape_html(name))
            .replace("__FLOW_DATA__", &json.replace("</", "<\\/")),
    )?;
    Ok(vec![json_path, html_path])
}

fn safe_filename(name: &str) -> String {
    let filename: String = name
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.') {
                character
            } else {
                '_'
            }
        })
        .collect();
    if filename.is_empty() {
        "flow".to_string()
    } else {
        filename
    }
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

const FLOW_HTML: &str = r###"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>__FLOW_TITLE__ · Origen Flow</title>
<style>
:root {
  color-scheme: dark;
  --bg: #071019;
  --surface: #0c1924;
  --surface-raised: #122432;
  --line: #294354;
  --text: #e8f1f5;
  --muted: #9ab0bd;
  --primary: #4dc6e7;
  --primary-ink: #031116;
  --accent: #f0a43c;
  --danger: #ff6b6b;
  --success: #55d697;
  --focus: #ffd166;
  --shadow: 0 16px 42px rgba(0,0,0,.34);
}
@media (prefers-color-scheme: light) {
  :root {
    color-scheme: light;
    --bg: #edf3f6;
    --surface: #ffffff;
    --surface-raised: #f7fafb;
    --line: #bfd0d9;
    --text: #102733;
    --muted: #536b78;
    --primary: #087c9d;
    --primary-ink: #ffffff;
    --accent: #a95d00;
    --danger: #b4232d;
    --success: #087b4a;
    --focus: #8a5800;
    --shadow: 0 16px 38px rgba(21,54,70,.13);
  }
}
* { box-sizing: border-box; }
html, body { margin: 0; min-height: 100%; background: var(--bg); color: var(--text); }
body { font-family: "Fira Sans", "Avenir Next", "Segoe UI", sans-serif; font-size: 15px; }
button, input { font: inherit; }
button { touch-action: manipulation; }
.app { min-height: 100vh; display: grid; grid-template-rows: auto 1fr; }
header {
  display: grid;
  grid-template-columns: minmax(220px, 1fr) minmax(260px, 520px) auto;
  gap: 18px;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid var(--line);
  background: color-mix(in srgb, var(--surface) 92%, transparent);
  position: sticky;
  top: 0;
  z-index: 20;
  backdrop-filter: blur(12px);
}
.brand { min-width: 0; }
.eyebrow { color: var(--primary); font: 600 11px/1.2 "Fira Code", ui-monospace, monospace; letter-spacing: .14em; text-transform: uppercase; }
h1 { margin: 4px 0 0; font-size: clamp(18px, 2.2vw, 27px); line-height: 1.1; overflow-wrap: anywhere; }
.search { position: relative; }
.search label { position: absolute; width: 1px; height: 1px; overflow: hidden; clip: rect(0 0 0 0); }
.search input {
  width: 100%;
  min-height: 44px;
  border: 1px solid var(--line);
  border-radius: 8px;
  padding: 0 14px;
  color: var(--text);
  background: var(--surface-raised);
  outline: none;
}
.search input:focus { border-color: var(--focus); box-shadow: 0 0 0 3px color-mix(in srgb, var(--focus) 28%, transparent); }
.toolbar { display: flex; gap: 8px; }
.toolbar button {
  min-width: 44px;
  min-height: 44px;
  border: 1px solid var(--line);
  border-radius: 8px;
  color: var(--text);
  background: var(--surface-raised);
  cursor: pointer;
}
.toolbar button:hover { border-color: var(--primary); color: var(--primary); }
.toolbar button:focus-visible { outline: 3px solid var(--focus); outline-offset: 2px; }
.workspace { min-height: 0; display: grid; grid-template-columns: minmax(0, 1fr) 430px; }
.graph-shell { min-width: 0; min-height: 0; overflow: auto; position: relative; }
.graph-shell::before {
  content: "";
  position: fixed;
  inset: 76px 430px 0 0;
  pointer-events: none;
  background-image: radial-gradient(circle at center, color-mix(in srgb, var(--line) 55%, transparent) 1px, transparent 1px);
  background-size: 24px 24px;
  opacity: .5;
}
#graph { position: relative; display: block; }
.detail {
  border-left: 1px solid var(--line);
  background: var(--surface);
  padding: 20px;
  overflow: auto;
  position: sticky;
  top: 77px;
  height: calc(100vh - 77px);
}
.detail h2 { margin: 0; font-size: 19px; }
.detail .hint { color: var(--muted); line-height: 1.55; margin-top: 10px; }
.meta-grid { display: grid; grid-template-columns: 92px 1fr; gap: 8px 10px; margin: 18px 0; font-size: 13px; }
.meta-grid dt { color: var(--muted); }
.meta-grid dd { margin: 0; font-family: "Fira Code", ui-monospace, monospace; overflow-wrap: anywhere; }
.param-summary { display: flex; gap: 8px; margin: 18px 0 10px; color: var(--muted); font-size: 12px; }
.param-table { width: 100%; border-collapse: collapse; table-layout: fixed; font-size: 12px; }
.param-table th { text-align: left; color: var(--muted); font-weight: 500; border-bottom: 1px solid var(--line); padding: 8px 6px; }
.param-table th:nth-child(1) { width: 50%; }
.param-table th:nth-child(2), .param-table th:nth-child(3) { width: 25%; }
.param-table td { vertical-align: top; border-bottom: 1px solid color-mix(in srgb, var(--line) 65%, transparent); padding: 9px 6px; overflow-wrap: anywhere; }
.param-table code { font-family: "Fira Code", ui-monospace, monospace; color: var(--text); }
.badge { display: inline-block; border: 1px solid var(--line); border-radius: 999px; padding: 2px 6px; color: var(--muted); font-size: 10px; margin: 2px 3px 2px 0; }
.badge.configured { border-color: var(--primary); color: var(--primary); }
.edge { fill: none; stroke: var(--line); stroke-width: 1.5; marker-end: url(#arrow); }
.edge.pass { stroke: var(--success); }
.edge.fail, .edge.error { stroke: var(--danger); }
.edge.loop { stroke: var(--accent); stroke-dasharray: 5 4; }
.edge-label { fill: var(--muted); font: 600 11px "Fira Code", ui-monospace, monospace; paint-order: stroke; stroke: var(--bg); stroke-width: 5px; }
.node { cursor: default; }
.node.interactive { cursor: pointer; }
.node rect { fill: var(--surface-raised); stroke: var(--line); stroke-width: 1.5; rx: 8; filter: drop-shadow(0 7px 10px rgba(0,0,0,.18)); }
.node text { fill: var(--text); font: 500 13px "Fira Code", ui-monospace, monospace; pointer-events: none; }
.node .kind { fill: var(--muted); font-size: 9px; font-weight: 700; letter-spacing: .12em; }
.node.test rect { stroke: var(--primary); }
.node.test .kind { fill: var(--primary); }
.node.condition rect, .node.handler-pass rect, .node.handler-fail rect, .node.handler-error rect { stroke: var(--accent); }
.node.bin-bad rect { stroke: var(--danger); }
.node.bin-good rect { stroke: var(--success); }
.node.group rect, .node.subflow rect { stroke: #a78bfa; }
.node.note rect { stroke-dasharray: 4 4; }
.node:hover rect { stroke-width: 2.5; }
.node:focus { outline: none; }
.node:focus rect { stroke: var(--focus); stroke-width: 3; }
.node.dim { opacity: .18; }
.empty { border: 1px dashed var(--line); border-radius: 10px; padding: 18px; color: var(--muted); }
.stats { color: var(--muted); font: 500 11px "Fira Code", ui-monospace, monospace; margin-top: 7px; }
@media (max-width: 900px) {
  header { grid-template-columns: 1fr auto; }
  .search { grid-column: 1 / -1; grid-row: 2; }
  .workspace { grid-template-columns: 1fr; }
  .detail { position: static; height: auto; min-height: 320px; border-left: 0; border-top: 1px solid var(--line); }
  .graph-shell { min-height: 58vh; }
  .graph-shell::before { position: absolute; inset: 0; height: auto; }
}
@media (max-width: 600px) {
  header { grid-template-columns: 1fr; }
  .toolbar { grid-row: 2; justify-content: flex-start; }
  .search { grid-row: 3; }
}
@media (prefers-reduced-motion: reduce) { * { scroll-behavior: auto !important; transition: none !important; } }
</style>
</head>
<body>
<div class="app">
  <header>
    <div class="brand">
      <div class="eyebrow">Origen · processed test flow</div>
      <h1>__FLOW_TITLE__</h1>
      <div class="stats" id="stats"></div>
    </div>
    <div class="search">
      <label for="node-search">Find a flow node</label>
      <input id="node-search" type="search" placeholder="Find tests, conditions, bins..." autocomplete="off">
    </div>
    <div class="toolbar" aria-label="Graph controls">
      <button type="button" id="sections" aria-label="Expand all sections">Expand sections</button>
      <button type="button" id="zoom-out" aria-label="Zoom out">−</button>
      <button type="button" id="zoom-reset" aria-label="Reset zoom">100%</button>
      <button type="button" id="zoom-in" aria-label="Zoom in">+</button>
      <button type="button" id="download-json" aria-label="Download flow data">JSON</button>
    </div>
  </header>
  <main class="workspace">
    <section class="graph-shell" aria-label="Test flow graph">
      <svg id="graph" role="img" aria-labelledby="graph-title"></svg>
    </section>
    <aside class="detail" id="detail" aria-live="polite">
      <h2 id="detail-title">Select a test</h2>
      <p class="hint">Choose a cyan test node to inspect its method library, configured values, defaults, aliases, and constraints.</p>
    </aside>
  </main>
</div>
<script>
const FLOW_DATA = __FLOW_DATA__;
const NS = "http://www.w3.org/2000/svg";
const nodeWidth = 230;
const nodeHeight = 72;
const xGap = 282;
const yGap = 108;
let zoom = 1;
let baseWidth = 0;
let baseHeight = 0;
let nodeElements = [];
const collapsedSections = new Set();
const svg = document.getElementById("graph");
const graphShell = document.querySelector(".graph-shell");
const searchInput = document.getElementById("node-search");
const positions = new Map();
const sectionKinds = new Set(["group", "subflow"]);
const ancestorsByNode = buildAncestors(FLOW_DATA.nodes);

function element(name, attributes = {}) {
  const node = document.createElementNS(NS, name);
  for (const [key, value] of Object.entries(attributes)) node.setAttribute(key, value);
  return node;
}
function escapeHtml(value) {
  return String(value ?? "").replace(/[&<>"']/g, character => ({"&":"&amp;","<":"&lt;",">":"&gt;",'"':"&quot;","'":"&#39;"})[character]);
}
function lines(value) {
  const result = [];
  for (const raw of String(value).split("\n")) {
    let line = raw;
    while (line.length > 30) { result.push(line.slice(0, 30)); line = line.slice(30); }
    result.push(line);
  }
  return result.slice(0, 3);
}
function isSection(node) {
  return sectionKinds.has(node.kind);
}
function buildAncestors(nodes) {
  const ancestors = new Map();
  const sections = [];
  nodes.forEach(node => {
    while (sections.length && node.depth <= sections[sections.length - 1].depth) sections.pop();
    ancestors.set(node.id, sections.map(section => section.id));
    if (isSection(node)) sections.push(node);
  });
  return ancestors;
}
function visibleGraph() {
  const remappedIds = new Map();
  const nodes = FLOW_DATA.nodes.filter(node => {
    const ancestors = ancestorsByNode.get(node.id) || [];
    const collapsedAncestor = ancestors.find(id => collapsedSections.has(id));
    remappedIds.set(node.id, collapsedAncestor || node.id);
    return !collapsedAncestor;
  });
  const edgeKeys = new Set();
  const edges = [];
  FLOW_DATA.edges.forEach(edge => {
    const from = remappedIds.get(edge.from);
    const to = remappedIds.get(edge.to);
    if (!from || !to || from === to) return;
    const key = `${from}|${to}|${edge.kind}`;
    if (edgeKeys.has(key)) return;
    edgeKeys.add(key);
    edges.push({...edge, from, to});
  });
  return {nodes, edges};
}
function setGraphSize() {
  svg.setAttribute("width", baseWidth * zoom);
  svg.setAttribute("height", baseHeight * zoom);
  document.getElementById("zoom-reset").textContent = `${Math.round(zoom * 100)}%`;
}
function updateSectionsButton() {
  const button = document.getElementById("sections");
  button.textContent = collapsedSections.size ? "Expand sections" : "Collapse sections";
  button.setAttribute("aria-label", collapsedSections.size ? "Expand all sections" : "Collapse all sections");
}
function currentQuery() {
  return searchInput.value.trim().toLowerCase();
}
function expandMatchingAncestors(query) {
  let expanded = false;
  if (!query) return expanded;
  FLOW_DATA.nodes.filter(node => node.label.toLowerCase().includes(query)).forEach(node => {
    (ancestorsByNode.get(node.id) || []).forEach(sectionId => {
      if (collapsedSections.delete(sectionId)) expanded = true;
    });
  });
  return expanded;
}
function applySearchStyles(scrollToMatch) {
  const query = currentQuery();
  nodeElements.forEach(node => node.classList.toggle("dim", query && !node.dataset.label.includes(query)));
  const first = nodeElements.find(node => query && node.dataset.label.includes(query));
  if (scrollToMatch && first) first.scrollIntoView({block:"center", inline:"center", behavior:matchMedia("(prefers-reduced-motion: reduce)").matches ? "auto" : "smooth"});
}
function toggleSection(nodeId) {
  if (collapsedSections.has(nodeId)) collapsedSections.delete(nodeId);
  else collapsedSections.add(nodeId);
  expandMatchingAncestors(currentQuery());
  renderGraph();
}
function renderGraph() {
  svg.replaceChildren();
  positions.clear();
  nodeElements = [];
  const visible = visibleGraph();
  const maxDepth = Math.max(0, ...visible.nodes.map(node => node.depth));
  baseWidth = 80 + (maxDepth + 1) * xGap;
  baseHeight = 70 + Math.max(1, visible.nodes.length) * yGap;
  svg.setAttribute("viewBox", `0 0 ${baseWidth} ${baseHeight}`);
  setGraphSize();
  const title = element("title", {id: "graph-title"});
  title.textContent = `${FLOW_DATA.name} test flow`;
  svg.append(title);
  const defs = element("defs");
  const marker = element("marker", {id:"arrow", viewBox:"0 0 10 10", refX:"9", refY:"5", markerWidth:"7", markerHeight:"7", orient:"auto-start-reverse"});
  marker.append(element("path", {d:"M 0 0 L 10 5 L 0 10 z", fill:"context-stroke"}));
  defs.append(marker);
  svg.append(defs);
  const viewport = element("g");
  svg.append(viewport);
  visible.nodes.forEach((node, index) => {
    positions.set(node.id, {x: 36 + node.depth * xGap, y: 36 + index * yGap});
  });
  visible.edges.forEach(edge => {
    const source = positions.get(edge.from);
    const target = positions.get(edge.to);
    if (!source || !target) return;
    const sx = source.x + nodeWidth / 2;
    const sy = source.y + nodeHeight;
    const tx = target.x + nodeWidth / 2;
    const ty = target.y;
    const midpoint = sy + Math.max(18, (ty - sy) / 2);
    const path = element("path", {d:`M ${sx} ${sy} L ${sx} ${midpoint} L ${tx} ${midpoint} L ${tx} ${ty}`, class:`edge ${edge.kind}`});
    viewport.append(path);
    if (edge.label) {
      const label = element("text", {x:(sx + tx) / 2 + 5, y:midpoint - 6, class:"edge-label"});
      label.textContent = edge.label;
      viewport.append(label);
    }
  });
  visible.nodes.forEach(node => {
    const position = positions.get(node.id);
    const section = isSection(node);
    const interactive = section || node.detail_id;
    const group = element("g", {
      class:`node ${node.kind}${interactive ? " interactive" : ""}`,
      transform:`translate(${position.x} ${position.y})`,
      "data-label":node.label.toLowerCase(),
      "data-node-id":node.id
    });
    group.append(element("rect", {width:nodeWidth, height:nodeHeight}));
    const kind = element("text", {x:"13", y:"17", class:"kind"});
    kind.textContent = section ? `${node.kind.toUpperCase()} · ${collapsedSections.has(node.id) ? "COLLAPSED" : "EXPANDED"}` : node.kind.replaceAll("-", " ").toUpperCase();
    group.append(kind);
    lines(node.label).forEach((line, index) => {
      const text = element("text", {x:"13", y:String(39 + index * 16)});
      text.textContent = line;
      group.append(text);
    });
    const title = element("title");
    title.textContent = node.source ? `${node.label}\n${node.source}` : node.label;
    group.append(title);
    if (interactive) {
      group.setAttribute("role", "button");
      group.setAttribute("tabindex", "0");
      if (section) {
        group.setAttribute("aria-label", `${collapsedSections.has(node.id) ? "Expand" : "Collapse"} ${node.label}`);
        group.setAttribute("aria-expanded", String(!collapsedSections.has(node.id)));
      } else {
        group.setAttribute("aria-label", `Inspect ${node.label}`);
      }
      const activate = () => section ? toggleSection(node.id) : showDetails(node.detail_id);
      group.addEventListener("click", activate);
      group.addEventListener("keydown", event => {
        if (event.key === "Enter" || event.key === " ") { event.preventDefault(); activate(); }
      });
    }
    nodeElements.push(group);
    viewport.append(group);
  });
  document.getElementById("stats").textContent = `${FLOW_DATA.tester} · ${visible.nodes.length} of ${FLOW_DATA.nodes.length} nodes · ${FLOW_DATA.tests ? Object.keys(FLOW_DATA.tests).length : 0} tests`;
  updateSectionsButton();
  applySearchStyles(false);
}
function showDetails(detailId) {
  const detail = FLOW_DATA.tests[detailId];
  if (!detail) return;
  const configured = detail.parameters.filter(parameter => parameter.configured).length;
  document.getElementById("detail").innerHTML = `
    <h2 id="detail-title">${escapeHtml(detail.invocation || detail.method)}</h2>
    <dl class="meta-grid">
      <dt>Method</dt><dd>${escapeHtml(detail.method)}</dd>
      <dt>Library</dt><dd>${escapeHtml(detail.library || "not recorded")}</dd>
      <dt>Template</dt><dd>${escapeHtml(detail.template || "not recorded")}</dd>
      <dt>Class</dt><dd>${escapeHtml(detail.class_name || "not specified")}</dd>
    </dl>
    <div class="param-summary"><span>${detail.parameters.length} library parameters</span><span>${configured} configured</span></div>
    ${detail.parameters.length ? `<table class="param-table"><thead><tr><th>Parameter</th><th>Type / scope</th><th>Value</th></tr></thead><tbody>${detail.parameters.map(parameter => `
      <tr><td><code>${escapeHtml(parameter.path)}</code>${parameter.aliases.map(alias => `<span class="badge">${escapeHtml(alias)}</span>`).join("")}${parameter.constraints.map(constraint => `<span class="badge">${escapeHtml(constraint)}</span>`).join("")}</td>
      <td>${escapeHtml(parameter.kind)}<br><span class="badge">${escapeHtml(parameter.scope)}</span>${parameter.configured ? '<span class="badge configured">configured</span>' : ''}</td>
      <td><code>${escapeHtml(parameter.value ?? parameter.default ?? "—")}</code>${parameter.value !== null && parameter.default !== null ? `<br><span class="badge">default ${escapeHtml(parameter.default)}</span>` : ''}</td></tr>`).join("")}</tbody></table>` : '<div class="empty">No parameter library was associated with this test.</div>'}`;
}
function setZoom(value) {
  zoom = Math.min(1.8, Math.max(.55, value));
  setGraphSize();
}
document.getElementById("sections").addEventListener("click", () => {
  if (collapsedSections.size) collapsedSections.clear();
  else FLOW_DATA.nodes.filter(isSection).forEach(node => collapsedSections.add(node.id));
  expandMatchingAncestors(currentQuery());
  renderGraph();
});
document.getElementById("zoom-in").addEventListener("click", () => setZoom(zoom + .15));
document.getElementById("zoom-out").addEventListener("click", () => setZoom(zoom - .15));
document.getElementById("zoom-reset").addEventListener("click", () => setZoom(1));
searchInput.addEventListener("input", () => {
  if (expandMatchingAncestors(currentQuery())) renderGraph();
  applySearchStyles(true);
});
document.getElementById("download-json").addEventListener("click", () => {
  const url = URL.createObjectURL(new Blob([JSON.stringify(FLOW_DATA, null, 2)], {type:"application/json"}));
  const link = document.createElement("a");
  link.href = url;
  link.download = `${FLOW_DATA.name.replace(/[^a-z0-9_.-]+/gi, "_")}.flow.json`;
  link.click();
  URL.revokeObjectURL(url);
});
FLOW_DATA.nodes.filter(isSection).forEach(node => collapsedSections.add(node.id));
renderGraph();
</script>
</body>
</html>
"###;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prog_gen::{FlowID, GroupType, ParamType};

    fn sample() -> (Node<PGM>, Model) {
        let mut model = Model::new(SupportedTester::V93KSMT8);
        let mut method = Test::new("continuity_method", 10, SupportedTester::V93KSMT8);
        method.template_library = Some("dc_tml".to_string());
        method.template_name = Some("continuity".to_string());
        method.class_name = Some("dc_tml.Continuity".to_string());
        method
            .params
            .insert("forceCurrent".to_string(), ParamType::Current);
        method
            .default_values
            .insert("forceCurrent".to_string(), ParamValue::Current(0.0001));
        method
            .values
            .insert("forceCurrent".to_string(), ParamValue::Current(0.0002));
        model.tests.insert(10, method);
        let mut invocation = Test::new("continuity_1", 20, SupportedTester::V93KSMT8);
        invocation.test_id = Some(10);
        model.test_invocations.insert(20, invocation);

        let test = Node::new_with_children(
            PGM::Test(20, FlowID::from_str("continuity")),
            vec![Node::new_with_children(
                PGM::OnFailed(FlowID::from_str("continuity")),
                vec![Node::new(PGM::Bin(
                    10,
                    Some(110),
                    super::super::BinType::Bad,
                ))],
            )],
        );
        let ast = Node::new_with_children(
            PGM::Flow("main".to_string()),
            vec![
                Node::new_with_children(
                    PGM::Group(
                        "Production".to_string(),
                        None,
                        GroupType::Flow,
                        Some(FlowID::from_str("production_group")),
                    ),
                    vec![Node::new_with_children(
                        PGM::Condition(FlowCondition::IfEnable(vec!["production".to_string()])),
                        vec![test],
                    )],
                ),
                Node::new_with_children(
                    PGM::SubFlow(
                        "cleanup".to_string(),
                        Some(FlowID::from_str("cleanup_subflow")),
                    ),
                    vec![Node::new(PGM::Log("Flow complete".to_string()))],
                ),
            ],
        );
        (ast, model)
    }

    #[test]
    fn graph_contains_clickable_test_details_and_typed_edges() {
        let (ast, model) = sample();
        let graph = FlowGraph::from_processed_ast("main", SupportedTester::V93KSMT8, &ast, &model);
        assert_eq!(graph.tests.len(), 1);
        let details = graph.tests.values().next().unwrap();
        assert_eq!(details.library.as_deref(), Some("dc_tml"));
        assert_eq!(details.template.as_deref(), Some("continuity"));
        assert_eq!(details.parameters[0].path, "forceCurrent");
        assert!(details.parameters[0].configured);
        let fail_edges: Vec<&FlowGraphEdge> = graph
            .edges
            .iter()
            .filter(|edge| edge.label.as_deref() == Some("fail"))
            .collect();
        assert_eq!(fail_edges.len(), 2);
        assert!(fail_edges.iter().all(|edge| edge.kind == "fail"));
        assert!(graph.nodes.iter().any(|node| node.kind == "condition"));
        assert!(graph.nodes.iter().any(|node| node.kind == "bin-bad"));
        assert!(graph.nodes.iter().any(|node| node.kind == "group"));
        assert!(graph.nodes.iter().any(|node| node.kind == "subflow"));
    }

    #[test]
    fn model_preserves_loaded_template_identity() {
        let mut model = Model::new(SupportedTester::V93KSMT7);
        model.create_flow("main").unwrap();
        model.select_flow("main").unwrap();
        model
            .add_test_from_template(
                1,
                "continuity_method".to_string(),
                &SupportedTester::V93KSMT7,
                "continuity",
                Some("dc_tml"),
            )
            .unwrap();
        let test = &model.tests[&1];
        assert_eq!(test.template_library.as_deref(), Some("dc_tml"));
        assert_eq!(test.template_name.as_deref(), Some("continuity"));
        assert!(!test.params.is_empty());
        let ast = Node::new_with_children(
            PGM::Flow("main".to_string()),
            vec![Node::new(PGM::Test(1, FlowID::from_str("continuity")))],
        );
        let graph = FlowGraph::from_processed_ast("main", SupportedTester::V93KSMT7, &ast, &model);
        let details = graph.tests.values().next().unwrap();
        assert_eq!(details.library.as_deref(), Some("dc_tml"));
        assert!(details.parameters.len() > 5);
    }

    #[test]
    fn writes_self_contained_json_and_html() {
        let (ast, model) = sample();
        let directory = tempfile::tempdir().unwrap();
        let files = render_flow_visualization(
            "main",
            SupportedTester::V93KSMT8,
            &ast,
            &model,
            directory.path(),
        )
        .unwrap();
        assert_eq!(files.len(), 2);
        let html = fs::read_to_string(&files[1]).unwrap();
        assert!(html.contains("const FLOW_DATA ="));
        assert!(html.contains("continuity_method"));
        assert!(html.contains("forceCurrent"));
        assert!(!html.contains("https://"));
        assert!(html.contains("const collapsedSections = new Set()"));
        assert!(html.contains("const sectionKinds = new Set([\"group\", \"subflow\"])"));
        assert!(html.contains("function visibleGraph()"));
        assert!(html.contains("visible.nodes.length} of ${FLOW_DATA.nodes.length} nodes"));
        let set_zoom = html
            .split("function setZoom(value) {")
            .nth(1)
            .unwrap()
            .split_once('}')
            .unwrap()
            .0;
        assert!(set_zoom.contains("setGraphSize()"));
        assert!(!set_zoom.contains("renderGraph()"));

        if let Ok(output) = std::env::var("ORIGEN_FLOW_VISUALIZATION_TEST_OUTPUT") {
            fs::create_dir_all(&output).unwrap();
            fs::copy(&files[0], Path::new(&output).join("main.flow.json")).unwrap();
            fs::copy(&files[1], Path::new(&output).join("main.flow.html")).unwrap();
        }
    }
}
