use std::path::PathBuf;

/// Stores a location and implements some helpers for discerning where its pointing
/// Support types so far:
///   url
///   git https
///   git ssh
///   path (Windows or Linux)
/// Notes:
///   * This does not check for existence and permissions. Simply stores the location and discerns its type
///   * A git HTTPS URL will also return as a URL
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Location {
  /// The raw location
  pub location: String
}

impl Location {
  pub fn new(loc: &str) -> Self {
    Self {
      location: loc.to_string()
    }
  }

  pub fn url(&self) -> Option<String> {
    if self.location.starts_with("http://") || self.location.starts_with("https://") {
      Some(self.location.clone())
    } else {
      None
    }
  }

  pub fn git(&self) -> Option<String> {
    if self.git_https().is_some() || self.git_ssh().is_some() {
      Some(self.location.clone())
    } else {
      None
    }
  }

  pub fn git_https(&self) -> Option<String> {
    if self.location.starts_with("https://") && self.location.ends_with(".git") {
      Some(self.location.clone())
    } else {
      None
    }
  }

  pub fn git_ssh(&self) -> Option<String> {
    if self.location.starts_with("git@") && self.location.ends_with(".git") {
      Some(self.location.clone())
    } else {
      None
    }
  }

  pub fn path(&self) -> Option<PathBuf> {
    // Anything else, assume its a path and convert it to a PathBuf
    if self.url().is_none() && self.git().is_none() {
      Some(PathBuf::from(&self.location))
    } else {
      None
    }
  }
}
