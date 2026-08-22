use super::instances::build_instance_names;
use super::patterns::{Patset, PatsetPattern};
use super::resources::ResourceRow;
use crate::ast::{Node, Processor, Return};
use crate::prog_gen::{
    BinType, FlowCondition, IGXLResourceKind, Limit, Model, PatternGroupType, ResourcesType,
    SupportedTester, PGM,
};
use crate::Result;
use indexmap::IndexMap;
use std::collections::HashMap;

#[derive(Clone, Default)]
struct Gate {
    enable: Option<String>,
    job: Option<String>,
    device_sense: Option<String>,
    device_condition: Option<String>,
    device_name: Option<String>,
    group_specifier: Option<String>,
    group_sense: Option<String>,
    group_condition: Option<String>,
    group_name: Option<String>,
}

pub(super) struct FlowGenerator {
    pub(super) model: Model,
    pub(super) rows: Vec<String>,
    gates: Vec<Gate>,
    resources: bool,
    pub(super) patsets: IndexMap<usize, Patset>,
    pub(super) wait_flags: HashMap<usize, Vec<String>>,
    label_counter: usize,
    pub(super) resources_rows: Vec<ResourceRow>,
    resource_filename: String,
    resource_filenames: HashMap<IGXLResourceKind, String>,
    group_results: Vec<(String, String)>,
    pub(super) instance_names: HashMap<usize, String>,
}

impl FlowGenerator {
    pub(super) fn new(model: Model) -> Self {
        let instance_names = build_instance_names(&model);
        Self {
            model,
            rows: vec![],
            gates: vec![],
            resources: false,
            patsets: IndexMap::new(),
            wait_flags: HashMap::new(),
            label_counter: 0,
            resources_rows: vec![],
            resource_filename: "global".to_string(),
            resource_filenames: HashMap::new(),
            group_results: vec![],
            instance_names,
        }
    }

    fn emit(&mut self, mut fields: Vec<String>) {
        if self.resources {
            return;
        }
        fields.resize(32, String::new());
        for gate in &self.gates {
            merge_field(&mut fields[1], gate.enable.as_deref());
            merge_field(&mut fields[2], gate.job.as_deref());
            if let Some(value) = &gate.device_sense {
                fields[26] = value.clone();
            }
            if let Some(value) = &gate.device_condition {
                fields[27] = value.clone();
            }
            if let Some(value) = &gate.device_name {
                merge_field(&mut fields[28], Some(value));
            }
            if let Some(value) = &gate.group_specifier {
                fields[22] = value.clone();
            }
            if let Some(value) = &gate.group_sense {
                fields[23] = value.clone();
            }
            if let Some(value) = &gate.group_condition {
                fields[24] = value.clone();
            }
            if let Some(value) = &gate.group_name {
                merge_field(&mut fields[25], Some(value));
            }
        }
        self.rows.push(format!("\t{}", fields.join("\t")));
    }

    fn simple_row(&mut self, opcode: &str, parameter: &str) {
        let mut row = vec![String::new(); 32];
        row[5] = opcode.to_string();
        row[6] = sanitize(parameter);
        self.emit(row);
    }

