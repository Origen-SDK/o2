use crate::ast::{Node, Processor, Return};
use crate::prog_gen::{
    process_flow, BinType, FlowCondition, Model, ParamValue, PatternGroupType, ResourcesType,
    SupportedTester, PGM,
};
use crate::{Result, FLOW};
use indexmap::IndexMap;
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};

const FLOW_HEADER: [&str; 4] = [
    "DTFlowtableSheet,version=2.2:platform=Jaguar:toprow=-1:leftcol=-1:rightcol=-1\tFlow Table",
    "\t\t\t\t\t\tFlow Domain:",
    "\t\t\tGate\t\t\tCommand\t\t\t\tLimits\t\tDatalog Display Results\t\t\tBin Number\t\tSort Number\t\t\tFlag\t\t\tGroup\t\t\t\tDevice\t\t\tDebug",
    "\tLabel\tEnable\tJob\tPart\tEnv\tOpcode\tParameter\tTName\tTNum\tLoLim\tHiLim\tScale\tUnits\tFormat\tPass\tFail\tPass\tFail\tResult\tPass\tFail\tState\tSpecifier\tSense\tCondition\tName\tSense\tCondition\tName\tAssume\tSites\tComment",
];

const INSTANCE_PREFIX: [&str; 4] = [
    "DTTestInstancesSheet,version=2.4:platform=Jaguar:toprow=-1:leftcol=-1:rightcol=-1\tTest Instances",
    "",
    "\t\tTest Procedure\t\t\tDC Specs\t\tAC Specs\t\tSheet Parameters\t\t\t\t\tOther Parameters",
    "",
];

const PATSET_HEADER: [&str; 4] = [
    "DTPatternSetSheet,version=2.1:platform=Jaguar:toprow=-1:leftcol=-1:rightcol=-1\tPattern Sets",
    "",
    "\tPattern Set\tTD Group\tTime Domain\tFile/Group Name\tBurst\tStart Label\tStop Label\tComment",
    "",
];

/// Render the current program as UltraFLEX IG-XL text worksheets.
///
/// Flow, Test Instances and Pattern Sets are emitted as independent importable
/// sheets.  This deliberately uses the tester-neutral PGM AST rather than
/// exposing a second UltraFLEX-specific flow API.
pub fn render(output_dir: &Path) -> Result<(Vec<PathBuf>, Model)> {
    std::fs::create_dir_all(output_dir)?;
    let mut generated = vec![];
    let mut resource_rows = vec![];
    let mut referenced_patterns = vec![];
    let model = FLOW.with_all_flows(|flows| {
        let mut model = Model::new(SupportedTester::ULTRAFLEX);
        for (name, flow) in flows {
            let (ast, next_model) = process_flow(flow, model, SupportedTester::ULTRAFLEX, true)?;
            let (next_model, mut files, mut rows, mut patterns) =
                render_flow(&ast, output_dir, next_model, name)?;
            model = next_model;
            generated.append(&mut files);
            resource_rows.append(&mut rows);
            referenced_patterns.append(&mut patterns);
        }
        Ok(model)
    })?;
    let mut resource_writer = FlowGenerator::new(Model::new(SupportedTester::ULTRAFLEX));
    resource_writer.resources_rows = resource_rows;
    generated.append(&mut resource_writer.write_resource_sheets(output_dir)?);
    if let Some(path) = write_referenced_list(output_dir, referenced_patterns)? {
        generated.push(path);
    }
    Ok((generated, model))
}

fn write_referenced_list(output_dir: &Path, mut patterns: Vec<String>) -> Result<Option<PathBuf>> {
    patterns.sort();
    patterns.dedup();
    if patterns.is_empty() {
        return Ok(None);
    }
    let path = output_dir.join("referenced.list");
    let mut file = std::fs::File::create(&path)?;
    writeln!(file, "# Main patterns")?;
    for pattern in patterns {
        writeln!(file, "{}", pattern)?;
    }
    Ok(Some(path))
}

fn render_flow(
    ast: &Node<PGM>,
    output_dir: &Path,
    model: Model,
    flow_name: &str,
) -> Result<(Model, Vec<PathBuf>, Vec<ResourceRow>, Vec<String>)> {
    let mut generator = FlowGenerator::new(model);
    ast.process(&mut generator)?;

    let flow_path = output_dir.join(format!("{}_flow.txt", flow_name));
    write_sheet(&flow_path, &FLOW_HEADER, &generator.rows)?;

    let instance_path = output_dir.join(format!("{}_instances.txt", flow_name));
    generator.write_instances(&instance_path, flow_name)?;

    let patset_path = output_dir.join(format!("{}_patsets.txt", flow_name));
    generator.write_patsets(&patset_path)?;

    let mut files = vec![flow_path, instance_path, patset_path];
    let patgroups_path = output_dir.join(format!("{}_patgroups.txt", flow_name));
    if generator.write_patgroups(&patgroups_path)? {
        files.push(patgroups_path);
    }
    let patsubrs_path = output_dir.join(format!("{}_patsubrs.txt", flow_name));
    if generator.write_patsubrs(&patsubrs_path)? {
        files.push(patsubrs_path);
    }

    let patterns = generator
        .patsets
        .values()
        .flat_map(|group| group.patterns.iter())
        .map(|pattern| {
            Path::new(&pattern.path)
                .file_stem()
                .and_then(|name| name.to_str())
                .unwrap_or(&pattern.path)
                .to_string()
        })
        .collect();
    Ok((generator.model, files, generator.resources_rows, patterns))
}

fn write_sheet(path: &Path, header: &[&str], rows: &[String]) -> Result<()> {
    let mut file = std::fs::File::create(path)?;
    for line in header {
        writeln!(file, "{}", line)?;
    }
    for row in rows {
        writeln!(file, "{}", row)?;
    }
    Ok(())
}

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

#[derive(Clone)]
struct PatsetPattern {
    path: String,
    start_label: Option<String>,
}

#[derive(Clone)]
struct Patset {
    name: String,
    kind: PatternGroupType,
    patterns: Vec<PatsetPattern>,
}

type ResourceRow = (String, String, String, IndexMap<String, Vec<String>>);

struct FlowGenerator {
    model: Model,
    rows: Vec<String>,
    gates: Vec<Gate>,
    resources: bool,
    patsets: IndexMap<usize, Patset>,
    wait_flags: HashMap<usize, Vec<String>>,
    label_counter: usize,
    resources_rows: Vec<ResourceRow>,
    resource_filename: String,
    resource_filenames: HashMap<String, String>,
    group_results: Vec<(String, String)>,
    instance_names: HashMap<usize, String>,
}

