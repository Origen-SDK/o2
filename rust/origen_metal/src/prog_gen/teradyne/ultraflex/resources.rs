use crate::prog_gen::IGXLResourceKind;
use crate::Result;
use indexmap::{IndexMap, IndexSet};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub(super) struct ResourceRow {
    pub(super) sheet: String,
    pub(super) kind: IGXLResourceKind,
    pub(super) name: String,
    pub(super) values: IndexMap<String, Vec<String>>,
}

pub(super) struct ResourceGenerator {
    rows: Vec<ResourceRow>,
}

impl ResourceGenerator {
    pub(super) fn new(rows: Vec<ResourceRow>) -> Self {
        Self { rows }
    }
}

#[derive(Default)]
pub(super) struct PartFileGuard {
    tracked: Vec<PathBuf>,
    created: Vec<PathBuf>,
}

impl PartFileGuard {
    pub(super) fn track(&mut self, path: PathBuf) {
        self.tracked.push(path);
    }

    fn mark_created(&mut self, path: PathBuf) {
        self.created.push(path);
    }

    pub(super) fn keep(&mut self, path: &Path) {
        self.tracked.retain(|tracked| tracked != path);
    }

    fn created(&self) -> &[PathBuf] {
        &self.created
    }

    fn is_empty(&self) -> bool {
        self.created.is_empty()
    }
}

impl Drop for PartFileGuard {
    fn drop(&mut self) {
        for path in &self.tracked {
            let _ = std::fs::remove_file(path);
        }
    }
}