    fn test_row(
        &mut self,
        invocation_id: usize,
        opcode: &str,
        cz_setup: Option<&str>,
        flow_id: &crate::prog_gen::FlowID,
        flag_pass: bool,
        flag_fail: bool,
    ) -> Result<()> {
        let invocation = self
            .model
            .test_invocations
            .get(&invocation_id)
            .ok_or_else(|| {
                crate::Error::new(&format!("No UltraFLEX flow line with ID {}", invocation_id))
            })?;
        let test = invocation.test_id.and_then(|id| self.model.tests.get(&id));
        let mut row = vec![String::new(); 32];
        let test_parameter = match (test, cz_setup) {
            (Some(test), Some(setup)) => format!(
                "{} {}",
                self.instance_names.get(&test.id).unwrap_or(&test.name),
                setup
            ),
            (Some(test), None) => self
                .instance_names
                .get(&test.id)
                .cloned()
                .unwrap_or_else(|| test.name.clone()),
            (None, _) => invocation.name.clone(),
        };
        row[7] = invocation
            .tname
            .clone()
            .unwrap_or_else(|| invocation.name.clone());
        row[8] = invocation.number.map(|n| n.to_string()).unwrap_or_default();
        let mapped_fields = [
            (0, "label"),
            (1, "enable"),
            (2, "job"),
            (3, "part"),
            (4, "env"),
            (5, "opcode"),
            (6, "parameter"),
            (7, "tname"),
            (9, "lolim"),
            (10, "hilim"),
            (11, "scale"),
            (12, "units"),
            (13, "format"),
            (14, "bin_pass"),
            (15, "bin_fail"),
            (16, "sort_pass"),
            (17, "sort_fail"),
            (18, "result"),
            (19, "flag_pass"),
            (20, "flag_fail"),
            (21, "state"),
            (22, "group_specifier"),
            (23, "group_sense"),
            (24, "group_condition"),
            (25, "group_name"),
            (26, "device_sense"),
            (27, "device_condition"),
            (28, "device_name"),
            (29, "debug_assume"),
            (30, "debug_sites"),
        ];
        for (column, name) in mapped_fields {
            if let Some(value) = invocation.get(name)? {
                row[column] = value.to_string();
            }
        }
        let lo_limit = invocation
            .lo_limit
            .as_ref()
            .or_else(|| test.and_then(|t| t.lo_limit.as_ref()));
        let hi_limit = invocation
            .hi_limit
            .as_ref()
            .or_else(|| test.and_then(|t| t.hi_limit.as_ref()));
        if let Some(limit) = lo_limit {
            row[9] = limit.value.to_string();
        }
        if let Some(limit) = hi_limit {
            row[10] = limit.value.to_string();
        }
        row[12] = resolve_limit_units(&invocation.name, lo_limit, hi_limit)?;
        if let Some(comment) = invocation.get("comment")? {
            row[31] = comment.to_string();
        }
        let subtest_ids = test.map(|test| test.sub_tests.clone()).unwrap_or_default();
        row[5] = if opcode == "Test" && !subtest_ids.is_empty() {
            "Test-defer-limits".to_string()
        } else {
            opcode.to_string()
        };
        row[6] = test_parameter;
        row[18] = if opcode == "characterize" || flag_pass || flag_fail {
            "None".to_string()
        } else {
            invocation
                .get("result")?
                .map(ToString::to_string)
                .unwrap_or_else(|| "Fail".to_string())
        };
        if flag_pass {
            row[19] = format!("{}_PASSED", flow_id.to_str());
        }
        if flag_fail {
            row[20] = format!("{}_FAILED", flow_id.to_str());
        }
        let base_row = row.clone();
        let test_name = test
            .map(|test| {
                self.instance_names
                    .get(&test.id)
                    .cloned()
                    .unwrap_or_else(|| test.name.clone())
            })
            .unwrap_or_default();
        let invocation_name = invocation.name.clone();
        self.emit(row);
        for subtest_id in subtest_ids {
            let subtest = self.model.sub_tests[subtest_id].clone();
            let mut limit_row = vec![String::new(); 32];
            limit_row[5] = "Use-Limit".to_string();
            limit_row[6] = test_name.clone();
            limit_row[7] = subtest
                .name
                .clone()
                .unwrap_or_else(|| invocation_name.clone());
            limit_row[8] = subtest.number.map(|n| n.to_string()).unwrap_or_default();
            if let Some(limit) = &subtest.lo_limit {
                limit_row[9] = limit.value.to_string();
            }
            if let Some(limit) = &subtest.hi_limit {
                limit_row[10] = limit.value.to_string();
            }
            limit_row[12] = resolve_limit_units(
                subtest.name.as_deref().unwrap_or(&invocation_name),
                subtest.lo_limit.as_ref(),
                subtest.hi_limit.as_ref(),
            )?;
            for column in 11..=20 {
                if limit_row[column].is_empty() {
                    limit_row[column] = base_row[column].clone();
                }
            }
            self.emit(limit_row);
        }
        Ok(())
    }

    fn string_test_row(
        &mut self,
        name: &str,
        bin: Option<usize>,
        softbin: Option<usize>,
        number: Option<usize>,
        flow_id: &crate::prog_gen::FlowID,
        flag_pass: bool,
        flag_fail: bool,
    ) {
        let mut row = vec![String::new(); 32];
        row[5] = "Test".to_string();
        row[6] = name.to_string();
        row[7] = name.to_string();
        row[8] = number.map(|n| n.to_string()).unwrap_or_default();
        row[15] = bin.map(|n| n.to_string()).unwrap_or_default();
        row[17] = softbin.map(|n| n.to_string()).unwrap_or_default();
        row[18] = if flag_pass || flag_fail {
            "None"
        } else {
            "Fail"
        }
        .to_string();
        if flag_pass {
            row[19] = format!("{}_PASSED", flow_id.to_str());
        }
        if flag_fail {
            row[20] = format!("{}_FAILED", flow_id.to_str());
        }
        self.emit(row);
    }

