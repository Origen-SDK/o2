//! Utility functions for dealing with app/Origen version numbers

use crate::Result;
use regex::Regex;
use std::fmt;
use semver;

lazy_static! {
    static ref VALID_VERSION: Regex = Regex::new(r#"^\d+\.\d+\.\d+([\.-]?[a-z]+\d+)?$"#).unwrap();
}

const BETA: &str = "beta";
const ALPHA: &str = "alpha";
const DEV: &str = "dev";

#[derive(Debug, Clone)]
pub enum VersionSpec {
    Pep440,
    Semver,
}

#[derive(Debug, Clone)]
pub struct Version {
    semver: semver::Version,
    spec: VersionSpec,
}

impl std::default::Default for Version {
    fn default() -> Self {
        Self::new("0.0.0.pre0", VersionSpec::Semver).unwrap()
    }
}

impl Version {
    pub fn new(ver: &str, spec: VersionSpec) -> Result<Self> {
        let split = ver.splitn(5, ".").collect::<Vec<&str>>();
        let mut v: String = ver.to_string();
        let mut pre: Option<semver::Prerelease> = None;
        if split.len() == 4 {
            v = format!("{}.{}.{}", split[0], split[1], split[2]);

            // Check that the prerelease is of expected format
            Self::split_prerelease(split[3])?;
            pre = Some(semver::Prerelease::new(split[3])?);
        } else if split.len() > 4 {
            return error!("Unexpected extra content after pre-release: '{}'", split[4]);
        }
        let mut semver = semver::Version::parse(&v)?;
        if let Some(p) = pre {
            semver.pre = p;
        }
        Ok(Self { semver: semver, spec: spec })
    }

    pub fn new_semver(ver: &str) -> Result<Self> {
        Self::new(ver, VersionSpec::Semver)
    }

    pub fn new_pep440(ver: &str) -> Result<Self> {
        Self::new(ver, VersionSpec::Pep440)
    }

    fn split_prerelease(pre: &str) -> Result<(&str, usize)> {
        match pre.find( |c: char| c.is_digit(10)) {
            Some(i) => {
                let split = pre.split_at(i);
                match split.0 {
                    DEV | ALPHA | BETA => Ok((split.0, split.1.parse::<usize>()?)),
                    _ => error!(
                        "Expected prerelease of {}, {}, or {} but found {}",
                        DEV,
                        ALPHA,
                        BETA,
                        split.0
                    )
                }
            },
            None => error!("Found existing prerelease '{}' but was unable to extract integer portion", pre)
        }
    }

    pub fn increment_major(&mut self) -> &Self {
        self.semver = semver::Version::new(self.semver.major + 1, 0, 0);
        self
    }

    pub fn next_major(&self) -> Self {
        Self {
            semver: semver::Version::new(self.semver.major + 1, 0, 0),
            spec: self.spec.clone()
        }
    }

    pub fn increment_minor(&mut self) -> &Self {
        self.semver = semver::Version::new(self.semver.major, self.semver.minor + 1, 0);
        self
    }

    pub fn next_minor(&self) -> Self {
        Self {
            semver: semver::Version::new(self.semver.major, self.semver.minor + 1, 0),
            spec: self.spec.clone()
        }
    }

    pub fn increment_patch(&mut self) -> &Self {
        self.semver = semver::Version::new(self.semver.major, self.semver.minor, self.semver.patch + 1);
        self
    }

    pub fn next_patch(&self) -> Self {
        Self {
            semver: semver::Version::new(self.semver.major, self.semver.minor, self.semver.patch + 1),
            spec: self.spec.clone()
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

    fn is_of_prerelease(&self, prerelease: &str) -> Result<bool> {
        if self.semver.pre.is_empty() {
            return Ok(false);
        } else {
            match self.semver.pre.as_str().find( |c: char| c.is_digit(10)) {
                Some(i) => {
                    let split = self.semver.pre.as_str().split_at(i);
                    if split.0 == prerelease {
                        return Ok(true);
                    } else {
                        return Ok(false);
                    }
                },
                None => return error!("Found existing prerelease '{}' but was unable to extract integer portion", self.semver.pre.as_str())
            }
        }
    }

    fn increment_existing_prerelease(&mut self, prerelease: &str) -> Result<&Self> {
        if self.semver.pre.is_empty() {
            return error!("No {} release currently on version {}", prerelease, self);
        }
        match self.semver.pre.as_str().find( |c: char| c.is_digit(10)) {
            Some(i) => {
                let split = self.semver.pre.as_str().split_at(i);
                if split.0 == prerelease {
                    // Same prerelease type - increment existing
                    let current = split.1.parse::<usize>()?;
                    self.semver.pre = semver::Prerelease::new(&format!("{}{}", prerelease, current + 1))?;
                } else {
                    return error!("Attempted to increment existing prerelease '{}' but found existing prerelease of '{}'", prerelease, split.0);
                }
            }
            None => return error!("Found existing prerelease '{}' but was unable to extract integer portion", self.semver.pre.as_str())
        }
        Ok(self)
    }

    fn append_prerelease(&mut self, prerelease: &str, release_type: ReleaseType) -> Result<&Self> {
        match release_type {
            ReleaseType::Major => self.increment_major(),
            ReleaseType::Minor => self.increment_minor(),
            ReleaseType::Patch => self.increment_patch(),
            _ => return error!("Cannot create a {} tag from release type {:?}", prerelease, release_type)
        };
        self.semver.pre = semver::Prerelease::new(&format!("{}0", prerelease))?;
        Ok(self)
    }

    fn force_prerelease(&mut self, prerelease: &str) -> Result<&Self> {
        self.semver.pre = semver::Prerelease::new(&format!("{}0", prerelease))?;
        Ok(self)
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
                v.append_beta(Self::_update_custom_dialogue(&ReleaseType::Beta.to_string())?)?;
                v
            },
            ReleaseType::Alpha => self.next_alpha()?,
            ReleaseType::AlphaCustom => {
                let mut v = self.clone();
                v.append_alpha(Self::_update_custom_dialogue(&ReleaseType::Alpha.to_string())?)?;
                v
            },
            ReleaseType::Dev => self.next_dev()?,
            ReleaseType::DevCustom => {
                let mut v = self.clone();
                v.append_dev(Self::_update_custom_dialogue(&ReleaseType::Dev.to_string())?)?;
                v
            }
        })
    }

    pub fn _update_custom_dialogue(release_type: &str) -> Result<ReleaseType> {
        Ok(ReleaseType::from_idx(
            dialoguer::Select::new()
                .with_prompt(&format!("Which official release would you like to make a {} release for?", release_type))
                .items(&ReleaseType::official_releases_as_strings())
                .default(2)
                .interact()?
        ))
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Customize so only `x` and `y` are denoted.
        match self.spec {
            VersionSpec::Semver => self.semver.fmt(f),
            VersionSpec::Pep440 => match to_pep440(&self.semver.to_string()) {
                Ok(v) => write!(f, "{}", v),
                Err(_e) => Err(fmt::Error)
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
            Self::DevCustom
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_semver() {
        let v = Version::new_semver("0.0.0").unwrap();
        assert_eq!(v.to_string(), "0.0.0");

        let v = Version::new_semver("0.0.0.dev0").unwrap();
        assert_eq!(v.to_string(), "0.0.0-dev0");
    }

    #[test]
    fn test_new_pep440() {
        let v = Version::new_pep440("0.0.0").unwrap();
        assert_eq!(v.to_string(), "0.0.0");

        let v = Version::new_pep440("0.0.0.dev0").unwrap();
        assert_eq!(v.to_string(), "0.0.0.dev0");
    }

    #[test]
    fn test_new() {
        let v = Version::new("1.2.3", VersionSpec::Semver).unwrap();
        assert_eq!(v.to_string(), "1.2.3");

        let v = Version::new("1.2.3.dev4", VersionSpec::Semver).unwrap();
        assert_eq!(v.to_string(), "1.2.3-dev4");

        let v = Version::new("1.2.3.alpha0", VersionSpec::Semver).unwrap();
        assert_eq!(v.to_string(), "1.2.3-alpha0");

        let v = Version::new("1.2.3.beta10", VersionSpec::Semver).unwrap();
        assert_eq!(v.to_string(), "1.2.3-beta10");

        // Missing version number
        assert_eq!(Version::new("0.0.0.dev", VersionSpec::Semver).is_err(), true);

        // Invalid prerelease
        assert_eq!(Version::new("0.0.0.blah", VersionSpec::Semver).is_err(), true);
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
        assert_eq!(v.to_string(), "3.2.1-dev0");
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
        assert_eq!(v.to_string(), "3.2.1-dev0");
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
        assert_eq!(v.to_string(), "3.2.1-dev0");
        v.increment_patch();
        assert_eq!(v.to_string(), "3.2.2");
    }

    #[test]
    fn test_increment_dev_version() {
        let mut v = Version::new_semver("0.0.0.dev0").unwrap();
        assert_eq!(v.to_string(), "0.0.0-dev0");
        v.increment_dev().unwrap();
        assert_eq!(v.to_string(), "0.0.0-dev1");

        let mut v = Version::new_semver("0.0.0").unwrap();
        assert_eq!(v.to_string(), "0.0.0");
        v.increment_dev().unwrap();
        assert_eq!(v.to_string(), "0.0.1-dev0");

        let mut v = Version::new_semver("1.2.3").unwrap();
        assert_eq!(v.to_string(), "1.2.3");
        v.increment_dev().unwrap();
        assert_eq!(v.to_string(), "1.2.4-dev0");

        let mut v = Version::new_semver("1.2.3.dev10").unwrap();
        assert_eq!(v.to_string(), "1.2.3-dev10");
        v.increment_dev().unwrap();
        assert_eq!(v.to_string(), "1.2.3-dev11");
    }

    #[test]
    fn test_increment_existing_dev_version() {
        let mut v = Version::new_semver("0.0.0.dev0").unwrap();
        assert_eq!(v.to_string(), "0.0.0-dev0");
        v.increment_existing_dev().unwrap();
        assert_eq!(v.to_string(), "0.0.0-dev1");

        let mut v = Version::new_semver("0.0.0").unwrap();
        assert_eq!(v.to_string(), "0.0.0");
        assert_eq!(v.increment_existing_dev().is_err(), true);
    }

    #[test]
    fn test_append_dev() {
        let mut v = Version::new_semver("0.0.0.dev0").unwrap();
        assert_eq!(v.to_string(), "0.0.0-dev0");
        v.append_dev(ReleaseType::Major).unwrap();
        assert_eq!(v.to_string(), "1.0.0-dev0");

        let mut v = Version::new_semver("0.0.0.dev1").unwrap();
        assert_eq!(v.to_string(), "0.0.0-dev1");
        v.append_dev(ReleaseType::Minor).unwrap();
        assert_eq!(v.to_string(), "0.1.0-dev0");

        let mut v = Version::new_semver("0.0.0.dev2").unwrap();
        assert_eq!(v.to_string(), "0.0.0-dev2");
        v.append_dev(ReleaseType::Patch).unwrap();
        assert_eq!(v.to_string(), "0.0.1-dev0");

        let mut v = Version::new_semver("0.0.0.dev3").unwrap();
        assert_eq!(v.to_string(), "0.0.0-dev3");
        assert_eq!(v.append_dev(ReleaseType::Dev).is_err(), true);
    }

    #[test]
    fn test_increment_alpha_version() {
        let mut v = Version::new_semver("0.0.0.dev0").unwrap();
        assert_eq!(v.to_string(), "0.0.0-dev0");
        v.increment_alpha().unwrap();
        assert_eq!(v.to_string(), "0.0.0-alpha0");

        let mut v = Version::new_semver("0.0.0").unwrap();
        assert_eq!(v.to_string(), "0.0.0");
        v.increment_alpha().unwrap();
        assert_eq!(v.to_string(), "0.0.1-alpha0");

        let mut v = Version::new_semver("1.2.3").unwrap();
        assert_eq!(v.to_string(), "1.2.3");
        v.increment_alpha().unwrap();
        assert_eq!(v.to_string(), "1.2.4-alpha0");

        let mut v = Version::new_semver("1.2.3.alpha4").unwrap();
        assert_eq!(v.to_string(), "1.2.3-alpha4");
        v.increment_alpha().unwrap();
        assert_eq!(v.to_string(), "1.2.3-alpha5");

        let mut v = Version::new_semver("1.2.3.beta4").unwrap();
        assert_eq!(v.to_string(), "1.2.3-beta4");
        v.increment_alpha().unwrap();
        assert_eq!(v.to_string(), "1.2.4-alpha0");
    }

    #[test]
    fn test_increment_existing_alpha_version() {
        let mut v = Version::new_semver("0.0.0.alpha0").unwrap();
        assert_eq!(v.to_string(), "0.0.0-alpha0");
        v.increment_existing_alpha().unwrap();
        assert_eq!(v.to_string(), "0.0.0-alpha1");

        let mut v = Version::new_semver("0.0.0").unwrap();
        assert_eq!(v.to_string(), "0.0.0");
        assert_eq!(v.increment_existing_alpha().is_err(), true);

        let mut v = Version::new_semver("0.0.0.dev0").unwrap();
        assert_eq!(v.to_string(), "0.0.0-dev0");
        assert_eq!(v.increment_existing_alpha().is_err(), true);
    }

    #[test]
    fn test_append_alpha() {
        let mut v = Version::new_semver("0.0.0.alpha0").unwrap();
        assert_eq!(v.to_string(), "0.0.0-alpha0");
        v.append_alpha(ReleaseType::Major).unwrap();
        assert_eq!(v.to_string(), "1.0.0-alpha0");

        let mut v = Version::new_semver("0.0.0.alpha1").unwrap();
        assert_eq!(v.to_string(), "0.0.0-alpha1");
        v.append_alpha(ReleaseType::Minor).unwrap();
        assert_eq!(v.to_string(), "0.1.0-alpha0");

        let mut v = Version::new_semver("0.0.0.alpha2").unwrap();
        assert_eq!(v.to_string(), "0.0.0-alpha2");
        v.append_alpha(ReleaseType::Patch).unwrap();
        assert_eq!(v.to_string(), "0.0.1-alpha0");

        let mut v = Version::new_semver("0.0.0.alpha3").unwrap();
        assert_eq!(v.to_string(), "0.0.0-alpha3");
        assert_eq!(v.append_alpha(ReleaseType::Dev).is_err(), true);

        let mut v = Version::new_semver("0.0.0.alpha4").unwrap();
        assert_eq!(v.to_string(), "0.0.0-alpha4");
        assert_eq!(v.append_alpha(ReleaseType::Alpha).is_err(), true);
    }

    #[test]
    fn test_increment_beta_version() {
        let mut v = Version::new_semver("0.0.0.dev0").unwrap();
        assert_eq!(v.to_string(), "0.0.0-dev0");
        v.increment_beta().unwrap();
        assert_eq!(v.to_string(), "0.0.0-beta0");

        let mut v = Version::new_semver("0.0.0").unwrap();
        assert_eq!(v.to_string(), "0.0.0");
        v.increment_beta().unwrap();
        assert_eq!(v.to_string(), "0.0.1-beta0");

        let mut v = Version::new_semver("1.2.3").unwrap();
        assert_eq!(v.to_string(), "1.2.3");
        v.increment_beta().unwrap();
        assert_eq!(v.to_string(), "1.2.4-beta0");

        let mut v = Version::new_semver("1.2.3.beta4").unwrap();
        assert_eq!(v.to_string(), "1.2.3-beta4");
        v.increment_beta().unwrap();
        assert_eq!(v.to_string(), "1.2.3-beta5");

        let mut v = Version::new_semver("1.2.3.alpha4").unwrap();
        assert_eq!(v.to_string(), "1.2.3-alpha4");
        v.increment_beta().unwrap();
        assert_eq!(v.to_string(), "1.2.3-beta0");
    }

    #[test]
    fn test_increment_existing_beta_version() {
        let mut v = Version::new_semver("0.0.0.beta0").unwrap();
        assert_eq!(v.to_string(), "0.0.0-beta0");
        v.increment_existing_beta().unwrap();
        assert_eq!(v.to_string(), "0.0.0-beta1");

        let mut v = Version::new_semver("0.0.0").unwrap();
        assert_eq!(v.to_string(), "0.0.0");
        assert_eq!(v.increment_existing_beta().is_err(), true);

        let mut v = Version::new_semver("0.0.0.dev0").unwrap();
        assert_eq!(v.to_string(), "0.0.0-dev0");
        assert_eq!(v.increment_existing_beta().is_err(), true);
    }

    #[test]
    fn test_append_beta() {
        let mut v = Version::new_semver("0.0.0.beta0").unwrap();
        assert_eq!(v.to_string(), "0.0.0-beta0");
        v.append_beta(ReleaseType::Major).unwrap();
        assert_eq!(v.to_string(), "1.0.0-beta0");

        let mut v = Version::new_semver("0.0.0").unwrap();
        assert_eq!(v.to_string(), "0.0.0");
        v.append_beta(ReleaseType::Minor).unwrap();
        assert_eq!(v.to_string(), "0.1.0-beta0");

        let mut v = Version::new_semver("0.0.0.beta2").unwrap();
        assert_eq!(v.to_string(), "0.0.0-beta2");
        v.append_beta(ReleaseType::Patch).unwrap();
        assert_eq!(v.to_string(), "0.0.1-beta0");

        let mut v = Version::new_semver("0.0.0.alpha3").unwrap();
        assert_eq!(v.to_string(), "0.0.0-alpha3");
        assert_eq!(v.append_beta(ReleaseType::Dev).is_err(), true);

        let mut v = Version::new_semver("0.0.0.alpha4").unwrap();
        assert_eq!(v.to_string(), "0.0.0-alpha4");
        assert_eq!(v.append_beta(ReleaseType::Beta).is_err(), true);
    }
}