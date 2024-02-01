//! Utility functions for dealing with app/Origen version numbers

use crate::Result;
use semver;
use std::fmt;
use std::fs;
use std::path::PathBuf;
use crate::{toml_edit, dialoguer};
use crate::toml_edit::Document;

lazy_static! {
    static ref PYPROJECT_PATH: [&'static str; 3] = ["tool", "poetry", "version"];
    static ref CARGO_PATH: [&'static str; 2] = ["package", "version"];
}

const BETA: &str = "beta";
const ALPHA: &str = "alpha";
const DEV: &str = "dev";

#[derive(Debug, Clone, PartialEq)]
pub enum VersionSpec {
    Pep440,
    Semver,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Version {
    semver: semver::Version,
    spec: VersionSpec,
}

impl std::default::Default for Version {
    fn default() -> Self {
        Self::new("0.0.0-dev.0", VersionSpec::Semver).unwrap()
    }
}

impl Version {
    pub fn new(ver: &str, spec: VersionSpec) -> Result<Self> {
        let split = ver
            .splitn(4, |c| c == '.' || c == '-')
            .collect::<Vec<&str>>();
        let mut v: String = ver.to_string();
        let mut pre: Option<semver::Prerelease> = None;
        if split.len() == 4 {
            v = format!("{}.{}.{}", split[0], split[1], split[2]);

            // Check that the prerelease is of expected format
            let (t, num) = Self::split_prerelease(split[3])?;
            pre = Some(semver::Prerelease::new(&format!("{}.{}", t, num))?);
        } else if split.len() > 4 {
            bail!("Unexpected extra content after pre-release: '{}'", split[4]);
        }
        let mut semver = semver::Version::parse(&v)?;
        if let Some(p) = pre {
            semver.pre = p;
        }
        Ok(Self {
            semver: semver,
            spec: spec,
        })
    }

    pub fn new_semver(ver: &str) -> Result<Self> {
        Self::new(ver, VersionSpec::Semver)
    }

    pub fn new_pep440(ver: &str) -> Result<Self> {
        Self::new(ver, VersionSpec::Pep440)
    }

    fn split_prerelease(pre: &str) -> Result<(&str, usize)> {
        match pre.find(|c: char| c.is_digit(10)) {
            Some(i) => {
                let mut split = pre.split_at(i);
                split.0 = split.0.trim_end_matches(".");
                match split.0 {
                    DEV | ALPHA | BETA => Ok((split.0, split.1.parse::<usize>()?)),
                    _ => bail!(
                        "Expected prerelease of {}, {}, or {} but found {}",
                        DEV,
                        ALPHA,
                        BETA,
                        split.0
                    ),
                }
            }
            None => bail!(
                "Found existing prerelease '{}' but was unable to extract integer portion",
                pre
            ),
        }
    }

    pub fn increment_major(&mut self) -> &Self {
        self.semver = semver::Version::new(self.semver.major + 1, 0, 0);
        self
    }

    pub fn next_major(&self) -> Self {
        Self {
            semver: semver::Version::new(self.semver.major + 1, 0, 0),
            spec: self.spec.clone(),
        }
    }

    pub fn increment_minor(&mut self) -> &Self {
        self.semver = semver::Version::new(self.semver.major, self.semver.minor + 1, 0);
        self
    }

    pub fn next_minor(&self) -> Self {
        Self {
            semver: semver::Version::new(self.semver.major, self.semver.minor + 1, 0),
            spec: self.spec.clone(),
        }
    }

    pub fn increment_patch(&mut self) -> &Self {
        self.semver =
            semver::Version::new(self.semver.major, self.semver.minor, self.semver.patch + 1);
        self
    }

    pub fn next_patch(&self) -> Self {
        Self {
            semver: semver::Version::new(
                self.semver.major,
                self.semver.minor,
                self.semver.patch + 1,
            ),
            spec: self.spec.clone(),
        }
    }

    pub fn is_beta(&self) -> Result<bool> {
        self.is_of_prerelease(BETA)
    }

    pub fn increment_beta(&mut self) -> Result<&Self> {
        if self.is_beta()? {
            self.increment_existing_beta()?;
        } else if self.is_alpha()? || self.is_dev()? {
            self.force_prerelease(BETA)?;
        } else {
            self.append_beta(ReleaseType::Patch)?;
        }
        Ok(self)
    }

    pub fn next_beta(&self) -> Result<Self> {
        let mut next = self.clone();
        next.increment_beta()?;
        Ok(next)
    }

    pub fn increment_existing_beta(&mut self) -> Result<&Self> {
        self.increment_existing_prerelease(BETA)?;
        Ok(self)
    }

    pub fn append_beta(&mut self, release_type: ReleaseType) -> Result<&Self> {
        self.append_prerelease(BETA, release_type)?;
        Ok(self)
    }

    pub fn is_alpha(&self) -> Result<bool> {
        self.is_of_prerelease(ALPHA)
    }

    pub fn increment_alpha(&mut self) -> Result<&Self> {
        if self.is_alpha()? {
            self.increment_existing_alpha()?;
        } else if self.is_dev()? {
            self.force_prerelease(ALPHA)?;
        } else {
            self.append_alpha(ReleaseType::Patch)?;
        }
        Ok(self)
    }

    pub fn next_alpha(&self) -> Result<Self> {
        let mut next = self.clone();
        next.increment_alpha()?;
        Ok(next)
    }

    pub fn increment_existing_alpha(&mut self) -> Result<&Self> {
        self.increment_existing_prerelease(ALPHA)?;
        Ok(self)
    }

    pub fn append_alpha(&mut self, release_type: ReleaseType) -> Result<&Self> {
        self.append_prerelease(ALPHA, release_type)?;
        Ok(self)
    }

    pub fn is_dev(&self) -> Result<bool> {
        self.is_of_prerelease(DEV)
    }

    pub fn increment_dev(&mut self) -> Result<&Self> {
        if self.is_dev()? {
            self.increment_existing_dev()?;
        } else {
            self.append_dev(ReleaseType::Patch)?;
        }
        Ok(self)
    }

    pub fn next_dev(&self) -> Result<Self> {
        let mut next = self.clone();
        next.increment_dev()?;
        Ok(next)
    }

    /// Increments an existing dev or fails
    pub fn increment_existing_dev(&mut self) -> Result<&Self> {
        self.increment_existing_prerelease(DEV)?;
        Ok(self)
    }

    /// Increments the given release type and appends the dev.
    // Can go from, say, 0.1.0.dev4 to 1.0.0.dev0
    pub fn append_dev(&mut self, release_type: ReleaseType) -> Result<&Self> {
        self.append_prerelease(DEV, release_type)?;
        Ok(self)
    }

    pub fn is_prerelease(&self) -> Result<bool> {
        Ok(self.is_dev()? || self.is_alpha()? || self.is_beta()?)
    }

    fn is_of_prerelease(&self, prerelease: &str) -> Result<bool> {
        if self.semver.pre.is_empty() {
            return Ok(false);
        } else {
            match self.semver.pre.as_str().find(|c: char| c.is_digit(10)) {
                Some(i) => {
                    let mut split = self.semver.pre.as_str().split_at(i);
                    split.0 = split.0.trim_end_matches(".");
                    if split.0 == prerelease {
                        return Ok(true);
                    } else {
                        return Ok(false);
                    }
                }
                None => {
                    bail!(
                        "Found existing prerelease '{}' but was unable to extract integer portion",
                        self.semver.pre.as_str()
                    )
                }
            }
        }
    }

    fn increment_existing_prerelease(&mut self, prerelease: &str) -> Result<&Self> {
        if self.semver.pre.is_empty() {
            bail!("No {} release currently on version {}", prerelease, self);
        }
        match self.semver.pre.as_str().find(|c: char| c.is_digit(10)) {
            Some(i) => {
                let mut split = self.semver.pre.as_str().split_at(i);
                split.0 = split.0.trim_end_matches(".");
                if split.0 == prerelease {
                    // Same prerelease type - increment existing
                    let current = split.1.parse::<usize>()?;
                    self.semver.pre =
                        semver::Prerelease::new(&format!("{}.{}", prerelease, current + 1))?;
                } else {
                    bail!("Attempted to increment existing prerelease '{}' but found existing prerelease of '{}'", prerelease, split.0);
                }
            }
            None => {
                bail!(
                    "Found existing prerelease '{}' but was unable to extract integer portion",
                    self.semver.pre.as_str()
                )
            }
        }
        Ok(self)
    }

    fn append_prerelease(&mut self, prerelease: &str, release_type: ReleaseType) -> Result<&Self> {
        match release_type {
            ReleaseType::Major => self.increment_major(),
            ReleaseType::Minor => self.increment_minor(),
            ReleaseType::Patch => self.increment_patch(),
            _ => {
                bail!(
                    "Cannot create a {} tag from release type {:?}",
                    prerelease,
                    release_type
                )
            }
        };
        self.semver.pre = semver::Prerelease::new(&format!("{}.0", prerelease))?;
        Ok(self)
    }

    fn force_prerelease(&mut self, prerelease: &str) -> Result<&Self> {
        self.semver.pre = semver::Prerelease::new(&format!("{}.0", prerelease))?;
        Ok(self)
    }

    pub fn increment(&self, release_type: &ReleaseType) -> Result<Self> {
        Ok(match release_type {
            ReleaseType::Major => self.next_major(),
            ReleaseType::Minor => self.next_minor(),
            ReleaseType::Patch => self.next_patch(),
            ReleaseType::Beta => self.next_beta()?,
            ReleaseType::Alpha => self.next_alpha()?,
            ReleaseType::Dev => self.next_dev()?,
            ReleaseType::DevCustom | ReleaseType::AlphaCustom | ReleaseType::BetaCustom => {
                return Err("Error: Cannot use custom release: dialogue unavailable".into())
            }
        })
    }

    pub fn update_dialogue(&self) -> Result<Self> {
        let release_type = ReleaseType::from_idx(
            dialoguer::Select::new()
                .with_prompt("Please select the release type")
                .items(&ReleaseType::next_versions(self)?)
                .default(7)
                .interact()?,
        );

        Ok(match release_type {
            ReleaseType::Major => self.next_major(),
            ReleaseType::Minor => self.next_minor(),
            ReleaseType::Patch => self.next_patch(),
            ReleaseType::Beta => self.next_beta()?,
            ReleaseType::BetaCustom => {
                let mut v = self.clone();
                v.append_beta(Self::_update_custom_dialogue(
                    &ReleaseType::Beta.to_string(),
                )?)?;
                v
            }
            ReleaseType::Alpha => self.next_alpha()?,
            ReleaseType::AlphaCustom => {
                let mut v = self.clone();
                v.append_alpha(Self::_update_custom_dialogue(
                    &ReleaseType::Alpha.to_string(),
                )?)?;
                v
            }
            ReleaseType::Dev => self.next_dev()?,
            ReleaseType::DevCustom => {
                let mut v = self.clone();
                v.append_dev(Self::_update_custom_dialogue(
                    &ReleaseType::Dev.to_string(),
                )?)?;
                v
            }
        })
    }

    pub fn _update_custom_dialogue(release_type: &str) -> Result<ReleaseType> {
        Ok(ReleaseType::from_idx(
            dialoguer::Select::new()
                .with_prompt(&format!(
                    "Which official release would you like to make a {} release for?",
                    release_type
                ))
                .items(&ReleaseType::official_releases_as_strings())
                .default(2)
                .interact()?,
        ))
    }

    pub fn from_pyproject_with_toml_handle(pyproject: PathBuf) -> Result<VersionWithTOML> {
        VersionWithTOML::new(pyproject, &*PYPROJECT_PATH)
    }

    pub fn from_cargo_with_toml_handle(cargo_toml: PathBuf) -> Result<VersionWithTOML> {
        VersionWithTOML::new(cargo_toml, &*CARGO_PATH)
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.spec {
            VersionSpec::Semver => self.semver.fmt(f),
            VersionSpec::Pep440 => {
                let v = &self.semver;
                if v.pre.is_empty() {
                    write!(f, "{}.{}.{}", v.major, v.minor, v.patch)
                } else {
                    write!(
                        f,
                        "{}.{}.{}.{}",
                        v.major,
                        v.minor,
                        v.patch,
                        v.pre.replace(".", "")
                    )
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum ReleaseType {
    Major,
    Minor,
    Patch,
    Beta,
    BetaCustom,
    Alpha,
    AlphaCustom,
    Dev,
    DevCustom,
}

impl ReleaseType {
    pub fn to_vec() -> Vec<Self> {
        vec![
            Self::Major,
            Self::Minor,
            Self::Patch,
            Self::Beta,
            Self::BetaCustom,
            Self::Alpha,
            Self::AlphaCustom,
            Self::Dev,
            Self::DevCustom,
        ]
    }

    fn next_versions(v: &Version) -> Result<Vec<String>> {
        Ok(vec![
            format!("Major ({})", v.next_major()),
            format!("Minor ({})", v.next_minor()),
            format!("Patch ({})", v.next_patch()),
            format!("Next Beta ({})", v.next_beta()?),
            "Custom Dev (a.b.c.beta0)".to_string(),
            format!("Next Alpha ({})", v.next_alpha()?),
            "Custom Alpha (a.b.c.alpha0)".to_string(),
            format!("Dev ({})", v.next_dev()?),
            "Custom Dev (a.b.c.dev0)".to_string(),
        ])
    }

    fn official_releases_as_strings() -> Vec<String> {
        vec![
            Self::Major.to_string(),
            Self::Minor.to_string(),
            Self::Patch.to_string(),
        ]
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::Major => "Major",
            Self::Minor => "Minor",
            Self::Patch => "Patch",
            Self::Beta => "Beta",
            Self::BetaCustom => "Beta (Custom)",
            Self::Alpha => "Alpha",
            Self::AlphaCustom => "Alpha (Custom)",
            Self::Dev => "Dev",
            Self::DevCustom => "Dev (Custom)",
        }
        .to_string()
    }

    pub fn from_idx(idx: usize) -> Self {
        Self::to_vec()[idx].clone()
    }
}

impl fmt::Display for ReleaseType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl TryFrom<&str> for ReleaseType {
    type Error = String;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        Ok(match value.to_lowercase().as_str() {
            "major" => Self::Major,
            "minor" => Self::Minor,
            "patch" => Self::Patch,
            "beta" => Self::Beta,
            "alpha" => Self::Alpha,
            "dev" => Self::Dev,
            // Self::BetaCustom => "Beta (Custom)",
            // Self::AlphaCustom => "Alpha (Custom)",
            // Self::DevCustom => "Dev (Custom)",
            _ => return Err(format!("Unrecognized Release Type '{}'", value))
        })
    }
}

pub struct VersionWithTOML {
    pub toml: Document,
    source: PathBuf,
    orig_version: Version,
    new_version: Option<Version>,
    rel_type: Option<ReleaseType>,
    written: bool,
    version_path: &'static [&'static str],
}

impl VersionWithTOML {
    pub fn new(source: PathBuf, version_path: &'static [&'static str]) -> Result<Self> {
        if version_path.is_empty() {
            bail!("Version path should not be empty!");
        }

        let content = std::fs::read_to_string(&source)?;
        let toml = content.parse::<Document>().unwrap();
        let mut i: &toml_edit::Item = &toml[version_path[0]];
        for p in version_path[1..].iter() {
            i = &i[p];
        }
        let current = Version::new_pep440({
            match i.as_str() {
                Some(v) => v,
                None => bail!("Failed to parse version from TOML '{}'. 'version' not found or could not be parsed as a string", source.display())
            }
        })?;

        Ok(Self {
            orig_version: current,
            toml: toml,
            source: source,
            new_version: None,
            rel_type: None,
            written: false,
            version_path: version_path,
        })
    }

    pub fn increment(&mut self, increment: ReleaseType) -> Result<()> {
        self.new_version = Some(self.orig_version.increment(&increment)?);
        self.rel_type = Some(increment);
        Ok(())
    }

    pub fn was_version_updated(&self) -> bool {
        self.new_version.is_some()
    }

    pub fn orig_version(&self) -> &Version {
        &self.orig_version
    }

    pub fn new_version(&self) -> Option<&Version> {
        self.new_version.as_ref()
    }

    pub fn rel_type(&self) -> Option<&ReleaseType> {
        self.rel_type.as_ref()
    }

    pub fn source(&self) -> &PathBuf {
        &self.source
    }

    pub fn version(&self) -> &Version {
        if let Some(v) = self.new_version.as_ref() {
            v
        } else {
            &self.orig_version
        }
    }

    pub fn set_new_version(&mut self, new: Version) -> Result<()> {
        self.new_version = Some(new);
        Ok(())
    }

    pub fn write(&mut self) -> Result<()> {
        if let Some(v) = self.new_version() {
            let ver = v.to_string();
            let mut i: &mut toml_edit::Item = &mut self.toml[self.version_path[0]];
            for p in self.version_path[1..].iter() {
                i = &mut i[p];
            }

            *i = toml_edit::value(ver);
            fs::write(&self.source, self.toml.to_string())?;
            self.written = true;
            Ok(())
        } else {
            bail!("Version has not been updated! Nothing to update!");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_semver() {
        let v = Version::new_semver("0.0.0").unwrap();
        assert_eq!(v.to_string(), "0.0.0");

        let v = Version::new_semver("0.0.0.dev0").unwrap();
        assert_eq!(v.to_string(), "0.0.0-dev.0");

        let v = Version::new_semver("0.0.0-dev0").unwrap();
        assert_eq!(v.to_string(), "0.0.0-dev.0");
    }

    #[test]
    fn test_new_pep440() {
        let v = Version::new_pep440("0.0.0").unwrap();
        assert_eq!(v.to_string(), "0.0.0");

        let v = Version::new_pep440("0.0.0.dev0").unwrap();
        assert_eq!(v.to_string(), "0.0.0.dev0");

        let v = Version::new_pep440("0.0.0-dev0").unwrap();
        assert_eq!(v.to_string(), "0.0.0.dev0");
    }

    #[test]
    fn test_new() {
        let v = Version::new("1.2.3", VersionSpec::Semver).unwrap();
        assert_eq!(v.to_string(), "1.2.3");

        let v = Version::new("1.2.3.dev4", VersionSpec::Semver).unwrap();
        assert_eq!(v.to_string(), "1.2.3-dev.4");

        let v = Version::new("1.2.3.alpha0", VersionSpec::Semver).unwrap();
        assert_eq!(v.to_string(), "1.2.3-alpha.0");

        let v = Version::new("1.2.3.beta10", VersionSpec::Semver).unwrap();
        assert_eq!(v.to_string(), "1.2.3-beta.10");

        // Missing version number
        assert_eq!(
            Version::new("0.0.0.dev", VersionSpec::Semver).is_err(),
            true
        );

        // Invalid prerelease
        assert_eq!(
            Version::new("0.0.0.blah", VersionSpec::Semver).is_err(),
            true
        );
    }

    #[test]
    fn test_default() {
        let v = Version::default();
        assert_eq!(v.to_string(), "0.0.0-dev.0");

        let v = Version::new_pep440(&Version::default().to_string()).unwrap();
        assert_eq!(v.to_string(), "0.0.0.dev0");
    }

    #[test]
    fn test_bumping_major_version() {
        let mut v = Version::new_semver("0.0.0").unwrap();
        assert_eq!(v.to_string(), "0.0.0");
        v.increment_major();
        assert_eq!(v.to_string(), "1.0.0");

        let mut v = Version::new_semver("1.2.3").unwrap();
        assert_eq!(v.to_string(), "1.2.3");
        v.increment_major();
        assert_eq!(v.to_string(), "2.0.0");

        let mut v = Version::new_semver("3.2.1.dev0").unwrap();
        assert_eq!(v.to_string(), "3.2.1-dev.0");
        v.increment_major();
        assert_eq!(v.to_string(), "4.0.0");
    }

    #[test]
    fn test_bumping_minor_version() {
        let mut v = Version::new_semver("0.0.0").unwrap();
        assert_eq!(v.to_string(), "0.0.0");
        v.increment_minor();
        assert_eq!(v.to_string(), "0.1.0");

        let mut v = Version::new_semver("1.2.3").unwrap();
        assert_eq!(v.to_string(), "1.2.3");
        v.increment_minor();
        assert_eq!(v.to_string(), "1.3.0");

        let mut v = Version::new_semver("3.2.1.dev0").unwrap();
        assert_eq!(v.to_string(), "3.2.1-dev.0");
        v.increment_minor();
        assert_eq!(v.to_string(), "3.3.0");
    }

    #[test]
    fn test_bumping_patch_version() {
        let mut v = Version::new_semver("0.0.0").unwrap();
        assert_eq!(v.to_string(), "0.0.0");
        v.increment_patch();
        assert_eq!(v.to_string(), "0.0.1");

        let mut v = Version::new_semver("1.2.3").unwrap();
        assert_eq!(v.to_string(), "1.2.3");
        v.increment_patch();
        assert_eq!(v.to_string(), "1.2.4");

        let mut v = Version::new_semver("3.2.1.dev0").unwrap();
        assert_eq!(v.to_string(), "3.2.1-dev.0");
        v.increment_patch();
        assert_eq!(v.to_string(), "3.2.2");
    }

    #[test]
    fn test_increment_dev_version() {
        let mut v = Version::new_semver("0.0.0.dev0").unwrap();
        assert_eq!(v.to_string(), "0.0.0-dev.0");
        v.increment_dev().unwrap();
        assert_eq!(v.to_string(), "0.0.0-dev.1");

        let mut v = Version::new_semver("0.0.0").unwrap();
        assert_eq!(v.to_string(), "0.0.0");
        v.increment_dev().unwrap();
        assert_eq!(v.to_string(), "0.0.1-dev.0");

        let mut v = Version::new_semver("1.2.3").unwrap();
        assert_eq!(v.to_string(), "1.2.3");
        v.increment_dev().unwrap();
        assert_eq!(v.to_string(), "1.2.4-dev.0");

        let mut v = Version::new_semver("1.2.3.dev10").unwrap();
        assert_eq!(v.to_string(), "1.2.3-dev.10");
        v.increment_dev().unwrap();
        assert_eq!(v.to_string(), "1.2.3-dev.11");
    }

    #[test]
    fn test_increment_existing_dev_version() {
        let mut v = Version::new_semver("0.0.0.dev0").unwrap();
        assert_eq!(v.to_string(), "0.0.0-dev.0");
        v.increment_existing_dev().unwrap();
        assert_eq!(v.to_string(), "0.0.0-dev.1");

        let mut v = Version::new_semver("0.0.0").unwrap();
        assert_eq!(v.to_string(), "0.0.0");
        assert_eq!(v.increment_existing_dev().is_err(), true);
    }

    #[test]
    fn test_append_dev() {
        let mut v = Version::new_semver("0.0.0.dev0").unwrap();
        assert_eq!(v.to_string(), "0.0.0-dev.0");
        v.append_dev(ReleaseType::Major).unwrap();
        assert_eq!(v.to_string(), "1.0.0-dev.0");

        let mut v = Version::new_semver("0.0.0.dev1").unwrap();
        assert_eq!(v.to_string(), "0.0.0-dev.1");
        v.append_dev(ReleaseType::Minor).unwrap();
        assert_eq!(v.to_string(), "0.1.0-dev.0");

        let mut v = Version::new_semver("0.0.0.dev2").unwrap();
        assert_eq!(v.to_string(), "0.0.0-dev.2");
        v.append_dev(ReleaseType::Patch).unwrap();
        assert_eq!(v.to_string(), "0.0.1-dev.0");

        let mut v = Version::new_semver("0.0.0.dev3").unwrap();
        assert_eq!(v.to_string(), "0.0.0-dev.3");
        assert_eq!(v.append_dev(ReleaseType::Dev).is_err(), true);
    }

    #[test]
    fn test_increment_alpha_version() {
        let mut v = Version::new_semver("0.0.0.dev0").unwrap();
        assert_eq!(v.to_string(), "0.0.0-dev.0");
        v.increment_alpha().unwrap();
        assert_eq!(v.to_string(), "0.0.0-alpha.0");

        let mut v = Version::new_semver("0.0.0").unwrap();
        assert_eq!(v.to_string(), "0.0.0");
        v.increment_alpha().unwrap();
        assert_eq!(v.to_string(), "0.0.1-alpha.0");

        let mut v = Version::new_semver("1.2.3").unwrap();
        assert_eq!(v.to_string(), "1.2.3");
        v.increment_alpha().unwrap();
        assert_eq!(v.to_string(), "1.2.4-alpha.0");

        let mut v = Version::new_semver("1.2.3.alpha4").unwrap();
        assert_eq!(v.to_string(), "1.2.3-alpha.4");
        v.increment_alpha().unwrap();
        assert_eq!(v.to_string(), "1.2.3-alpha.5");

        let mut v = Version::new_semver("1.2.3.beta4").unwrap();
        assert_eq!(v.to_string(), "1.2.3-beta.4");
        v.increment_alpha().unwrap();
        assert_eq!(v.to_string(), "1.2.4-alpha.0");
    }

    #[test]
    fn test_increment_existing_alpha_version() {
        let mut v = Version::new_semver("0.0.0.alpha0").unwrap();
        assert_eq!(v.to_string(), "0.0.0-alpha.0");
        v.increment_existing_alpha().unwrap();
        assert_eq!(v.to_string(), "0.0.0-alpha.1");

        let mut v = Version::new_semver("0.0.0").unwrap();
        assert_eq!(v.to_string(), "0.0.0");
        assert_eq!(v.increment_existing_alpha().is_err(), true);

        let mut v = Version::new_semver("0.0.0.dev0").unwrap();
        assert_eq!(v.to_string(), "0.0.0-dev.0");
        assert_eq!(v.increment_existing_alpha().is_err(), true);
    }

    #[test]
    fn test_append_alpha() {
        let mut v = Version::new_semver("0.0.0.alpha0").unwrap();
        assert_eq!(v.to_string(), "0.0.0-alpha.0");
        v.append_alpha(ReleaseType::Major).unwrap();
        assert_eq!(v.to_string(), "1.0.0-alpha.0");

        let mut v = Version::new_semver("0.0.0.alpha1").unwrap();
        assert_eq!(v.to_string(), "0.0.0-alpha.1");
        v.append_alpha(ReleaseType::Minor).unwrap();
        assert_eq!(v.to_string(), "0.1.0-alpha.0");

        let mut v = Version::new_semver("0.0.0.alpha2").unwrap();
        assert_eq!(v.to_string(), "0.0.0-alpha.2");
        v.append_alpha(ReleaseType::Patch).unwrap();
        assert_eq!(v.to_string(), "0.0.1-alpha.0");

        let mut v = Version::new_semver("0.0.0.alpha3").unwrap();
        assert_eq!(v.to_string(), "0.0.0-alpha.3");
        assert_eq!(v.append_alpha(ReleaseType::Dev).is_err(), true);

        let mut v = Version::new_semver("0.0.0.alpha4").unwrap();
        assert_eq!(v.to_string(), "0.0.0-alpha.4");
        assert_eq!(v.append_alpha(ReleaseType::Alpha).is_err(), true);
    }

    #[test]
    fn test_increment_beta_version() {
        let mut v = Version::new_semver("0.0.0.dev0").unwrap();
        assert_eq!(v.to_string(), "0.0.0-dev.0");
        v.increment_beta().unwrap();
        assert_eq!(v.to_string(), "0.0.0-beta.0");

        let mut v = Version::new_semver("0.0.0").unwrap();
        assert_eq!(v.to_string(), "0.0.0");
        v.increment_beta().unwrap();
        assert_eq!(v.to_string(), "0.0.1-beta.0");

        let mut v = Version::new_semver("1.2.3").unwrap();
        assert_eq!(v.to_string(), "1.2.3");
        v.increment_beta().unwrap();
        assert_eq!(v.to_string(), "1.2.4-beta.0");

        let mut v = Version::new_semver("1.2.3.beta4").unwrap();
        assert_eq!(v.to_string(), "1.2.3-beta.4");
        v.increment_beta().unwrap();
        assert_eq!(v.to_string(), "1.2.3-beta.5");

        let mut v = Version::new_semver("1.2.3.alpha4").unwrap();
        assert_eq!(v.to_string(), "1.2.3-alpha.4");
        v.increment_beta().unwrap();
        assert_eq!(v.to_string(), "1.2.3-beta.0");
    }

    #[test]
    fn test_increment_existing_beta_version() {
        let mut v = Version::new_semver("0.0.0.beta0").unwrap();
        assert_eq!(v.to_string(), "0.0.0-beta.0");
        v.increment_existing_beta().unwrap();
        assert_eq!(v.to_string(), "0.0.0-beta.1");

        let mut v = Version::new_semver("0.0.0").unwrap();
        assert_eq!(v.to_string(), "0.0.0");
        assert_eq!(v.increment_existing_beta().is_err(), true);

        let mut v = Version::new_semver("0.0.0.dev0").unwrap();
        assert_eq!(v.to_string(), "0.0.0-dev.0");
        assert_eq!(v.increment_existing_beta().is_err(), true);
    }

    #[test]
    fn test_append_beta() {
        let mut v = Version::new_semver("0.0.0.beta0").unwrap();
        assert_eq!(v.to_string(), "0.0.0-beta.0");
        v.append_beta(ReleaseType::Major).unwrap();
        assert_eq!(v.to_string(), "1.0.0-beta.0");

        let mut v = Version::new_semver("0.0.0").unwrap();
        assert_eq!(v.to_string(), "0.0.0");
        v.append_beta(ReleaseType::Minor).unwrap();
        assert_eq!(v.to_string(), "0.1.0-beta.0");

        let mut v = Version::new_semver("0.0.0.beta2").unwrap();
        assert_eq!(v.to_string(), "0.0.0-beta.2");
        v.append_beta(ReleaseType::Patch).unwrap();
        assert_eq!(v.to_string(), "0.0.1-beta.0");

        let mut v = Version::new_semver("0.0.0.alpha3").unwrap();
        assert_eq!(v.to_string(), "0.0.0-alpha.3");
        assert_eq!(v.append_beta(ReleaseType::Dev).is_err(), true);

        let mut v = Version::new_semver("0.0.0.alpha4").unwrap();
        assert_eq!(v.to_string(), "0.0.0-alpha.4");
        assert_eq!(v.append_beta(ReleaseType::Beta).is_err(), true);
    }
}