    fn result_flags(node: &Node<PGM>) -> (bool, bool) {
        let pass = node
            .children
            .iter()
            .any(|n| matches!(n.attrs, PGM::OnPassed(_)));
        let fail = node
            .children
            .iter()
            .any(|n| matches!(n.attrs, PGM::OnFailed(_)));
        (pass, fail)
    }

    fn process_test_children(
        &mut self,
        node: &Node<PGM>,
        flow_id: &crate::prog_gen::FlowID,
    ) -> Result<()> {
        for child in &node.children {
            match &child.attrs {
                PGM::OnPassed(_) => {
                    self.gates.push(Gate {
                        device_condition: Some("flag-true".to_string()),
                        device_name: Some(format!("{}_PASSED", flow_id.to_str())),
                        ..Default::default()
                    });
                    child.process_children(self)?;
                    self.gates.pop();
                }
                PGM::OnFailed(_) => {
                    self.gates.push(Gate {
                        device_condition: Some("flag-true".to_string()),
                        device_name: Some(format!("{}_FAILED", flow_id.to_str())),
                        ..Default::default()
                    });
                    child.process_children(self)?;
                    self.gates.pop();
                }
                _ => {
                    child.process(self)?;
                }
            }
        }
        Ok(())
    }

    fn update_group_results(&mut self, flow_id: &crate::prog_gen::FlowID) -> Result<()> {
        if self.group_results.is_empty() {
            return Ok(());
        }
        self.gates.push(Gate {
            device_condition: Some("flag-true".to_string()),
            device_name: Some(format!("{}_FAILED", flow_id.to_str())),
            ..Default::default()
        });
        for (failed, passed) in self.group_results.clone() {
            self.simple_row("flag-true", &failed);
            self.simple_row("flag-false", &passed);
        }
        self.gates.pop();
        Ok(())
    }

    fn process_group(
        &mut self,
        node: &Node<PGM>,
        flow_id: Option<&crate::prog_gen::FlowID>,
    ) -> Result<()> {
        if let Some(flow_id) = flow_id {
            let failed = format!("{}_FAILED", flow_id.to_str());
            let passed = format!("{}_PASSED", flow_id.to_str());
            self.simple_row("flag-false", &failed);
            self.simple_row("flag-true", &passed);
            self.group_results.push((failed.clone(), passed.clone()));
            for child in &node.children {
                if !matches!(child.attrs, PGM::OnFailed(_) | PGM::OnPassed(_)) {
                    child.process(self)?;
                }
            }
            self.group_results.pop();
            for child in &node.children {
                match child.attrs {
                    PGM::OnFailed(_) => {
                        self.gates.push(Gate {
                            device_condition: Some("flag-true".to_string()),
                            device_name: Some(failed.clone()),
                            ..Default::default()
                        });
                        child.process_children(self)?;
                        self.gates.pop();
                    }
                    PGM::OnPassed(_) => {
                        self.gates.push(Gate {
                            device_condition: Some("flag-true".to_string()),
                            device_name: Some(passed.clone()),
                            ..Default::default()
                        });
                        child.process_children(self)?;
                        self.gates.pop();
                    }
                    _ => {}
                }
            }
        } else {
            node.process_children(self)?;
        }
        Ok(())
    }

