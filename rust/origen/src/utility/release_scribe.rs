use crate::{Result, STATUS};
use origen_metal::dialoguer::{Input, Select};
use origen_metal::utils::version::Version;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

const HISTORY_FILE_NAME: &str = "history";

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ReleaseProduct {
    Origen,
    OrigenMetal,
}

impl ReleaseProduct {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Origen => "Origen",
            Self::OrigenMetal => "Origen Metal",
        }
    }

    pub fn tag_prefix(&self) -> &'static str {
        match self {
            Self::Origen => "origen",
            Self::OrigenMetal => "origen-metal",
        }
    }
}

pub struct ReleaseScribe {
    pub history_file: PathBuf,
    pub release_file: PathBuf,
}

impl ReleaseScribe {
    // Currently requires an application, but should be updated for non-app use cases in future
    pub fn new(_config: &HashMap<String, String>) -> Result<Self> {
        Self::for_product(ReleaseProduct::Origen)
    }

    pub fn for_product(product: ReleaseProduct) -> Result<Self> {
        let dir;
        match &STATUS.app {
            Some(app) => dir = &app.root,
            None => {
                bail!("ReleaseScribe currently requires an application! No application found.")
            }
        }

        Ok(Self::for_root(product, dir))
    }

    pub fn for_root(product: ReleaseProduct, dir: &std::path::Path) -> Self {
        let history_file = match product {
            ReleaseProduct::Origen => dir.join("doc").join(HISTORY_FILE_NAME),
            ReleaseProduct::OrigenMetal => dir.join("doc").join("metal").join(HISTORY_FILE_NAME),
        };
        Self {
            history_file,
            release_file: PathBuf::from(format!("{}/release_note.txt", dir.display())),
        }
    }

    fn get_release_note_from_file_inner(&self) -> Result<String> {
        let mut content: String;
        let mut f = File::open(&self.release_file)?;
        content = String::new();
        f.read_to_string(&mut content)?;
        Ok(content)
    }

    pub fn get_release_note_from_file(&self) -> Result<String> {
        if self.release_file.exists() {
            Ok(self.get_release_note_from_file_inner()?)
        } else {
            bail!("No release note file at {}", self.release_file.display())
        }
    }

    pub fn get_release_note(&self) -> Result<String> {
        let content: String;
        if self.release_file.exists() {
            log_trace!("Found release note at {}", self.release_file.display());
            let _content = self.get_release_note_from_file_inner()?;
            if self.confirm_release_note_file_dialogue(&_content)? {
                content = _content;
            } else {
                content = self.release_body_dialog()?;
            }
        } else {
            log_trace!(
                "No release note found at {}. Running dialog...",
                self.release_file.display()
            );
            content = self.release_body_dialog()?;
        }
        Ok(content)
    }

    pub fn get_release_title(&self) -> Result<Option<String>> {
        self.release_title_dialog()
    }

    fn release_title_dialog(&self) -> Result<Option<String>> {
        let title: String = Input::new()
            .with_prompt("Enter release title (leave empty for no title)")
            .allow_empty(true)
            .interact()?;
        Ok(if title.is_empty() { None } else { Some(title) })
    }

    fn release_body_dialog(&self) -> Result<String> {
        let mut body: String;
        loop {
            body = Input::new().with_prompt("Enter release note").interact()?;

            if body.is_empty() {
                log_error!("Release body cannot be empty!");
            } else {
                return Ok(body);
            }
        }
    }

    fn confirm_release_note_file_dialogue(&self, content: &str) -> Result<bool> {
        let choice: usize = Select::new()
            .with_prompt(format!(
                "Found release note with content\n\
                 -------------------------------\n\
                 \n\
                 {}\
                 \n\n\
                 ----------------------\n\
                 Use this release note?",
                content
            ))
            .item("Yes")
            .item("No")
            .default(1)
            .interact()?;

        Ok(choice == 0)
    }

    pub fn create_history_file(&self) -> Result<()> {
        if let Some(parent) = self.history_file.parent() {
            std::fs::create_dir_all(parent)?;
        }
        File::create(&self.history_file)?;
        Ok(())
    }