impl ResourceGenerator {
    pub(super) fn render(self, output_dir: &Path) -> Result<Vec<PathBuf>> {
        let all_resource_rows = self.rows;
        let mut files = vec![];
        let mut resource_sheets = vec![];
        for row in &all_resource_rows {
            if !resource_sheets.contains(&row.sheet) {
                resource_sheets.push(row.sheet.clone());
            }
        }
        for (sheet_index, sheet) in resource_sheets.into_iter().enumerate() {
            let rows = all_resource_rows
                .iter()
                .filter(|row| row.sheet == sheet)
                .cloned()
                .collect::<Vec<_>>();
            let mut parts = PartFileGuard::default();
            for (part_index, (suffix, writer)) in [
                (
                    "references",
                    ResourceGenerator::write_references
                        as fn(&[ResourceRow], &Path) -> Result<bool>,
                ),
                ("jobs", ResourceGenerator::write_jobs),
                ("global_specs", ResourceGenerator::write_global_specs),
                ("pinmap", ResourceGenerator::write_pinmap),
                ("levels", ResourceGenerator::write_levels),
                ("edgesets", ResourceGenerator::write_edgesets),
                ("timesets", ResourceGenerator::write_timesets),
                ("timesets_basic", ResourceGenerator::write_timesets_basic),
            ]
            .into_iter()
            .enumerate()
            {
                let path = output_dir.join(format!(
                    ".origen_uflex_{}_{}_{}.part",
                    sheet_index, part_index, suffix
                ));
                parts.track(path.clone());
                if writer(&rows, &path)? {
                    parts.mark_created(path);
                }
            }
            for (part_index, (kind, suffix, title)) in [
                (IGXLResourceKind::ACSpecs, "ac_specs", "AC"),
                (IGXLResourceKind::DCSpecs, "dc_specs", "DC"),
            ]
            .into_iter()
            .enumerate()
            {
                let path = output_dir.join(format!(
                    ".origen_uflex_{}_spec_{}_{}.part",
                    sheet_index, part_index, suffix
                ));
                parts.track(path.clone());
                if Self::write_specs(&rows, &path, kind, title)? {
                    parts.mark_created(path);
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
                parts.track(sheet_path.clone());
                let mut output = std::fs::File::create(&sheet_path)?;
                for part in parts.created() {
                    let mut input = std::fs::File::open(part)?;
                    std::io::copy(&mut input, &mut output)?;
                    std::fs::remove_file(part)?;
                }
                parts.keep(&sheet_path);
                files.push(sheet_path);
            }
        }
        Ok(files)
    }

    fn write_references(resource_rows: &[ResourceRow], path: &Path) -> Result<bool> {
        let rows = resource_rows
            .iter()
            .filter(|row| row.kind == IGXLResourceKind::References)
            .collect::<Vec<_>>();
        if rows.is_empty() {
            return Ok(false);
        }
        let mut file = std::fs::File::create(path)?;
        writeln!(file, "DTReferencesSheet,version=2.0:platform=Jaguar:toprow=-1:leftcol=-1:rightcol=-1\tReferences")?;
        writeln!(file)?;
        writeln!(file, "\tFile Path\tComment\t")?;
        for row in rows {
            writeln!(
                file,
                "\t{}\t{}",
                row.name,
                resource_value(&row.values, "comment")
            )?;
        }
        Ok(true)
    }

    fn write_jobs(resource_rows: &[ResourceRow], path: &Path) -> Result<bool> {
        let rows = resource_rows
            .iter()
            .filter(|row| row.kind == IGXLResourceKind::Jobs)
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
        for row in rows {
            let mut fields = vec![row.name.clone()];
            fields.extend(
                columns
                    .iter()
                    .map(|column| resource_value(&row.values, column)),
            );
            writeln!(file, "\t{}", fields.join("\t"))?;
        }
        Ok(true)
    }

    fn write_global_specs(resource_rows: &[ResourceRow], path: &Path) -> Result<bool> {
        let rows = resource_rows
            .iter()
            .filter(|row| row.kind == IGXLResourceKind::GlobalSpecs)
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
        rows.sort_by(|left, right| left.name.cmp(&right.name));
        for row in rows {
            writeln!(
                file,
                "\t{}\t{}\t{}\t{}",
                row.name,
                resource_value(&row.values, "job"),
                uflex_expression(&resource_value(&row.values, "value"), false),
                resource_value(&row.values, "comment")
            )?;
        }
        Ok(true)
    }

    pub(super) fn write_specs(
        resource_rows: &[ResourceRow],
        path: &Path,
        kind: IGXLResourceKind,
        title: &str,
    ) -> Result<bool> {
        let rows = resource_rows
            .iter()
            .filter(|row| row.kind == kind)
            .collect::<Vec<_>>();
        if rows.is_empty() {
            return Ok(false);
        }
        let mut specsets = IndexSet::new();
        for row in &rows {
            specsets.insert(resource_value(&row.values, "specset"));
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

        let mut keys = IndexSet::new();
        for row in &rows {
            keys.insert((row.name.clone(), resource_value(&row.values, "selector")));
        }
        let mut keys = keys.into_iter().collect::<Vec<_>>();
        keys.sort();
        for (symbol, selector) in keys {
            let mut category: Option<(&str, &str)> = None;
            let mut fields = vec![symbol.clone(), String::new(), selector.clone()];
            for specset in &specsets {
                let matching = rows.iter().find(|row| {
                    row.name == symbol
                        && resource_value(&row.values, "selector") == selector
                        && resource_value(&row.values, "specset") == *specset
                });
                if let Some(row) = matching {
                    let current_category = spec_category(&row.values);
                    if let Some((previous_category, previous_specset)) = category {
                        if previous_category != current_category {
                            bail!(
                                "UltraFLEX {} spec '{}' selector '{}' uses inconsistent categories across specsets: '{}' resolves to {} while '{}' resolves to {}",
                                title,
                                symbol,
                                selector,
                                previous_specset,
                                previous_category,
                                specset,
                                current_category,
                            )
                        }
                    } else {
                        category = Some((current_category, specset));
                    }
                    fields.push(uflex_spec_expression(&resource_value(&row.values, "typ")));
                    fields.push(uflex_spec_expression(&resource_value(&row.values, "min")));
                    fields.push(uflex_spec_expression(&resource_value(&row.values, "max")));
                } else {
                    fields.extend(["0".to_string(), "0".to_string(), "0".to_string()]);
                }
            }
            fields.insert(
                3,
                category
                    .map(|(category, _)| category)
                    .unwrap_or("Typ")
                    .to_string(),
            );
            let comment = rows
                .iter()
                .find(|row| {
                    row.name == symbol && resource_value(&row.values, "selector") == selector
                })
                .map(|row| resource_value(&row.values, "comment"))
                .unwrap_or_default();
            fields.push(comment);
            writeln!(file, "\t{}", fields.join("\t"))?;
        }
        Ok(true)
    }

    fn write_pinmap(resource_rows: &[ResourceRow], path: &Path) -> Result<bool> {
        let rows = resource_rows
            .iter()
            .filter(|row| row.kind == IGXLResourceKind::Pinmap)
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
            for row in rows
                .iter()
                .filter(|row| resource_value(&row.values, "kind") == wanted_kind)
            {
                let group = resource_value(&row.values, "group");
                let mut pin_type = resource_value(&row.values, "type");
                if wanted_kind == "group" && group == previous_group {
                    pin_type.clear();
                }
                writeln!(
                    file,
                    "\t{}\t{}\t{}\t{}",
                    group,
                    row.name,
                    pin_type,
                    resource_value(&row.values, "comment")
                )?;
                previous_group = group;
            }
        }
        Ok(true)
    }

    fn write_levels(resource_rows: &[ResourceRow], path: &Path) -> Result<bool> {
        let rows = resource_rows
            .iter()
            .filter(|row| row.kind == IGXLResourceKind::Levels)
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
        for row in rows {
            writeln!(
                file,
                "\t{}\t\t{}\t{}\t{}",
                row.name,
                resource_value(&row.values, "parameter"),
                uflex_expression(&resource_value(&row.values, "value"), false),
                resource_value(&row.values, "comment")
            )?;
        }
        Ok(true)
    }

    fn write_edgesets(resource_rows: &[ResourceRow], path: &Path) -> Result<bool> {
        let rows = resource_rows
            .iter()
            .filter(|row| row.kind == IGXLResourceKind::Edgesets)
            .collect::<Vec<_>>();
        if rows.is_empty() {
            return Ok(false);
        }
        let mut file = std::fs::File::create(path)?;
        let timing_mode = resource_value(&rows[0].values, "timing_mode");
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
        for row in rows {
            let mut fields = vec![row.name.clone()];
            for name in columns {
                let value = resource_value(&row.values, name);
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
            fields.push(resource_value(&row.values, "resolution"));
            fields.push(resource_value(&row.values, "comment"));
            writeln!(file, "\t{}", fields.join("\t"))?;
        }
        Ok(true)
    }

    fn write_timesets(resource_rows: &[ResourceRow], path: &Path) -> Result<bool> {
        let rows = resource_rows
            .iter()
            .filter(|row| row.kind == IGXLResourceKind::Timesets)
            .collect::<Vec<_>>();
        if rows.is_empty() {
            return Ok(false);
        }
        let mut file = std::fs::File::create(path)?;
        let timing_mode = resource_value(&rows[0].values, "timing_mode");
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
        for row in rows {
            writeln!(
                file,
                "\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                row.name,
                uflex_expression(&resource_value(&row.values, "period"), false),
                resource_value(&row.values, "pin"),
                uflex_expression(&resource_value(&row.values, "clock_period"), false),
                resource_value(&row.values, "setup"),
                resource_value(&row.values, "edgeset"),
                resource_value(&row.values, "comment")
            )?;
        }
        Ok(true)
    }

    fn write_timesets_basic(resource_rows: &[ResourceRow], path: &Path) -> Result<bool> {
        let rows = resource_rows
            .iter()
            .filter(|row| row.kind == IGXLResourceKind::TimesetsBasic)
            .collect::<Vec<_>>();
        if rows.is_empty() {
            return Ok(false);
        }
        let mut file = std::fs::File::create(path)?;
        let timing_mode = resource_value(&rows[0].values, "timing_mode");
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
        for row in rows {
            let mut fields = vec![row.name.clone()];
            for column in columns {
                let value = resource_value(&row.values, column);
                fields.push(match column {
                    "period" | "clock_period" => uflex_expression(&value, false),
                    "drive_on" | "drive_data" | "drive_return" | "drive_off" | "compare_open"
                    | "compare_close" => uflex_expression(&value, true),
                    _ => value,
                });
            }
            fields.push(String::new());
            fields.push(resource_value(&row.values, "resolution"));
            fields.push(resource_value(&row.values, "comment"));
            writeln!(file, "\t{}", fields.join("\t"))?;
        }
        Ok(true)
    }
}
fn resource_value(values: &IndexMap<String, Vec<String>>, name: &str) -> String {
    values.get(name).map(|v| v.join(",")).unwrap_or_default()
}

pub(super) fn uflex_expression(value: &str, disable_when_empty: bool) -> String {
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

pub(super) fn uflex_spec_expression(value: &str) -> String {
    if value.trim().is_empty() {
        "0".to_string()
    } else {
        uflex_expression(value, false)
    }
}

fn spec_category(values: &IndexMap<String, Vec<String>>) -> &'static str {
    if !resource_value(values, "max").is_empty() {
        "Max"
    } else if !resource_value(values, "min").is_empty() {
        "Min"
    } else {
        "Typ"
    }
}
