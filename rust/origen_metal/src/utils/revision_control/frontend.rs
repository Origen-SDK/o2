use super::Status;
use crate::{Outcome, Result};
use std::path::Path;

pub trait RevisionControlFrontendAPI {
    fn is_modified(&self) -> Result<bool>;
    fn status(&self) -> Result<Status>;
    fn checkin(
        &self,
        files_or_dirs: Option<Vec<&Path>>,
        msg: &str,
        dry_run: bool,
    ) -> Result<Outcome>;
    fn tag(&self, tag: &str, force: bool, msg: Option<&str>) -> Result<()>;
    fn system(&self) -> Result<String>;
    fn init(&self) -> Result<Outcome>;
}