impl FlowGenerator {
    fn new(model: Model) -> Self {
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

    fn write_resource_sheets(&mut self, output_dir: &Path) -> Result<Vec<PathBuf>> {
        let all_resource_rows = self.resources_rows.clone();
        let mut files = vec![];
        let mut resource_sheets = vec![];
        for (sheet, _, _, _) in &all_resource_rows {
            if !resource_sheets.contains(sheet) {
                resource_sheets.push(sheet.clone());
            }
        }
        for (sheet_index, sheet) in resource_sheets.into_iter().enumerate() {
            self.resources_rows = all_resource_rows
                .iter()
                .filter(|(row_sheet, _, _, _)| row_sheet == &sheet)
                .cloned()
                .collect();
            let mut parts = vec![];
            for (part_index, (suffix, writer)) in [
                (
                    "references",
                    FlowGenerator::write_references as fn(&FlowGenerator, &Path) -> Result<bool>,
                ),
                ("jobs", FlowGenerator::write_jobs),
                ("global_specs", FlowGenerator::write_global_specs),
                ("pinmap", FlowGenerator::write_pinmap),
                ("levels", FlowGenerator::write_levels),
                ("edgesets", FlowGenerator::write_edgesets),
                ("timesets", FlowGenerator::write_timesets),
                ("timesets_basic", FlowGenerator::write_timesets_basic),
            ]
            .into_iter()
            .enumerate()
            {
                let path = output_dir.join(format!(
                    ".origen_uflex_{}_{}_{}.part",
                    sheet_index, part_index, suffix
                ));
                if writer(self, &path)? {
                    parts.push(path);
                }
            }
            for (part_index, (kind, suffix, title)) in [
                ("ac_specs", "ac_specs", "AC"),
                ("dc_specs", "dc_specs", "DC"),
            ]
            .into_iter()
            .enumerate()
            {
                let path = output_dir.join(format!(
                    ".origen_uflex_{}_spec_{}_{}.part",
                    sheet_index, part_index, suffix
                ));
                if self.write_specs(&path, kind, title)? {
                    parts.push(path);
                }
            }
            if !parts.is_empty() {
                let mut sheet_path = PathBuf::from(&sheet);
                if sheet_path.extension().is_none() {
                    sheet_path.set_extension("txt");
                }
                if !sheet_path.is_absolute() {
                    sheet_path = output_dir.join(sheet_path);
                }
                if let Some(parent) = sheet_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let mut output = std::fs::File::create(&sheet_path)?;
                for part in parts {
                    let mut input = std::fs::File::open(&part)?;
                    std::io::copy(&mut input, &mut output)?;
                    std::fs::remove_file(part)?;
                }
                files.push(sheet_path);
            }
        }
        self.resources_rows = all_resource_rows;
        Ok(files)
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
        if let Some(limit) = invocation
            .lo_limit
            .as_ref()
            .or_else(|| test.and_then(|t| t.lo_limit.as_ref()))
        {
            row[9] = limit.value.to_string();
            if !limit.unit_str().is_empty() {
                row[12] = limit.unit_str().to_string();
            }
        }
        if let Some(limit) = invocation
            .hi_limit
            .as_ref()
            .or_else(|| test.and_then(|t| t.hi_limit.as_ref()))
        {
            row[10] = limit.value.to_string();
            if !limit.unit_str().is_empty() {
                row[12] = limit.unit_str().to_string();
            }
        }
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
                limit_row[12] = limit.unit_str().to_string();
            }
            if let Some(limit) = &subtest.hi_limit {
                limit_row[10] = limit.value.to_string();
                if !limit.unit_str().is_empty() {
                    limit_row[12] = limit.unit_str().to_string();
                }
            }
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
            FlowCondition::UnlessEnable(values) => Some(Gate {
                enable: Some(
                    values
                        .iter()
                        .map(|v| format!("!{}", v))
                        .collect::<Vec<_>>()
                        .join(","),
                ),
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
            FlowCondition::UnlessFlag(values) => Some(Gate {
                device_sense: Some("not".to_string()),
                device_condition: Some("flag-true".to_string()),
                device_name: Some(values.join(",")),
                ..Default::default()
            }),
            FlowCondition::IfAllSitesFlag(values) => Some(Gate {
                group_specifier: Some("all-active".to_string()),
                group_condition: Some("flag-true".to_string()),
                group_name: Some(values.join(",")),
                ..Default::default()
            }),
            _ => None,
        }
    }

    fn write_instances(&self, path: &Path, flow_name: &str) -> Result<()> {
        let mut file = std::fs::File::create(path)?;
        for (index, line) in INSTANCE_PREFIX.iter().enumerate() {
            if index == 3 {
                let mut columns = vec![
                    "Test Name",
                    "Type",
                    "Name",
                    "Called As",
                    "Category",
                    "Selector",
                    "Category",
                    "Selector",
                    "Time Sets",
                    "Edge Sets",
                    "Pin Levels",
                    "Mixed Signal Timing",
                    "Overlay",
                ]
                .into_iter()
                .map(str::to_string)
                .collect::<Vec<_>>();
                columns.extend((0..=129).map(|i| format!("Arg{}", i)));
                columns.push("Comment".to_string());
                writeln!(file, "\t{}", columns.join("\t"))?;
            } else {
                writeln!(file, "{}", line)?;
            }
        }
        let ids = self
            .model
            .get_flow(Some(flow_name))
            .map(|flow| flow.tests.clone())
            .unwrap_or_default();
        let mut ids = ids;
        ids.sort_by(|left, right| {
            self.instance_names
                .get(left)
                .cmp(&self.instance_names.get(right))
        });
        let mut rendered_names = std::collections::HashSet::new();
        for id in ids {
            if let Some(test) = self.model.tests.get(&id) {
                let rendered_name = self
                    .instance_names
                    .get(&id)
                    .cloned()
                    .unwrap_or_else(|| test.name.clone());
                if !rendered_names.insert(rendered_name.clone()) {
                    continue;
                }
                let mut fields = vec![String::new(); 144];
                fields[0] = rendered_name;
                let names = [
                    "proc_type",
                    "proc_name",
                    "proc_called_as",
                    "dc_category",
                    "dc_selector",
                    "ac_category",
                    "ac_selector",
                    "time_sets",
                    "edge_sets",
                    "pin_levels",
                    "mixedsignal_timing",
                    "overlay",
                ];
                for (offset, name) in names.iter().enumerate() {
                    fields[offset + 1] = param(test.get(name)?);
                }
                for arg in 0..=129 {
                    fields[13 + arg] = param(test.get(&format!("arg{}", arg))?);
                }
                if let Some(flags) = self.wait_flags.get(&id) {
                    for flag in flags {
                        let offset = match flag.to_ascii_lowercase().as_str() {
                            "a" => Some(0),
                            "b" => Some(1),
                            "c" => Some(2),
                            "d" => Some(3),
                            _ => None,
                        };
                        if let Some(offset) = offset {
                            fields[13 + 28 + offset] = "-1".to_string();
                        }
                    }
                }
                writeln!(file, "\t{}", fields.join("\t"))?;
            }
        }
        Ok(())
    }

    fn write_patsets(&self, path: &Path) -> Result<()> {
        let mut rows = vec![];
        for patset in self.patsets.values() {
            if patset.kind != PatternGroupType::Patset {
                continue;
            }
            for pattern in &patset.patterns {
                let mut fields = vec![String::new(); 8];
                fields[0] = patset.name.clone();
                fields[3] = pattern.path.clone();
                fields[4] = "Yes".to_string();
                fields[5] = pattern.start_label.clone().unwrap_or_default();
                rows.push(format!("\t{}", fields.join("\t")));
            }
        }
        write_sheet(path, &PATSET_HEADER, &rows)
    }

    fn write_patgroups(&self, path: &Path) -> Result<bool> {
        let groups = self
            .patsets
            .values()
            .filter(|g| g.kind == PatternGroupType::Patgroup)
            .collect::<Vec<_>>();
        if groups.is_empty() {
            return Ok(false);
        }
        let mut file = std::fs::File::create(path)?;
        writeln!(file, "DFF 1.0\tPattern Groups")?;
        writeln!(file, "ULTRAFLEX DOES NOT SUPPORT PATTERN GROUP SHEETS!!")?;
        writeln!(file)?;
        writeln!(file, "\tGroup Name\tPattern File\tComment")?;
        for group in groups {
            for pattern in &group.patterns {
                writeln!(file, "\t{}\t{}\t", group.name, pattern.path)?;
            }
        }
        Ok(true)
    }

    fn write_patsubrs(&self, path: &Path) -> Result<bool> {
        let groups = self
            .patsets
            .values()
            .filter(|g| g.kind == PatternGroupType::Patsubr)
            .collect::<Vec<_>>();
        if groups.is_empty() {
            return Ok(false);
        }
        let mut file = std::fs::File::create(path)?;
        writeln!(file, "DTPatternSubroutineSheet,version=2.0:platform=Jaguar:toprow=-1:leftcol=-1:rightcol=-1\tPattern Subroutine")?;
        writeln!(file)?;
        writeln!(file, "\tPattern Filename\tComment")?;
        for group in groups {
            for pattern in &group.patterns {
                writeln!(file, "\t{}\t", pattern.path)?;
            }
        }
        Ok(true)
    }

    fn write_references(&self, path: &Path) -> Result<bool> {
        let rows = self
            .resources_rows
            .iter()
            .filter(|(_, kind, _, _)| kind == "references")
            .collect::<Vec<_>>();
        if rows.is_empty() {
            return Ok(false);
        }
        let mut file = std::fs::File::create(path)?;
        writeln!(file, "DTReferencesSheet,version=2.0:platform=Jaguar:toprow=-1:leftcol=-1:rightcol=-1\tReferences")?;
        writeln!(file)?;
        writeln!(file, "\tFile Path\tComment\t")?;
        for (_, _, name, values) in rows {
            writeln!(file, "\t{}\t{}", name, resource_value(values, "comment"))?;
        }
        Ok(true)
    }

    fn write_jobs(&self, path: &Path) -> Result<bool> {
        let rows = self
            .resources_rows
            .iter()
            .filter(|(_, kind, _, _)| kind == "jobs")
            .collect::<Vec<_>>();
        if rows.is_empty() {
            return Ok(false);
        }
        let mut file = std::fs::File::create(path)?;
        writeln!(
            file,
            "DTJobListSheet,version=2.5:platform=Jaguar:toprow=-1:leftcol=-1:rightcol=-1\tJob List"
        )?;
        writeln!(file)?;
        writeln!(file, "\t\tSheet Parameters\t")?;
        writeln!(file, "\tJob Name\tPin Map\tTest Instances\tFlow Table\tAC Specs\tDC Specs\tPattern Sets\tPattern Groups\tBin Table\tCharacterization\tTest Procedures\tMixed Signal Timing\tWave Definitions\tPsets\tSignals\tPort Map\tFractional Bus\tConcurrent Sequence\tComment")?;
        let columns = [
            "pinmap",
            "instances",
            "flows",
            "ac_specs",
            "dc_specs",
            "patsets",
            "patgroups",
            "bintables",
            "cz",
            "test_procs",
            "mix_sig_timing",
            "wave_defs",
            "signals",
            "port_map",
            "fract_bus",
            "concurrent_seq",
            "comment",
        ];
        for (_, _, name, values) in rows {
            let mut fields = vec![name.clone()];
            fields.extend(columns.iter().map(|column| resource_value(values, column)));
            writeln!(file, "\t{}", fields.join("\t"))?;
        }
        Ok(true)
    }

    fn write_global_specs(&self, path: &Path) -> Result<bool> {
        let rows = self
            .resources_rows
            .iter()
            .filter(|(_, kind, _, _)| kind == "global_specs")
            .collect::<Vec<_>>();
        if rows.is_empty() {
            return Ok(false);
        }
        let mut file = std::fs::File::create(path)?;
        writeln!(file, "DTGlobalSpecSheet,version=2.0:platform=Jaguar:toprow=-1:leftcol=-1:rightcol=-1\tGlobal Specs")?;
        writeln!(file)?;
        writeln!(file, "\tSymbol\tJob\tValue\tComment")?;
        writeln!(file, "\tVcl_default\t\t-1\tDetector clamp voltage low")?;
        writeln!(file, "\tVch_default\t\t6\tDetector clamp voltage high")?;
        writeln!(file, "\tVph_default\t\t5\t")?;
        let mut rows = rows;
        rows.sort_by(|(_, _, left, _), (_, _, right, _)| left.cmp(right));
        for (_, _, name, values) in rows {
            writeln!(
                file,
                "\t{}\t{}\t{}\t{}",
                name,
                resource_value(values, "job"),
                uflex_expression(&resource_value(values, "value"), false),
                resource_value(values, "comment")
            )?;
        }
        Ok(true)
    }

    fn write_specs(&self, path: &Path, kind: &str, title: &str) -> Result<bool> {
        let rows = self
            .resources_rows
            .iter()
            .filter(|(_, row_kind, _, _)| row_kind == kind)
            .collect::<Vec<_>>();
        if rows.is_empty() {
            return Ok(false);
        }
        let mut specsets = vec![];
        for (_, _, _, values) in &rows {
            let name = resource_value(values, "specset");
            if !specsets.contains(&name) {
                specsets.push(name);
            }
        }
        let mut file = std::fs::File::create(path)?;
        writeln!(
            file,
            "DT{}SpecSheet,version=2.0:platform=Jaguar:toprow=-1:leftcol=-1:rightcol=-1\t{} Specs",
            title, title
        )?;
        writeln!(file)?;
        let names = specsets
            .iter()
            .map(|name| format!("{}\t\t\t", name))
            .collect::<String>();
        writeln!(file, "\t\t\tSelector\t\t{}", names)?;
        writeln!(
            file,
            "\tSymbol\tValue\tName\tVal\t{}Comment",
            "Typ\tMin\tMax\t".repeat(specsets.len())
        )?;

        let mut keys = vec![];
        for (_, _, symbol, values) in &rows {
            let key = (symbol.clone(), resource_value(values, "selector"));
            if !keys.contains(&key) {
                keys.push(key);
            }
        }
        keys.sort();
        for (symbol, selector) in keys {
            let mut category = "Typ".to_string();
            let mut fields = vec![symbol.clone(), String::new(), selector.clone()];
            for specset in &specsets {
                let matching = rows.iter().find(|(_, _, row_symbol, values)| {
                    row_symbol == &symbol
                        && resource_value(values, "selector") == selector
                        && resource_value(values, "specset") == *specset
                });
                if let Some((_, _, _, values)) = matching {
                    if !resource_value(values, "max").is_empty() {
                        category = "Max".to_string();
                    } else if !resource_value(values, "min").is_empty() {
                        category = "Min".to_string();
                    }
                    fields.push(uflex_spec_expression(&resource_value(values, "typ")));
                    fields.push(uflex_spec_expression(&resource_value(values, "min")));
                    fields.push(uflex_spec_expression(&resource_value(values, "max")));
                } else {
                    fields.extend(["0".to_string(), "0".to_string(), "0".to_string()]);
                }
            }
            fields.insert(3, category);
            let comment = rows
                .iter()
                .find(|(_, _, row_symbol, values)| {
                    row_symbol == &symbol && resource_value(values, "selector") == selector
                })
                .map(|(_, _, _, values)| resource_value(values, "comment"))
                .unwrap_or_default();
            fields.push(comment);
            writeln!(file, "\t{}", fields.join("\t"))?;
        }
        Ok(true)
    }

    fn write_pinmap(&self, path: &Path) -> Result<bool> {
        let rows = self
            .resources_rows
            .iter()
            .filter(|(_, kind, _, _)| kind == "pinmap")
            .collect::<Vec<_>>();
        if rows.is_empty() {
            return Ok(false);
        }
        let mut file = std::fs::File::create(path)?;
        writeln!(
            file,
            "DTPinMap,version=2.1:platform=Jaguar:toprow=-1:leftcol=-1:rightcol=-1\tPin Map"
        )?;
        writeln!(file, "\t\t\tUSL Tag:\t")?;
        writeln!(file, "\tGroup Name\tPin Name\tType\tComment")?;
        for wanted_kind in ["power", "utility", "pin", "group"] {
            let mut previous_group = String::new();
            for (_, _, name, values) in rows
                .iter()
                .filter(|(_, _, _, values)| resource_value(values, "kind") == wanted_kind)
            {
                let group = resource_value(values, "group");
                let mut pin_type = resource_value(values, "type");
                if wanted_kind == "group" && group == previous_group {
                    pin_type.clear();
                }
                writeln!(
                    file,
                    "\t{}\t{}\t{}\t{}",
                    group,
                    name,
                    pin_type,
                    resource_value(values, "comment")
                )?;
                previous_group = group;
            }
        }
        Ok(true)
    }

    fn write_levels(&self, path: &Path) -> Result<bool> {
        let rows = self
            .resources_rows
            .iter()
            .filter(|(_, kind, _, _)| kind == "levels")
            .collect::<Vec<_>>();
        if rows.is_empty() {
            return Ok(false);
        }
        let mut file = std::fs::File::create(path)?;
        writeln!(
            file,
            "DTLevelSheet,version=2.1:platform=Jaguar:toprow=-1:leftcol=-1:rightcol=-1\tPin Levels"
        )?;
        writeln!(file)?;
        writeln!(file, "\tPin/Group\tSeq.\tParameter\tValue\tComment")?;
        for (_, _, pin, values) in rows {
            writeln!(
                file,
                "\t{}\t\t{}\t{}\t{}",
                pin,
                resource_value(values, "parameter"),
                uflex_expression(&resource_value(values, "value"), false),
                resource_value(values, "comment")
            )?;
        }
        Ok(true)
    }

    fn write_edgesets(&self, path: &Path) -> Result<bool> {
        let rows = self
            .resources_rows
            .iter()
            .filter(|(_, kind, _, _)| kind == "edgesets")
            .collect::<Vec<_>>();
        if rows.is_empty() {
            return Ok(false);
        }
        let mut file = std::fs::File::create(path)?;
        let timing_mode = resource_value(&rows[0].3, "timing_mode");
        writeln!(file, "DTEdgesetSheet,version=2.3:platform=Jaguar:toprow=-1:leftcol=-1:rightcol=-1\tEdge Sets")?;
        writeln!(file)?;
        writeln!(file, "\tTiming Mode:\t{}", timing_mode)?;
        writeln!(file, "\tTime Domain:\t\t\t\tStrobe Ref Setup Name:")?;
        writeln!(file)?;
        writeln!(
            file,
            "\t\t\tData\t\tDrive\t\t\t\tCompare\t\t\t\tEdge Resolution"
        )?;
        writeln!(file, "\tPin/Group\tEdge Set\tSrc\tFmt\tOn\tData\tReturn\tOff\tMode\tOpen\tClose\tRef Offset\tMode\tComment")?;
        let columns = [
            "edgeset",
            "src",
            "format",
            "drive_on",
            "drive_data",
            "drive_return",
            "drive_off",
            "compare_mode",
            "compare_open",
            "compare_close",
        ];
        for (_, _, pin, values) in rows {
            let mut fields = vec![pin.clone()];
            for name in columns {
                let value = resource_value(values, name);
                fields.push(
                    if matches!(
                        name,
                        "drive_on"
                            | "drive_data"
                            | "drive_return"
                            | "drive_off"
                            | "compare_open"
                            | "compare_close"
                    ) {
                        uflex_expression(&value, true)
                    } else {
                        value
                    },
                );
            }
            fields.push(String::new());
            fields.push(resource_value(values, "resolution"));
            fields.push(resource_value(values, "comment"));
            writeln!(file, "\t{}", fields.join("\t"))?;
        }
        Ok(true)
    }

    fn write_timesets(&self, path: &Path) -> Result<bool> {
        let rows = self
            .resources_rows
            .iter()
            .filter(|(_, kind, _, _)| kind == "timesets")
            .collect::<Vec<_>>();
        if rows.is_empty() {
            return Ok(false);
        }
        let mut file = std::fs::File::create(path)?;
        let timing_mode = resource_value(&rows[0].3, "timing_mode");
        writeln!(file, "DTTimesetSheet,version=2.1:platform=Jaguar:toprow=-1:leftcol=-1:rightcol=-1\tTime Sets")?;
        writeln!(file)?;
        writeln!(
            file,
            "\tTiming Mode:\t{}\t\tMaster Timeset Name:\t",
            timing_mode
        )?;
        writeln!(file, "\tTime Domain:\t")?;
        writeln!(file)?;
        writeln!(file, "\t\t\tCycle\tPin/Group")?;
        writeln!(
            file,
            "\tTime Set\tPeriod\tName\tClock Period\tSetup\tEdge Set\tComment"
        )?;
        for (_, _, name, values) in rows {
            writeln!(
                file,
                "\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                name,
                uflex_expression(&resource_value(values, "period"), false),
                resource_value(values, "pin"),
                uflex_expression(&resource_value(values, "clock_period"), false),
                resource_value(values, "setup"),
                resource_value(values, "edgeset"),
                resource_value(values, "comment")
            )?;
        }
        Ok(true)
    }

    fn write_timesets_basic(&self, path: &Path) -> Result<bool> {
        let rows = self
            .resources_rows
            .iter()
            .filter(|(_, kind, _, _)| kind == "timesets_basic")
            .collect::<Vec<_>>();
        if rows.is_empty() {
            return Ok(false);
        }
        let mut file = std::fs::File::create(path)?;
        let timing_mode = resource_value(&rows[0].3, "timing_mode");
        writeln!(file, "DTTimesetBasicSheet,version=2.3:platform=Jaguar:toprow=-1:leftcol=-1:rightcol=-1\tTime Sets (Basic)")?;
        writeln!(file)?;
        writeln!(
            file,
            "\tTiming Mode:\t{}\t\tMaster Timeset Name:\t",
            timing_mode
        )?;
        writeln!(file, "\tTime Domain:\t\t\tStrobe Ref Setup Name:")?;
        writeln!(file)?;
        writeln!(
            file,
            "\t\tCycle\tPin/Group\t\t\tData\t\tDrive\t\t\t\tCompare\t\t\t\tEdge Resolution"
        )?;
        writeln!(file, "\tTime Set\tPeriod\tName\tClock Period\tSetup\tSrc\tFmt\tOn\tData\tReturn\tOff\tMode\tOpen\tClose\tRef Offset\tMode\tComment")?;
        let columns = [
            "period",
            "pin",
            "clock_period",
            "setup",
            "src",
            "format",
            "drive_on",
            "drive_data",
            "drive_return",
            "drive_off",
            "compare_mode",
            "compare_open",
            "compare_close",
        ];
        for (_, _, name, values) in rows {
            let mut fields = vec![name.clone()];
            for column in columns {
                let value = resource_value(values, column);
                fields.push(match column {
                    "period" | "clock_period" => uflex_expression(&value, false),
                    "drive_on" | "drive_data" | "drive_return" | "drive_off" | "compare_open"
                    | "compare_close" => uflex_expression(&value, true),
                    _ => value,
                });
            }
            fields.push(String::new());
            fields.push(resource_value(values, "resolution"));
            fields.push(resource_value(values, "comment"));
            writeln!(file, "\t{}", fields.join("\t"))?;
        }
        Ok(true)
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
                self.resource_filenames
                    .insert(kind.as_str().to_string(), name.clone());
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
                    .get(resource.kind.as_str())
                    .cloned()
                    .unwrap_or_else(|| self.resource_filename.clone());
                self.resources_rows.push((
                    sheet,
                    resource.kind.as_str().to_string(),
                    resource.name.clone(),
                    resource.values.clone(),
                ));
                Return::None
            }
            PGM::Group(_, _, kind, flow_id) => {
                if matches!(kind, crate::prog_gen::GroupType::Test) {
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

fn param(value: Option<&ParamValue>) -> String {
    value.map(ToString::to_string).unwrap_or_default()
}

fn resource_value(values: &IndexMap<String, Vec<String>>, name: &str) -> String {
    values.get(name).map(|v| v.join(",")).unwrap_or_default()
}

fn uflex_expression(value: &str, disable_when_empty: bool) -> String {
    let value = value.trim();
    if value.is_empty() {
        if disable_when_empty {
            "disable".to_string()
        } else {
            String::new()
        }
    } else if value.starts_with('=') || value == "0" || value.eq_ignore_ascii_case("disable") {
        value.to_string()
    } else {
        format!("={}", value)
    }
}

fn uflex_spec_expression(value: &str) -> String {
    if value.trim().is_empty() {
        "0".to_string()
    } else {
        uflex_expression(value, false)
    }
}

fn normalize_line_endings(value: &str) -> String {
    value.replace("\r\n", "\n")
}

fn build_instance_names(model: &Model) -> HashMap<usize, String> {
    let mut variants: indexmap::IndexMap<String, Vec<String>> = indexmap::IndexMap::new();
    let mut test_keys = HashMap::new();
    for (id, test) in &model.tests {
        let base = model
            .test_instance_group_name(*id)
            .unwrap_or(&test.name)
            .to_string();
        let signature = test
            .sorted_params()
            .filter(|(name, _, _)| *name != "test_name")
            .map(|(name, _, value)| {
                format!(
                    "{}={}",
                    name,
                    value.map(ToString::to_string).unwrap_or_default()
                )
            })
            .collect::<Vec<_>>()
            .join("\u{1f}");
        let group = variants.entry(base.clone()).or_insert_with(Vec::new);
        if !group.contains(&signature) {
            group.push(signature.clone());
        }
        test_keys.insert(*id, (base, signature));
    }

    test_keys
        .into_iter()
        .map(|(id, (base, signature))| {
            let group = &variants[&base];
            let name = if group.len() > 1 {
                let version = group.iter().position(|item| item == &signature).unwrap() + 1;
                format!("{}_v{}", base, version)
            } else {
                base
            };
            (id, name)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prog_gen::{FlowID, IGXLResource, Limit, LimitSelector, LimitType};
    use tempfile::tempdir;

    #[test]
    fn renders_flow_conditions_and_core_worksheets() -> Result<()> {
        let reference_values =
            IndexMap::from([("comment".to_string(), vec!["Block1".to_string()])]);
        let job_values = IndexMap::from([
            ("pinmap".to_string(), vec!["pinmap_test".to_string()]),
            (
                "instances".to_string(),
                vec!["prb1_instances".to_string(), "global_instances".to_string()],
            ),
            ("flows".to_string(), vec!["prb1_flow".to_string()]),
        ]);
        let global_spec_values = IndexMap::from([
            ("value".to_string(), vec!["=17".to_string()]),
            ("job".to_string(), vec!["FT".to_string()]),
            ("comment".to_string(), vec!["entering spec1".to_string()]),
        ]);
        let ac_spec_values = IndexMap::from([
            ("specset".to_string(), vec!["func_100MHz".to_string()]),
            ("selector".to_string(), vec!["nom".to_string()]),
            ("typ".to_string(), vec!["=10*ns".to_string()]),
            ("min".to_string(), vec!["=9*ns".to_string()]),
            ("max".to_string(), vec!["=11*ns".to_string()]),
        ]);
        let dc_spec_values = IndexMap::from([
            ("specset".to_string(), vec!["power_down_levels".to_string()]),
            ("selector".to_string(), vec!["nom".to_string()]),
            ("typ".to_string(), vec!["=0.2*V".to_string()]),
            ("min".to_string(), vec!["=0.1*V".to_string()]),
            ("max".to_string(), vec!["=0.3*V".to_string()]),
        ]);
        let pin_values = IndexMap::from([
            ("kind".to_string(), vec!["power".to_string()]),
            ("group".to_string(), vec![String::new()]),
            ("type".to_string(), vec!["Power".to_string()]),
            ("comment".to_string(), vec!["# vdd1".to_string()]),
        ]);
        let level_values = IndexMap::from([
            ("parameter".to_string(), vec!["VMain".to_string()]),
            ("value".to_string(), vec!["=_vdd_main_val".to_string()]),
            ("comment".to_string(), vec![String::new()]),
        ]);
        let edgeset_values = IndexMap::from([
            ("edgeset".to_string(), vec!["es1".to_string()]),
            ("src".to_string(), vec!["PAT".to_string()]),
            ("format".to_string(), vec!["NR".to_string()]),
            ("drive_on".to_string(), vec!["=0*ns".to_string()]),
            ("drive_data".to_string(), vec!["=1*ns".to_string()]),
            ("drive_return".to_string(), vec![String::new()]),
            ("drive_off".to_string(), vec![String::new()]),
            ("compare_mode".to_string(), vec!["Edge".to_string()]),
            ("compare_open".to_string(), vec!["=2*ns".to_string()]),
            ("compare_close".to_string(), vec!["=3*ns".to_string()]),
            ("resolution".to_string(), vec![String::new()]),
            ("timing_mode".to_string(), vec!["Machine".to_string()]),
        ]);
        let timeset_values = IndexMap::from([
            ("period".to_string(), vec!["=10*ns".to_string()]),
            ("pin".to_string(), vec!["tclk".to_string()]),
            ("edgeset".to_string(), vec!["es1".to_string()]),
            ("clock_period".to_string(), vec!["=10*ns".to_string()]),
            ("setup".to_string(), vec!["clock".to_string()]),
            ("timing_mode".to_string(), vec!["Machine".to_string()]),
        ]);
        let timeset_basic_values = IndexMap::from([
            ("period".to_string(), vec!["=10*ns".to_string()]),
            ("pin".to_string(), vec!["tclk".to_string()]),
            ("clock_period".to_string(), vec!["=10*ns".to_string()]),
            ("setup".to_string(), vec!["clock".to_string()]),
            ("src".to_string(), vec!["PAT".to_string()]),
            ("format".to_string(), vec!["NR".to_string()]),
            ("drive_on".to_string(), vec!["=0*ns".to_string()]),
            ("drive_data".to_string(), vec!["=1*ns".to_string()]),
            ("drive_return".to_string(), vec![String::new()]),
            ("drive_off".to_string(), vec![String::new()]),
            ("compare_mode".to_string(), vec!["Edge".to_string()]),
            ("compare_open".to_string(), vec!["=2*ns".to_string()]),
            ("compare_close".to_string(), vec!["=3*ns".to_string()]),
            ("resolution".to_string(), vec![String::new()]),
            ("timing_mode".to_string(), vec!["Machine".to_string()]),
        ]);
        let flow = node!(PGM::Flow, "prb1".to_string() =>
            node!(PGM::Group, "func_group".to_string(), Some(SupportedTester::ULTRAFLEX), crate::prog_gen::GroupType::Test, None =>
                node!(PGM::DefTest, 1, "func_ins".to_string(), SupportedTester::ULTRAFLEX, "std".to_string(), "functional".to_string()),
                node!(PGM::DefTest, 7, "func_ins_duplicate".to_string(), SupportedTester::ULTRAFLEX, "std".to_string(), "functional".to_string()),
                node!(PGM::DefTest, 8, "func_ins_variant".to_string(), SupportedTester::ULTRAFLEX, "std".to_string(), "functional".to_string())
            ),
            node!(PGM::DefTestInv, 2, "func".to_string(), SupportedTester::ULTRAFLEX),
            node!(PGM::AssignTestToInv, 2, 1),
            node!(PGM::SetAttr, 1, "pattern".to_string(), Some(ParamValue::Any("func_pset".to_string())), false),
            node!(PGM::SetAttr, 7, "pattern".to_string(), Some(ParamValue::Any("func_pset".to_string())), false),
            node!(PGM::SetAttr, 8, "pattern".to_string(), Some(ParamValue::Any("func_variant_pset".to_string())), false),
            node!(PGM::SetAttr, 2, "bin".to_string(), Some(ParamValue::Any("3".to_string())), false),
            node!(PGM::SetAttr, 2, "softbin".to_string(), Some(ParamValue::Any("100".to_string())), false),
            node!(PGM::SetLimit, None, Some(2), LimitSelector::Lo, Some(Limit { kind: LimitType::GTE, value: ParamValue::Float(-2.0), unit: Some("V".to_string()) })),
            node!(PGM::SetLimit, None, Some(2), LimitSelector::Hi, Some(Limit { kind: LimitType::LTE, value: ParamValue::Float(2.0), unit: Some("V".to_string()) })),
            node!(PGM::DefSubTest, 1, "lim1".to_string(), Some(1001),
                Some(Limit { kind: LimitType::GTE, value: ParamValue::Float(-1.0), unit: Some("V".to_string()) }),
                Some(Limit { kind: LimitType::LTE, value: ParamValue::Float(1.0), unit: Some("V".to_string()) })),
            node!(PGM::DefTest, 6, "custom_ins".to_string(), SupportedTester::ULTRAFLEX, "std".to_string(), "custom".to_string()),
            node!(PGM::SetAttr, 6, "test_name".to_string(), Some(ParamValue::Any("custom_ins".to_string())), false),
            node!(PGM::SetAttr, 6, "proc_name".to_string(), Some(ParamValue::Any("MyCustomProcedure".to_string())), false),
            node!(PGM::PatternGroup, 3, "func_pset".to_string(), SupportedTester::ULTRAFLEX, Some(PatternGroupType::Patset)),
            node!(PGM::PushPattern, 3, "func.PAT".to_string(), None),
            node!(PGM::PatternGroup, 4, "legacy_group".to_string(), SupportedTester::ULTRAFLEX, Some(PatternGroupType::Patgroup)),
            node!(PGM::PushPattern, 4, "legacy.PAT".to_string(), None),
            node!(PGM::PatternGroup, 5, "subroutines".to_string(), SupportedTester::ULTRAFLEX, Some(PatternGroupType::Patsubr)),
            node!(PGM::PushPattern, 5, "nvm_global_subs.PAT".to_string(), None),
            node!(PGM::Condition, FlowCondition::IfJob(vec!["prb1".to_string()]) =>
                node!(PGM::Test, 2, FlowID::from_str("func"))
            ),
            node!(PGM::TestStr, "guarded".to_string(), FlowID::from_str("guarded"), None, None, Some(10) =>
                node!(PGM::OnFailed, FlowID::from_str("guarded") =>
                    node!(PGM::Bin, 5, Some(50), BinType::Bad)
                )
            ),
            node!(PGM::Condition, FlowCondition::UnlessEnable(vec!["quick".to_string()]) =>
                node!(PGM::Log, "slow path".to_string())
            ),
            node!(PGM::Group, "group1".to_string(), None, crate::prog_gen::GroupType::Flow, Some(FlowID::from_str("group1")) =>
                node!(PGM::TestStr, "group_test1".to_string(), FlowID::from_str("group_test1"), None, None, Some(20)),
                node!(PGM::TestStr, "group_test2".to_string(), FlowID::from_str("group_test2"), None, None, Some(21))
            ),
            node!(PGM::Condition, FlowCondition::IfFailed(vec![FlowID::from_str("group1")]) =>
                node!(PGM::Log, "group failed".to_string())
            ),
            node!(PGM::ResourcesFilename, "shared".to_string(), ResourcesType::All),
            node!(PGM::IGXLResource, IGXLResource::new("references", ".\\inc\\file1.xla".to_string(), reference_values)?),
            node!(PGM::IGXLResource, IGXLResource::new("jobs", "FT".to_string(), job_values)?),
            node!(PGM::IGXLResource, IGXLResource::new("global_specs", "spec1".to_string(), global_spec_values)?),
            node!(PGM::IGXLResource, IGXLResource::new("ac_specs", "cycle".to_string(), ac_spec_values)?),
            node!(PGM::IGXLResource, IGXLResource::new("dc_specs", "vdd_main_val".to_string(), dc_spec_values)?),
            node!(PGM::IGXLResource, IGXLResource::new("pinmap", "vdd1".to_string(), pin_values)?),
            node!(PGM::IGXLResource, IGXLResource::new("levels", "vdd1".to_string(), level_values)?),
            node!(PGM::IGXLResource, IGXLResource::new("edgesets", "tclk".to_string(), edgeset_values)?),
            node!(PGM::IGXLResource, IGXLResource::new("timesets", "t1".to_string(), timeset_values)?),
            node!(PGM::IGXLResource, IGXLResource::new("timesets_basic", "t1".to_string(), timeset_basic_values)?),
            node!(PGM::Log, "done".to_string())
        );
        let mut ast = crate::ast::AST::new();
        ast.start(flow);
        let (ast, model) = process_flow(
            &ast,
            Model::new(SupportedTester::ULTRAFLEX),
            SupportedTester::ULTRAFLEX,
            true,
        )?;
        let dir = tempdir()?;
        let (_model, mut files, rows, patterns) = render_flow(&ast, dir.path(), model, "prb1")?;
        let mut resource_writer = FlowGenerator::new(Model::new(SupportedTester::ULTRAFLEX));
        resource_writer.resources_rows = rows;
        files.append(&mut resource_writer.write_resource_sheets(dir.path())?);
        files.push(write_referenced_list(dir.path(), patterns)?.unwrap());
        assert_eq!(files.len(), 7);
        let flow = std::fs::read_to_string(dir.path().join("prb1_flow.txt"))?;
        assert!(flow.contains("\tPRB1\t\t\tTest-defer-limits\tfunc_group_v1\tfunc"));
        assert!(flow.contains("\tUse-Limit\tfunc_group_v1\tlim1\t1001\t-1\t1"));
        let func_row = flow
            .lines()
            .find(|line| line.contains("\tfunc_group_v1\tfunc\t"))
            .expect("expected functional test row");
        let columns = func_row.split('\t').skip(1).collect::<Vec<_>>();
        assert_eq!(columns.len(), 32);
        assert_eq!(columns[9], "-2");
        assert_eq!(columns[10], "2");
        assert_eq!(columns[12], "V");
        assert_eq!(columns[15], "3");
        assert_eq!(columns[17], "100");
        assert_eq!(columns[18], "Fail");
        assert!(flow.contains("Test\tguarded\tguarded\t10"));
        assert!(flow.contains("guarded_FAILED"));
        assert!(flow.contains("quick\t\t\t\tgoto\tORIGEN_SKIP_1"));
        assert!(flow.contains("\tORIGEN_SKIP_1\t\t\t\t\tnop"));
        assert!(flow.contains("flag-false\tgroup1_FAILED"));
        assert!(flow.contains("flag-true\tgroup1_PASSED"));
        assert!(flow.contains("group_test1_FAILED"));
        assert!(flow.contains("flag-true\tgroup1_FAILED"));
        assert!(flow.contains("flag-false\tgroup1_PASSED"));
        let instances = std::fs::read_to_string(dir.path().join("prb1_instances.txt"))?;
        assert!(instances.contains("\tfunc_group_v1\tVBT\tFunctional_T\tExcel Macro"));
        assert!(instances.contains("\tfunc_group_v2\tVBT\tFunctional_T\tExcel Macro"));
        assert_eq!(instances.matches("\tfunc_group_v1\t").count(), 1);
        assert!(instances.contains("\tcustom_ins\tOther\tMyCustomProcedure\tVB DLL"));
        let patsets = std::fs::read_to_string(dir.path().join("prb1_patsets.txt"))?;
        assert!(patsets.contains("\tfunc_pset\t\t\tfunc.PAT\tYes"));
        let shared = std::fs::read_to_string(dir.path().join("shared.txt"))?;
        assert!(shared.contains("\t.\\inc\\file1.xla\tBlock1"));
        assert!(shared.contains("\tFT\tpinmap_test\tprb1_instances,global_instances\tprb1_flow"));
        let patgroups = std::fs::read_to_string(dir.path().join("prb1_patgroups.txt"))?;
        assert!(patgroups.contains("ULTRAFLEX DOES NOT SUPPORT PATTERN GROUP SHEETS!!"));
        let patsubrs = std::fs::read_to_string(dir.path().join("prb1_patsubrs.txt"))?;
        assert!(patsubrs.contains("\tnvm_global_subs.PAT\t"));
        assert!(shared.contains("\tspec1\tFT\t=17\tentering spec1"));
        assert!(shared.contains("\tcycle\t\tnom\tMax\t=10*ns\t=9*ns\t=11*ns"));
        assert!(shared.contains("\tvdd_main_val\t\tnom\tMax\t=0.2*V\t=0.1*V\t=0.3*V"));
        assert!(shared.contains("\t\tvdd1\tPower\t# vdd1"));
        assert!(shared.contains("\tvdd1\t\tVMain\t=_vdd_main_val\t"));
        assert!(shared.contains("\ttclk\tes1\tPAT\tNR\t=0*ns\t=1*ns"));
        assert!(shared.contains("\tt1\t=10*ns\ttclk\t=10*ns\tclock\tes1"));
        assert!(shared.contains("\tt1\t=10*ns\ttclk\t=10*ns\tclock\tPAT\tNR"));

        if let Ok(capture_dir) = std::env::var("O2_UFLEX_CAPTURE_DIR") {
            let capture_dir = PathBuf::from(capture_dir);
            std::fs::create_dir_all(&capture_dir)?;
            for file in &files {
                if let Some(name) = file.file_name() {
                    std::fs::copy(file, capture_dir.join(name))?;
                }
            }
        }
        let approved_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test_apps/python_app/approved/ultraflex/test_program");
        if std::env::var_os("O2_UFLEX_UPDATE_GOLDENS").is_some() {
            std::fs::create_dir_all(&approved_dir)?;
            for generated_path in &files {
                let name = generated_path.file_name().expect("generated file name");
                std::fs::copy(generated_path, approved_dir.join(name))?;
            }
        } else {
            for generated_path in &files {
                let name = generated_path.file_name().expect("generated file name");
                let generated = std::fs::read_to_string(generated_path)?;
                let approved = std::fs::read_to_string(approved_dir.join(name))?;
                assert_eq!(
                    normalize_line_endings(&generated),
                    normalize_line_endings(&approved),
                    "UltraFLEX golden mismatch for {:?}",
                    name
                );
            }
        }
        Ok(())
    }

    #[test]
    fn aggregates_shared_resources_from_multiple_flows() -> Result<()> {
        let mut writer = FlowGenerator::new(Model::new(SupportedTester::ULTRAFLEX));
        writer.resources_rows = vec![
            (
                "shared".to_string(),
                "references".to_string(),
                "flow1.xla".to_string(),
                IndexMap::from([("comment".to_string(), vec!["flow 1".to_string()])]),
            ),
            (
                "shared".to_string(),
                "references".to_string(),
                "flow2.xla".to_string(),
                IndexMap::from([("comment".to_string(), vec!["flow 2".to_string()])]),
            ),
        ];
        let dir = tempdir()?;
        let files = writer.write_resource_sheets(dir.path())?;
        assert_eq!(files, vec![dir.path().join("shared.txt")]);
        let references = std::fs::read_to_string(&files[0])?;
        assert!(references.contains("\tflow1.xla\tflow 1"));
        assert!(references.contains("\tflow2.xla\tflow 2"));
        Ok(())
    }

    #[test]
    fn one_neutral_flow_targets_ultraflex_and_v93k() -> Result<()> {
        let source = node!(PGM::Flow, "multi_target".to_string() =>
            node!(PGM::Log, "shared step".to_string()),
            node!(PGM::TesterEq, vec![SupportedTester::ULTRAFLEX] =>
                node!(PGM::TestStr, "uflex_only".to_string(), FlowID::from_str("uflex_only"), None, None, Some(10))
            ),
            node!(PGM::TesterEq, vec![SupportedTester::V93KSMT8] =>
                node!(PGM::TestStr, "v93k_only".to_string(), FlowID::from_str("v93k_only"), None, None, Some(20))
            )
        );
        let mut source_ast = crate::ast::AST::new();
        source_ast.start(source);

        let (uflex_ast, uflex_model) = process_flow(
            &source_ast,
            Model::new(SupportedTester::ULTRAFLEX),
            SupportedTester::ULTRAFLEX,
            true,
        )?;
        let uflex_dir = tempdir()?;
        let (_, _, _, _) = render_flow(&uflex_ast, uflex_dir.path(), uflex_model, "multi_target")?;
        let uflex = std::fs::read_to_string(uflex_dir.path().join("multi_target_flow.txt"))?;
        assert!(uflex.contains("shared step"));
        assert!(uflex.contains("uflex_only"));
        assert!(!uflex.contains("v93k_only"));

        let (v93k_ast, v93k_model) = process_flow(
            &source_ast,
            Model::new(SupportedTester::V93KSMT8),
            SupportedTester::V93KSMT8,
            true,
        )?;
        let v93k_dir = tempdir()?;
        let (_, v93k_files) = crate::prog_gen::advantest::smt8::processors::flow_generator::run(
            &v93k_ast,
            v93k_dir.path(),
            v93k_model,
        )?;
        let v93k_path = v93k_files
            .iter()
            .find(|path| path.extension().and_then(|ext| ext.to_str()) == Some("flow"))
            .expect("expected SMT8 flow output");
        let v93k = std::fs::read_to_string(v93k_path)?;
        assert!(v93k.contains("shared step"));
        assert!(v93k.contains("v93k_only"));
        assert!(!v93k.contains("uflex_only"));
        Ok(())
    }

    #[test]
    fn renders_supported_control_opcodes_and_rejects_unsupported_ones() -> Result<()> {
        assert_eq!(uflex_expression("10*ns", false), "=10*ns");
        assert_eq!(uflex_expression("=10*ns", false), "=10*ns");
        assert_eq!(uflex_expression("", true), "disable");
        assert_eq!(uflex_expression("", false), "");
        assert_eq!(uflex_spec_expression(""), "0");
        assert_eq!(normalize_line_endings("a\r\nb\r\n"), "a\nb\n");
        let mut model = Model::new(SupportedTester::ULTRAFLEX);
        model.create_flow("control")?;
        let mut generator = FlowGenerator::new(model);
        let supported = node!(PGM::Flow, "control".to_string() =>
            node!(PGM::Label, "RETRY".to_string()),
            node!(PGM::Goto, "RETRY".to_string()),
            node!(PGM::Comment, "retry loop".to_string()),
            node!(PGM::Condition, FlowCondition::IfFlag(vec!["FLAG1".to_string(), "FLAG2".to_string()]) =>
                node!(PGM::Log, "multi flag".to_string())
            )
        );
        supported.process(&mut generator)?;
        assert!(generator
            .rows
            .iter()
            .any(|row| row.starts_with("\tRETRY\t") && row.contains("\tnop\t")));
        assert!(generator
            .rows
            .iter()
            .any(|row| row.contains("\tgoto\tRETRY\t")));
        assert!(generator.rows.iter().any(|row| row.ends_with("retry loop")));
        let multi = generator
            .rows
            .iter()
            .find(|row| row.contains("multi flag"))
            .unwrap();
        let columns = multi.split('\t').skip(1).collect::<Vec<_>>();
        assert_eq!(columns[22], "any-active");
        assert_eq!(columns[24], "flag-true");
        assert_eq!(columns[25], "FLAG1,FLAG2");

        let unsupported = node!(PGM::Wait, "1ms".to_string());
        let error = unsupported.process(&mut generator).unwrap_err();
        assert!(error.to_string().contains("no UltraFLEX IG-XL equivalent"));
        Ok(())
    }

    #[test]
    fn supports_independent_resource_sheet_names() -> Result<()> {
        let mut model = Model::new(SupportedTester::ULTRAFLEX);
        model.create_flow("resources")?;
        let reference = IGXLResource::new(
            "references",
            "library.xla".to_string(),
            IndexMap::from([("comment".to_string(), vec!["shared library".to_string()])]),
        )?;
        let node = node!(PGM::Flow, "resources".to_string() =>
            node!(PGM::IGXLResourcesFilename, crate::prog_gen::IGXLResourceKind::References, "Refs".to_string()),
            node!(PGM::IGXLResource, reference)
        );
        let mut generator = FlowGenerator::new(model);
        node.process(&mut generator)?;
        assert_eq!(generator.resources_rows[0].0, "Refs");
        let dir = tempdir()?;
        let files = generator.write_resource_sheets(dir.path())?;
        assert_eq!(files, vec![dir.path().join("Refs.txt")]);
        Ok(())
    }
}
