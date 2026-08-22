mod flow;
mod instances;
mod patterns;
mod resources;

use crate::ast::Node;
use crate::prog_gen::{process_flow, Model, SupportedTester, PGM};
use crate::{Result, FLOW};
use std::io::Write;
use std::path::{Path, PathBuf};

use flow::FlowGenerator;
use patterns::write_referenced_list;
use resources::{ResourceGenerator, ResourceRow};

const FLOW_HEADER: [&str; 4] = [
    "DTFlowtableSheet,version=2.2:platform=Jaguar:toprow=-1:leftcol=-1:rightcol=-1\tFlow Table",
    "\t\t\t\t\t\tFlow Domain:",
    "\t\t\tGate\t\t\tCommand\t\t\t\tLimits\t\tDatalog Display Results\t\t\tBin Number\t\tSort Number\t\t\tFlag\t\t\tGroup\t\t\t\tDevice\t\t\tDebug",
    "\tLabel\tEnable\tJob\tPart\tEnv\tOpcode\tParameter\tTName\tTNum\tLoLim\tHiLim\tScale\tUnits\tFormat\tPass\tFail\tPass\tFail\tResult\tPass\tFail\tState\tSpecifier\tSense\tCondition\tName\tSense\tCondition\tName\tAssume\tSites\tComment",
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
    generated.append(&mut ResourceGenerator::new(resource_rows).render(output_dir)?);
    if let Some(path) = write_referenced_list(output_dir, referenced_patterns)? {
        generated.push(path);
    }
    Ok((generated, model))
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

#[cfg(test)]
fn normalize_line_endings(value: &str) -> String {
    value.replace("\r\n", "\n")
}

#[cfg(test)]
mod tests;
