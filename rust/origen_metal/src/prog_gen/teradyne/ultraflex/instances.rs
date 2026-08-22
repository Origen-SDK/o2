use super::FlowGenerator;
use crate::prog_gen::{Model, ParamValue};
use crate::Result;
use indexmap::{IndexMap, IndexSet};
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::Path;

const INSTANCE_PREFIX: [&str; 4] = [
    "DTTestInstancesSheet,version=2.4:platform=Jaguar:toprow=-1:leftcol=-1:rightcol=-1\tTest Instances",
    "",
    "\t\tTest Procedure\t\t\tDC Specs\t\tAC Specs\t\tSheet Parameters\t\t\t\t\tOther Parameters",
    "",
];

fn param(value: Option<&ParamValue>) -> String {
    value.map(ToString::to_string).unwrap_or_default()
}

pub(super) fn build_instance_names(model: &Model) -> HashMap<usize, String> {
    let mut variants: IndexMap<String, IndexSet<String>> = IndexMap::new();
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
        variants
            .entry(base.clone())
            .or_insert_with(IndexSet::new)
            .insert(signature.clone());
        test_keys.insert(*id, (base, signature));
    }

    test_keys
        .into_iter()
        .map(|(id, (base, signature))| {
            let group = &variants[&base];
            let name = if group.len() > 1 {
                let version = group
                    .get_index_of(&signature)
                    .expect("every test signature must be registered")
                    + 1;
                format!("{}_v{}", base, version)
            } else {
                base
            };
            (id, name)
        })
        .collect()
}

impl FlowGenerator {
    pub(super) fn write_instances(&self, path: &Path, flow_name: &str) -> Result<()> {
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
            let left_name = self
                .instance_names
                .get(left)
                .expect("every test must have a rendered UltraFLEX instance name");
            let right_name = self
                .instance_names
                .get(right)
                .expect("every test must have a rendered UltraFLEX instance name");
            left_name.cmp(right_name)
        });
        let mut rendered_names = HashSet::new();
        for id in ids {
            if let Some(test) = self.model.tests.get(&id) {
                let rendered_name = self
                    .instance_names
                    .get(&id)
                    .cloned()
                    .expect("every test must have a rendered UltraFLEX instance name");
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
                    let key = format!("arg{}", arg);
                    if test.has_param(&key) {
                        fields[13 + arg] = param(test.get(&key)?);
                    }
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
}
