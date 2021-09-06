pub mod frontend;

use std::path::PathBuf;

#[derive(Clone, Default, Debug)]
pub struct Status {
    pub added: Vec<PathBuf>,
    pub removed: Vec<PathBuf>,
    pub changed: Vec<PathBuf>,
    /// Files with merge conflicts
    pub conflicted: Vec<PathBuf>,
    pub revision: String,
}

impl Status {
    /// Returns true if the workspace status is modified in any way
    pub fn is_modified(&self) -> bool {
        !self.added.is_empty()
            || !self.removed.is_empty()
            || !self.changed.is_empty()
            || !self.conflicted.is_empty()
            || !self.added.is_empty()
    }

    pub fn summarize(&self) {
        displayln!("Workspace Status");
        if !self.added.is_empty() {
            displayln!("  ADDED: {} ITEMS", self.added.len());
            for file in &self.added {
                displayln!("    {}", file.display());
            }
        }
        if !self.removed.is_empty() {
            displayln!("  DELETED: {} ITEMS", self.removed.len());
            for file in &self.removed {
                displayln!("    {}", file.display());
            }
        }
        if !self.changed.is_empty() {
            displayln!("  CHANGED: {} ITEMS", self.changed.len());
            for file in &self.changed {
                displayln!("    {}", file.display());
            }
        }
        if !self.conflicted.is_empty() {
            displayln!("  CONFLICTED: {} ITEMS", self.conflicted.len());
            for file in &self.conflicted {
                display_redln!("    {}", file.display());
            }
        }
    }
}