    fn condition_gate(condition: &FlowCondition) -> Option<Gate> {
        match condition {
            FlowCondition::IfJob(values) => Some(Gate {
                job: Some(
                    values
                        .iter()
                        .map(|v| v.to_uppercase())
                        .collect::<Vec<_>>()
                        .join(","),
                ),
                ..Default::default()
            }),
            FlowCondition::UnlessJob(values) => Some(Gate {
                job: Some(
                    values
                        .iter()
                        .map(|v| format!("!{}", v.to_uppercase()))
                        .collect::<Vec<_>>()
                        .join(","),
                ),
                ..Default::default()
            }),
            FlowCondition::IfEnable(values) => Some(Gate {
                enable: Some(values.join(",")),
                ..Default::default()
            }),
            FlowCondition::IfFlag(values) => {
                if values.len() > 1 {
                    Some(Gate {
                        group_specifier: Some("any-active".to_string()),
                        group_condition: Some("flag-true".to_string()),
                        group_name: Some(values.join(",")),
                        ..Default::default()
                    })
                } else {
                    Some(Gate {
                        device_condition: Some("flag-true".to_string()),
                        device_name: Some(values.join(",")),
                        ..Default::default()
                    })
                }
            }
            FlowCondition::IfAnySitesFlag(values) => Some(Gate {
                group_specifier: Some("any-active".to_string()),
                group_condition: Some("flag-true".to_string()),
                group_name: Some(values.join(",")),
                ..Default::default()
            }),
            FlowCondition::UnlessFlag(values) => {
                if values.len() > 1 {
                    Some(Gate {
                        group_specifier: Some("any-active".to_string()),
                        group_sense: Some("not".to_string()),
                        group_condition: Some("flag-true".to_string()),
                        group_name: Some(values.join(",")),
                        ..Default::default()
                    })
                } else {
                    Some(Gate {
                        device_sense: Some("not".to_string()),
                        device_condition: Some("flag-true".to_string()),
                        device_name: Some(values.join(",")),
                        ..Default::default()
                    })
                }
            }
            FlowCondition::IfAllSitesFlag(values) => Some(Gate {
                group_specifier: Some("all-active".to_string()),
                group_condition: Some("flag-true".to_string()),
                group_name: Some(values.join(",")),
                ..Default::default()
            }),
            _ => None,
        }
    }
}
impl Processor<PGM> for FlowGenerator {
    fn on_node(&mut self, node: &Node<PGM>) -> Result<Return<PGM>> {
        Ok(match &node.attrs {
            PGM::Flow(name) => {
                self.model.select_flow(name)?;
                Return::ProcessChildren
            }
            PGM::ResourcesFilename(name, kind) => {
                self.model.set_resources_filename(name.clone(), kind);
                if matches!(kind, ResourcesType::All) {
                    self.resource_filename = name.clone();
                }
                Return::None
            }
            PGM::IGXLResourcesFilename(kind, name) => {
                self.resource_filenames.insert(*kind, name.clone());
                Return::None
            }
            PGM::Resources => {
                let original = self.resources;
                self.resources = true;
                node.process_children(self)?;
                self.resources = original;
                Return::None
            }
            PGM::Test(id, flow_id) => {
                let (pass, mut fail) = Self::result_flags(node);
                fail |= !self.group_results.is_empty();
                self.test_row(*id, "Test", None, flow_id, pass, fail)?;
                self.process_test_children(node, flow_id)?;
                self.update_group_results(flow_id)?;
                Return::None
            }
            PGM::Cz(id, setup, flow_id) => {
                let (pass, mut fail) = Self::result_flags(node);
                fail |= !self.group_results.is_empty();
                self.test_row(*id, "characterize", Some(setup), flow_id, pass, fail)?;
                self.process_test_children(node, flow_id)?;
                self.update_group_results(flow_id)?;
                Return::None
            }
            PGM::TestStr(name, flow_id, bin, softbin, number) => {
                let (pass, mut fail) = Self::result_flags(node);
                fail |= !self.group_results.is_empty();
                self.string_test_row(name, *bin, *softbin, *number, flow_id, pass, fail);
                self.process_test_children(node, flow_id)?;
                self.update_group_results(flow_id)?;
                Return::None
            }
            PGM::Log(text) => {
                self.simple_row("logprint", text);
                Return::None
            }
            PGM::Render(text) => {
                if !self.resources {
                    self.rows.extend(text.lines().map(str::to_string));
                }
                Return::None
            }
            PGM::Condition(condition) => {
                // IG-XL cannot express "unless any of these enable words" as a
                // single gate. Emit one enabled-word goto per word, then place a
                // label after the guarded body so any enabled word skips it.
                if let FlowCondition::UnlessEnable(words) = condition {
                    self.label_counter += 1;
                    let label = format!("ORIGEN_SKIP_{}", self.label_counter);
                    for word in words {
                        self.gates.push(Gate {
                            enable: Some(word.clone()),
                            ..Default::default()
                        });
                        self.simple_row("goto", &label);
                        self.gates.pop();
                    }
                    for child in &node.children {
                        if !matches!(child.attrs, PGM::Else) {
                            child.process(self)?;
                        }
                    }
                    let mut row = vec![String::new(); 32];
                    row[0] = label;
                    row[5] = "nop".to_string();
                    self.emit(row);
                    Return::None
                } else if let Some(gate) = Self::condition_gate(condition) {
                    self.gates.push(gate);
                    for child in &node.children {
                        if !matches!(child.attrs, PGM::Else) {
                            child.process(self)?;
                        }
                    }
                    self.gates.pop();
                    Return::None
                } else {
                    Return::ProcessChildren
                }
            }
            PGM::Else => Return::None,
            PGM::SetFlag(flag, state, _) => {
                self.simple_row(if *state { "flag-true" } else { "flag-false" }, flag);
                Return::None
            }
            PGM::Enable(word) => {
                self.simple_row("enable-flow-word", word);
                Return::None
            }
            PGM::Disable(word) => {
                self.simple_row("disable-flow-word", word);
                Return::None
            }
            PGM::Label(label) => {
                let mut row = vec![String::new(); 32];
                row[0] = label.clone();
                row[5] = "nop".to_string();
                self.emit(row);
                Return::None
            }
            PGM::Goto(label) => {
                self.simple_row("goto", label);
                Return::None
            }
            PGM::Comment(comment) => {
                let mut row = vec![String::new(); 32];
                row[5] = "nop".to_string();
                row[31] = sanitize(comment);
                self.emit(row);
                Return::None
            }
            PGM::Bin(hard, soft, kind) => {
                let mut row = vec![String::new(); 32];
                row[5] = "set-device".to_string();
                row[15] = hard.to_string();
                row[17] = soft.map(|n| n.to_string()).unwrap_or_default();
                row[18] = if matches!(kind, BinType::Good) {
                    "Pass"
                } else {
                    "Fail"
                }
                .to_string();
                self.emit(row);
                Return::None
            }
            PGM::PatternGroup(id, name, tester, kind) => {
                if tester.is_compatible_with(&SupportedTester::ULTRAFLEX) {
                    self.patsets.insert(
                        *id,
                        Patset {
                            name: name.clone(),
                            kind: kind.clone().unwrap_or(PatternGroupType::Patset),
                            patterns: vec![],
                        },
                    );
                }
                Return::None
            }
            PGM::PushPattern(id, path, start_label) => {
                if let Some(patset) = self.patsets.get_mut(id) {
                    patset.patterns.push(PatsetPattern {
                        path: path.clone(),
                        start_label: start_label.clone(),
                    });
                }
                Return::None
            }
            PGM::IGXLSetWaitFlags(id, flags) => {
                self.wait_flags.insert(*id, flags.clone());
                Return::None
            }
            PGM::IGXLResource(resource) => {
                let sheet = self
                    .resource_filenames
                    .get(&resource.kind)
                    .cloned()
                    .unwrap_or_else(|| self.resource_filename.clone());
                self.resources_rows.push(ResourceRow {
                    sheet,
                    kind: resource.kind,
                    name: resource.name.clone(),
                    values: resource.values.clone(),
                });
                Return::None
            }
            PGM::Group(_, _, kind, flow_id) => {
                if matches!(kind, crate::prog_gen::GroupType::Test) {
                    // Test groups only scope IG-XL Test Instance membership. Their
                    // names and members were already captured by initial_model_extract;
                    // they do not represent executable Flow Table groups.
                    Return::ProcessChildren
                } else {
                    self.process_group(node, flow_id.as_ref())?;
                    Return::None
                }
            }
            PGM::SubFlow(_, _) => Return::ProcessChildren,
            PGM::OnFailed(_) | PGM::OnPassed(_) => Return::ProcessChildren,
            PGM::Continue | PGM::Delayed => Return::None,
            PGM::Unknown(kind, _) | PGM::Variable(_, kind, _) | PGM::Parameter(_, kind, _) => {
                bail!(
                    "UltraFLEX IG-XL flow generation does not support '{}' nodes",
                    kind
                )
            }
            PGM::Wait(_)
            | PGM::SetVariable(_, _)
            | PGM::Synchronize
            | PGM::OnError(_)
            | PGM::Call(_, _)
            | PGM::Loop(_, _)
            | PGM::Report(_, _)
            | PGM::Assertion(_, _)
            | PGM::Callback(_, _) => {
                bail!("The requested flow operation has no UltraFLEX IG-XL equivalent")
            }
            _ => Return::ProcessChildren,
        })
    }
}

fn merge_field(field: &mut String, value: Option<&str>) {
    if let Some(value) = value {
        if !value.is_empty() {
            if !field.is_empty() {
                field.push(',');
            }
            field.push_str(value);
        }
    }
}

fn sanitize(value: &str) -> String {
    value.replace(['\t', '\r', '\n'], "_")
}

pub(super) fn resolve_limit_units(
    test_name: &str,
    lo_limit: Option<&Limit>,
    hi_limit: Option<&Limit>,
) -> Result<String> {
    let lo_unit = lo_limit.map(Limit::unit_str).unwrap_or("");
    let hi_unit = hi_limit.map(Limit::unit_str).unwrap_or("");
    if !lo_unit.is_empty() && !hi_unit.is_empty() && lo_unit != hi_unit {
        bail!(
            "UltraFLEX test '{}' has incompatible limit units: low limit uses '{}' and high limit uses '{}'",
            test_name,
            lo_unit,
            hi_unit,
        )
    }
    Ok(if !hi_unit.is_empty() {
        hi_unit.to_string()
    } else {
        lo_unit.to_string()
    })
}
