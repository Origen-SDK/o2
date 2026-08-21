use super::{write_sheet, FlowGenerator};
use crate::prog_gen::PatternGroupType;
use crate::Result;
use std::io::Write;
use std::path::{Path, PathBuf};

const PATSET_HEADER: [&str; 4] = [
    "DTPatternSetSheet,version=2.1:platform=Jaguar:toprow=-1:leftcol=-1:rightcol=-1\tPattern Sets",
    "",
    "\tPattern Set\tTD Group\tTime Domain\tFile/Group Name\tBurst\tStart Label\tStop Label\tComment",
    "",
];

#[derive(Clone)]
pub(super) struct PatsetPattern {
    pub(super) path: String,
    pub(super) start_label: Option<String>,
}

#[derive(Clone)]
pub(super) struct Patset {
    pub(super) name: String,
    pub(super) kind: PatternGroupType,
    pub(super) patterns: Vec<PatsetPattern>,
}

pub(super) fn write_referenced_list(
    output_dir: &Path,
    mut patterns: Vec<String>,
) -> Result<Option<PathBuf>> {
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

impl FlowGenerator {
    pub(super) fn write_patsets(&self, path: &Path) -> Result<()> {
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

    pub(super) fn write_patgroups(&self, path: &Path) -> Result<bool> {
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

    pub(super) fn write_patsubrs(&self, path: &Path) -> Result<bool> {
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
}