    fn append_history_inner(
        &self,
        version: &Version,
        title: Option<String>,
        body: Option<String>,
    ) -> Result<()> {
        if !self.history_file.is_file() {
            log_trace!(
                "Creating release history at {}",
                self.history_file.display()
            );
            self.create_history_file()?;
        }

        let existing = std::fs::read_to_string(&self.history_file)?;
        let version_text = version.to_string();
        let anchor = version_text.replace('.', "-");
        let title = title.unwrap_or_else(|| format!("O2 {}", version_text));
        let body = body.unwrap_or_default();
        let entry = format!(
            "(release-{anchor})=\n# {title}\n\n- **Version:** `{version_text}`\n\
- **Released:** {}\n\n{body}\n\n---\n\n",
            chrono::Local::now().format("%Y-%m-%d %H:%M %Z"),
        );
        std::fs::write(&self.history_file, format!("{}{}", entry, existing))?;
        Ok(())
    }

    pub fn prepend_release(
        &self,
        product: ReleaseProduct,
        version: &Version,
        author: &str,
        body: &str,
        released: &str,
    ) -> Result<()> {
        if body.trim().is_empty() {
            bail!("Release note cannot be empty")
        }
        if !self.history_file.is_file() {
            self.create_history_file()?;
        }
        let existing = std::fs::read_to_string(&self.history_file)?;
        let version_text = version.to_string();
        let anchor = version_text.replace('.', "-");
        let tag = format!("{}-v{}", product.tag_prefix(), version_text);
        let entry = format!(
            "(release-{}-{})=\n# {} {}\n\n- **Released:** {}\n\
- **Author:** {}\n- **Tag:** `{}`\n\n{}\n\n---\n\n",
            product.tag_prefix(),
            anchor,
            product.display_name(),
            version_text,
            released,
            author,
            tag,
            body.trim(),
        );
        std::fs::write(&self.history_file, format!("{}{}", entry, existing))?;
        Ok(())
    }

    pub fn append_history(
        &mut self,
        version: &Version,
        title: Option<String>,
        body: Option<String>,
        dry_run: bool,
    ) -> Result<()> {
        if dry_run {
            log_trace!("Switching history file to dry-run temp file");
            self.history_file.set_file_name("history.dry_run");
        }
        let r = self.append_history_inner(version, title, body);
        if dry_run {
            log_trace!("Switching history file back");
            self.history_file.set_file_name(HISTORY_FILE_NAME);
        }
        r
    }

    pub fn read_history(&self) -> Result<String> {
        Ok(std::fs::read_to_string(&self.history_file)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn product_histories_are_independent_and_entries_are_prepended() -> Result<()> {
        let root = tempdir()?;
        let origen = ReleaseScribe::for_root(ReleaseProduct::Origen, root.path());
        let metal = ReleaseScribe::for_root(ReleaseProduct::OrigenMetal, root.path());
        assert_eq!(origen.history_file, root.path().join("doc/history"));
        assert_eq!(metal.history_file, root.path().join("doc/metal/history"));

        let v1 = Version::new_pep440("1.5.1")?;
        let v2 = Version::new_pep440("1.6.0")?;
        metal.prepend_release(
            ReleaseProduct::OrigenMetal,
            &v1,
            "Release Author",
            "Patch note",
            "2026-08-21",
        )?;
        metal.prepend_release(
            ReleaseProduct::OrigenMetal,
            &v2,
            "Release Author",
            "Minor note",
            "2026-08-22",
        )?;
        let history = metal.read_history()?;
        assert!(history.starts_with("(release-origen-metal-1-6-0)="));
        assert!(history.contains("**Tag:** `origen-metal-v1.6.0`"));
        assert!(history.find("Minor note").unwrap() < history.find("Patch note").unwrap());
        assert!(!origen.history_file.exists());
        Ok(())
    }

    #[test]
    fn empty_release_notes_are_rejected() -> Result<()> {
        let root = tempdir()?;
        let scribe = ReleaseScribe::for_root(ReleaseProduct::Origen, root.path());
        let version = Version::new_pep440("2.0.0.dev9")?;
        assert!(scribe
            .prepend_release(
                ReleaseProduct::Origen,
                &version,
                "Release Author",
                "  ",
                "2026-08-21",
            )
            .is_err());
        assert!(!scribe.history_file.exists());
        Ok(())
    }
}
