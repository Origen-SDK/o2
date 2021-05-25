//! Utility functions for dealing with app/Origen version numbers

use crate::Result;
use regex::Regex;
use semver::Version;

lazy_static! {
    static ref VALID_VERSION: Regex = Regex::new(r#"^\d+\.\d+\.\d+([\.-]?[a-z]+\d+)?$"#).unwrap();
}

#[derive(Clone, Debug)]
pub enum ReleaseType {
    Major,
    Minor,
    Patch,
    Prerelease,
}

impl ReleaseType {
    pub fn to_vec() -> Vec<Self> {
        vec![Self::Major, Self::Minor, Self::Patch, Self::Prerelease]
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::Major => "Major",
            Self::Minor => "Minor",
            Self::Patch => "Patch",
            Self::Prerelease => "Prerelease",
        }
        .to_string()
    }

    pub fn from_idx(idx: usize) -> Self {
        Self::to_vec()[idx].clone()
    }

    pub fn bump_version(&self, v: &mut Version) -> Result<()> {
        // let mut v = version.clone();
        match self {
            Self::Major => v.increment_major(),
            Self::Minor => v.increment_minor(),
            Self::Patch => v.increment_patch(),
            Self::Prerelease => {
                // if v.is_pre() {
                //     match v.pre.find( |c| c.is_char()) {
                //         Some(pre_ver) => {
                //             let i = pre_ver.parse::<usize>();

                //         }
                //     }
                // }
                panic!("Prerelease not supported yet!");
            }
        }
        Ok(())
    }
}

/// Converts a version number like 1.2.3-dev4 to 1.2.3.dev4, the latter being compatible with
/// the Python PEP440 version number spec.
/// Version numbers without a dev number will be returned un-modified, as will any versions which
/// are already PEP440 compliant.
pub fn to_pep440(version: &str) -> Result<String> {
    if VALID_VERSION.is_match(version) {
        let v = version.replace("-", ".");
        Ok(v)
    } else {
        error!("Invalid version: '{}', must be a semantic version like 1.2.3 or 1.2.3.dev4 (1.2.3-dev4 also accepted)", &version)
    }
}

/// Converts a PEP440 version number like 1.2.3.dev4 to 1.2.3-dev4, the latter being compatible with
/// the semver spec.
/// Version numbers without a dev number will be returned un-modified, as will any versions which
/// are already semver compliant.
pub fn to_semver(version: &str) -> Result<String> {
    lazy_static! {
        static ref WITH_DEV: Regex = Regex::new(r#"^(\d+\.\d+\.\d+)[\.-]?([a-z]+\d+)$"#).unwrap();
    }
    if VALID_VERSION.is_match(version) {
        if WITH_DEV.is_match(version) {
            let cap = WITH_DEV.captures(version).unwrap();
            Ok(format!("{}-{}", &cap[1], &cap[2]))
        } else {
            Ok(version.to_string())
        }
    } else {
        error!("Invalid version: '{}', must be a semantic version like 1.2.3 or 1.2.3.dev4 (1.2.3-dev4 also accepted)", &version)
    }
}
