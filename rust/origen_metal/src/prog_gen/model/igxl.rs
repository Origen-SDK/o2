use crate::Result;
use indexmap::IndexMap;
use std::str::FromStr;

/// Typed IG-XL worksheet families supported by the shared program model.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
pub enum IGXLResourceKind {
    References,
    Jobs,
    GlobalSpecs,
    ACSpecs,
    DCSpecs,
    Pinmap,
    Levels,
    Edgesets,
    Timesets,
    TimesetsBasic,
}

impl IGXLResourceKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::References => "references",
            Self::Jobs => "jobs",
            Self::GlobalSpecs => "global_specs",
            Self::ACSpecs => "ac_specs",
            Self::DCSpecs => "dc_specs",
            Self::Pinmap => "pinmap",
            Self::Levels => "levels",
            Self::Edgesets => "edgesets",
            Self::Timesets => "timesets",
            Self::TimesetsBasic => "timesets_basic",
        }
    }

    fn allowed_fields(&self) -> &'static [&'static str] {
        match self {
            Self::References => &["comment"],
            Self::Jobs => &[
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
                "psets",
                "signals",
                "port_map",
                "fract_bus",
                "concurrent_seq",
                "comment",
            ],
            Self::GlobalSpecs => &["value", "job", "comment"],
            Self::ACSpecs | Self::DCSpecs => {
                &["specset", "selector", "typ", "min", "max", "comment"]
            }
            Self::Pinmap => &["kind", "group", "type", "comment"],
            Self::Levels => &["parameter", "value", "comment"],
            Self::Edgesets => &[
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
                "resolution",
                "timing_mode",
                "comment",
            ],
            Self::Timesets => &[
                "period",
                "pin",
                "edgeset",
                "clock_period",
                "setup",
                "timing_mode",
                "comment",
            ],
            Self::TimesetsBasic => &[
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
                "resolution",
                "timing_mode",
                "comment",
            ],
        }
    }

    fn required_fields(&self) -> &'static [&'static str] {
        match self {
            Self::References => &[],
            Self::Jobs => &[],
            Self::GlobalSpecs => &["value"],
            Self::ACSpecs | Self::DCSpecs => &["specset", "selector"],
            Self::Pinmap => &["kind", "type"],
            Self::Levels => &["parameter", "value"],
            Self::Edgesets => &["edgeset", "src", "format", "compare_mode", "timing_mode"],
            Self::Timesets => &["period", "pin", "edgeset", "setup", "timing_mode"],
            Self::TimesetsBasic => &[
                "period",
                "pin",
                "setup",
                "src",
                "format",
                "compare_mode",
                "timing_mode",
            ],
        }
    }
}

impl FromStr for IGXLResourceKind {
    type Err = crate::Error;

    fn from_str(value: &str) -> Result<Self> {
        match value.to_ascii_lowercase().as_str() {
            "references" => Ok(Self::References),
            "jobs" => Ok(Self::Jobs),
            "global_specs" => Ok(Self::GlobalSpecs),
            "ac_specs" => Ok(Self::ACSpecs),
            "dc_specs" => Ok(Self::DCSpecs),
            "pinmap" => Ok(Self::Pinmap),
            "levels" => Ok(Self::Levels),
            "edgesets" => Ok(Self::Edgesets),
            "timesets" => Ok(Self::Timesets),
            "timesets_basic" => Ok(Self::TimesetsBasic),
            _ => bail!("Unknown IG-XL resource type '{}'", value),
        }
    }
}

/// A validated row in an IG-XL resource family.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct IGXLResource {
    pub kind: IGXLResourceKind,
    pub name: String,
    pub values: IndexMap<String, Vec<String>>,
}

impl IGXLResource {
    pub fn new(
        kind: impl AsRef<str>,
        name: String,
        values: IndexMap<String, Vec<String>>,
    ) -> Result<Self> {
        let kind = IGXLResourceKind::from_str(kind.as_ref())?;
        for field in values.keys() {
            if !kind.allowed_fields().contains(&field.as_str()) {
                bail!(
                    "Unknown field '{}' for IG-XL {} resource '{}'",
                    field,
                    kind.as_str(),
                    name
                );
            }
        }
        for field in kind.required_fields() {
            if !values.contains_key(*field) {
                bail!(
                    "Missing required field '{}' for IG-XL {} resource '{}'",
                    field,
                    kind.as_str(),
                    name
                );
            }
        }
        Ok(Self { kind, name, values })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unknown_resource_fields() {
        let values = IndexMap::from([("drive_retrun".to_string(), vec!["1ns".to_string()])]);
        let error = IGXLResource::new("edgesets", "es1".to_string(), values).unwrap_err();
        assert!(error.to_string().contains("drive_retrun"));
    }
}
