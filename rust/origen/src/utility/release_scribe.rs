use crate::utility::version::Version;
use crate::{Result, STATUS};
use dialoguer::{Input, Select};
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

const TITLE_ID: &str = "title";
const BODY_ID: &str = "body";
const DATETIME_ID: &str = "timestamp";

const HISTORY_FILE_NAME: &str = "history.generated.toml";

pub struct ReleaseScribe {
    pub history_toml: PathBuf,
    pub release_file: PathBuf,
}

impl ReleaseScribe {
    // Currently requires an application, but should be updated for non-app use cases in future
    pub fn new(_config: &HashMap<String, String>) -> Result<Self> {
        let dir;
        match &STATUS.app {
            Some(app) => dir = &app.root,
            None => {
                bail!("ReleaseScribe currently requires an application! No application found.")
            }
        }

        Ok(Self {
            history_toml: PathBuf::from(format!("{}/config/{}", dir.display(), HISTORY_FILE_NAME)),
            release_file: PathBuf::from(format!("{}/release_note.txt", dir.display())),
        })
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

    pub fn create_history_toml(&self) -> Result<()> {
        let mut f = File::create(&self.history_toml)?;
        f.write_all(
            format!(
                "# This file is automatically handled by Origen's release scribe.\n\
            # Initial version created with Origen-backend version {}\n\
            \n[releases]\n\n",
                &crate::STATUS.origen_version
            )
            .as_bytes(),
        )?;
        Ok(())
    }

    fn append_history_inner(
        &self,
        version: &Version,
        title: Option<String>,
        body: Option<String>,
    ) -> Result<()> {
        if !self.history_toml.is_file() {
            log_trace!(
                "Creating history toml file at {}",
                self.history_toml.display()
            );
            self.create_history_toml()?;
        }

        let mut m = toml::value::Map::new();
        m.insert(
            DATETIME_ID.to_string(),
            chrono::Local::now().to_string().into(),
        );
        if let Some(t) = title {
            m.insert(TITLE_ID.to_string(), t.into());
        }
        if let Some(b) = body {
            m.insert(BODY_ID.to_string(), b.into());
        }

        let mut f = std::fs::OpenOptions::new()
            .append(true)
            .open(&self.history_toml)?;
        f.write(
            format!(
                "[releases.\"{}\"]\n{}\n",
                version.to_string(),
                toml::to_string_pretty(&m)?
            )
            .as_bytes(),
        )?;
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
            self.history_toml
                .set_file_name(HISTORY_FILE_NAME.replace(".toml", ".dry_run.toml"));
        }
        let r = self.append_history_inner(version, title, body);
        if dry_run {
            log_trace!("Switching history file back");
            self.history_toml.set_file_name(HISTORY_FILE_NAME);
        }
        r
    }

    pub fn read_history(&self) -> Result<ReleaseHistory> {
        let mut f = File::open(&self.history_toml)?;
        let mut s = String::new();
        f.read_to_string(&mut s)?;
        let rs: ReleaseHistory = toml::from_str(&s)?;
        Ok(rs)
    }

    // pub fn write_history_rst(&self) -> Result<()> {
    // }
}

#[derive(Deserialize, Debug, Serialize)]
pub struct ReleaseHistory {
    releases: indexmap::IndexMap<String, Release>,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct Release {
    title: Option<String>,
    body: Option<String>,
    timestamp: Option<String>,
}
