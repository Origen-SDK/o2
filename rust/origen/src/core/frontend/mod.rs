use crate::{Metadata, Result};

pub mod callbacks;
use crate::utility::version::Version;
use indexmap::IndexMap;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub fn with_frontend_app<T, F>(mut func: F) -> Result<T>
where
    F: FnMut(&dyn App) -> Result<T>,
{
    let handle = crate::FRONTEND.read().unwrap();
    match handle.frontend() {
        Some(fe) => match fe.app()? {
            Some(app) => func(app.as_ref()),
            None => error!("No application is currently available!"),
        },
        None => error!("No frontend is currently available!"),
    }
}

pub fn emit_callback(
    callback: &str,
    args: Option<Vec<Metadata>>,
    kwargs: Option<IndexMap<String, Metadata>>,
    opts: Option<HashMap<String, Metadata>>,
) -> Result<Vec<Metadata>> {
    with_frontend(|f| f.emit_callback(callback, args.as_ref(), kwargs.as_ref(), opts.as_ref()))
}

pub fn with_frontend<T, F>(mut func: F) -> Result<T>
where
    F: FnMut(&dyn Frontend) -> Result<T>,
{
    let handle = crate::FRONTEND.read().unwrap();
    handle.with_frontend(|f| func(f))
}

pub fn with_optional_frontend<T, F>(mut func: F) -> Result<Option<T>>
where
    F: FnMut(&dyn Frontend) -> Result<T>,
{
    let handle = crate::FRONTEND.read().unwrap();
    handle.with_optional_frontend(|f| func(f))
}

pub struct Handle {
    frontend: Option<Box<dyn Frontend + std::marker::Sync + std::marker::Send>>,
}

impl Handle {
    pub fn new() -> Self {
        Self { frontend: None }
    }

    pub fn frontend(&self) -> Option<&dyn Frontend> {
        // Ok(&*(*self.frontend.as_ref().unwrap()))
        match self.frontend.as_ref() {
            Some(f) => Some(f.as_ref()),
            None => None,
        }
    }

    pub fn set_frontend(
        &mut self,
        frontend: Box<dyn Frontend + std::marker::Sync + std::marker::Send>,
    ) -> Result<()> {
        callbacks::register_callbacks(frontend.as_ref())?;
        self.frontend = Some(frontend);
        Ok(())
    }

    pub fn with_frontend<T, F>(&self, mut func: F) -> Result<T>
    where
        F: FnMut(&dyn Frontend) -> Result<T>,
    {
        match self.frontend.as_ref() {
            Some(f) => func(f.as_ref()),
            None => error!("No frontend is currently available!"),
        }
    }

    pub fn with_optional_frontend<T, F>(&self, mut func: F) -> Result<Option<T>>
    where
        F: FnMut(&dyn Frontend) -> Result<T>,
    {
        Ok(match self.frontend.as_ref() {
            Some(f) => Some(func(f.as_ref())?),
            None => None,
        })
    }
}

pub trait Frontend {
    fn app(&self) -> Result<Option<Box<dyn App>>>;
    fn emit_callback(
        &self,
        callback: &str,
        args: Option<&Vec<Metadata>>,
        kwargs: Option<&IndexMap<String, Metadata>>,
        opts: Option<&HashMap<String, Metadata>>,
    ) -> Result<Vec<Metadata>>;
    fn register_callback(&self, callback: &str, description: &str) -> Result<()>;
    fn list_local_dependencies(&self) -> Result<Vec<String>>;
    fn on_dut_change(&self) -> Result<()>;
}

pub trait App {
    fn rc(&self) -> Result<Option<&dyn RC>>;
    fn unit_tester(&self) -> Result<Option<&dyn UnitTester>>;
    fn publisher(&self) -> Result<Option<&dyn Publisher>>;
    fn linter(&self) -> Result<Option<&dyn Linter>>;
    fn website(&self) -> Result<Option<&dyn Website>>;
    fn mailer(&self) -> Result<Option<&dyn Mailer>>;
    fn release_scribe(&self) -> Result<Option<&dyn ReleaseScribe>>;

    fn get_rc(&self) -> Result<&dyn RC> {
        match self.rc()? {
            Some(rc) => Ok(rc),
            None => error!("No RC is available on the application!"),
        }
    }

    fn get_unit_tester(&self) -> Result<&dyn UnitTester> {
        match self.unit_tester()? {
            Some(ut) => Ok(ut),
            None => error!("No unit tester is available on the application!"),
        }
    }

    fn get_publisher(&self) -> Result<&dyn Publisher> {
        match self.publisher()? {
            Some(pb) => Ok(pb),
            None => error!("No publisher is available on the application!"),
        }
    }

    fn get_linter(&self) -> Result<&dyn Linter> {
        match self.linter()? {
            Some(l) => Ok(l),
            None => error!("No linter is available on the application!"),
        }
    }

    fn get_website(&self) -> Result<&dyn Website> {
        match self.website()? {
            Some(w) => Ok(w),
            None => error!("No website is available on the application!"),
        }
    }

    fn get_mailer(&self) -> Result<&dyn Mailer> {
        match self.mailer()? {
            Some(m) => Ok(m),
            None => error!("No mailer is available on the application!"),
        }
    }

    fn get_release_scribe(&self) -> Result<&dyn ReleaseScribe> {
        match self.release_scribe()? {
            Some(rs) => Ok(rs),
            None => error!("No release scribe is available on the application!"),
        }
    }

    // fn setup_production_status_checks(&self) -> Result<()>;
    // fn cleanup_production_status_checks(&self) -> Result<()>;
    // fn production_status_checks_pre(&self, stop_at_first_fail: bool) -> Result<IndexMap<String, (bool, String)>>;
    // fn production_status_checks_post(&self, stop_at_first_fail: bool) -> Result<IndexMap<String, (bool, String)>>;

    fn check_production_status(&self) -> Result<bool>;
    fn publish(&self) -> Result<()>;
}

pub trait Linter {}

pub trait UnitTester {
    fn run(&self) -> Result<UnitTestStatus>;
}

pub trait RC {
    fn is_modified(&self) -> Result<bool>;
    fn status(&self) -> Result<crate::revision_control::Status>;
    fn checkin(
        &self,
        files_or_dirs: Option<Vec<&Path>>,
        msg: &str,
        dry_run: bool,
    ) -> Result<GenericResult>;
    fn tag(&self, tag: &str, force: bool, msg: Option<&str>) -> Result<()>;
    fn system(&self) -> Result<String>;
    fn init(&self) -> Result<GenericResult>;
}

pub trait Publisher {
    fn build_package(&self) -> Result<BuildResult>;
    fn upload(&self, build: &BuildResult, dry_run: bool) -> Result<UploadResult>;

    fn build_and_upload(&self, dry_run: bool) -> Result<(BuildResult, UploadResult)> {
        let br = self.build_package()?;
        Ok((br.clone(), self.upload(&br, dry_run)?))
    }
}

pub trait Website {
    fn build(&self) -> Result<BuildResult>;
}

pub trait Mailer {}

pub trait ReleaseScribe {
    /// Returns the path to where a release note *could* be.
    /// The release note need not exists at this point, but this is where
    /// the release scribe will expect it to be.
    fn release_note_file(&self) -> Result<PathBuf>;

    /// Gets the release note, either from an existing file or from a release dialogue.
    fn get_release_note(&self) -> Result<String>;

    /// Gets the release note from an existing file or returns an error.
    fn get_release_note_from_file(&self) -> Result<String>;

    /// Starts dialogue to retrieve a release title
    fn get_release_title(&self) -> Result<Option<String>>;

    /// Returns the path to the history tracking file. This need not exists at this point,
    /// but is where it will be expected/created.
    fn history_tracking_file(&self) -> Result<PathBuf>;

    /// Updates the history file given the release, title, and release note body
    fn append_history(
        &self,
        version: &Version,
        title: Option<&str>,
        text: &str,
        dry_run: bool,
    ) -> Result<()>;

    // fn read_history(&self) -> Result<Option<ReleaseHistory>>;
    // fn last_update(&self) -> Result<Option<ReleaseHistory>>;

    /// Returns a list of all files that should be checked into revision control.
    /// By default, this is only the history toml file.
    fn rc_files(&self) -> Result<Vec<PathBuf>> {
        Ok(vec![self.history_tracking_file()?.clone()])
    }

    /// Goes through the process of getting release notes, updating the files, and
    /// returning whatever should be checked into revision control.
    /// Title and body can be given if derived elsewhere (e.g., from CLI),
    /// but normal retrieval process ensues if empty.
    fn publish(
        &self,
        release: &Version,
        title: Option<Option<&str>>,
        body: Option<&str>,
        dry_run: bool,
    ) -> Result<Vec<PathBuf>> {
        let t: String;
        let b: String;
        self.append_history(
            &release,
            match title.as_ref() {
                Some(title_) => *title_,
                None => {
                    if let Some(rt) = self.get_release_title()? {
                        t = rt;
                        Some(&t)
                    } else {
                        None
                    }
                }
            },
            match body.as_ref() {
                Some(body_) => body_,
                None => {
                    b = self.get_release_note()?;
                    &b
                }
            },
            dry_run,
        )?;
        self.rc_files()
    }
}

#[derive(Debug, Clone)]
pub struct UnitTestStatus {
    // tests: Vec<TestResult>,
    pub passed: Option<bool>,
    pub text: Option<String>,
}

impl UnitTestStatus {
    pub fn passed(&self) -> bool {
        match self.passed {
            Some(p) => p,
            None => {
                // for t in self.tests {
                //     if t.failed {
                //         self.passed = Some(false);
                //         return false
                //     }
                // }
                // self.passed = Some(true);
                true
            }
        }
    }

    //     fn non_empty_and_passed(&self) -> bool {
    //         match self.passed {
    //             Some(p) => p,
    //             None => {
    //                 if tests.is_empty() {
    //                     self.passed = false;
    //                     false
    //                 } else {
    //                     self.passed()
    //                 }
    //             }
    //         }
    //     }

    //     fn tests(&self) -> &Vec<TestResult> {
    //         &self.tests
    //     }
}

#[derive(Debug, Clone)]
pub struct BuildResult {
    pub succeeded: bool,
    pub build_contents: Option<Vec<String>>,
    pub message: Option<String>,
    pub metadata: Option<IndexMap<String, Metadata>>,
}

impl BuildResult {}

#[derive(Debug, Clone)]
pub struct UploadResult {
    pub succeeded: bool,
    pub message: Option<String>,
    pub metadata: Option<IndexMap<String, Metadata>>,
}

type AsNoun = String;
type AsVerb = String;

#[derive(Debug, Clone)]
pub enum GenericResultState {
    Success(AsNoun, AsVerb),
    Fail(AsNoun, AsVerb),
    Error(AsNoun, AsVerb),
}

impl GenericResultState {
    pub fn success() -> Self {
        Self::Success("Success".to_string(), "Succeeded".to_string())
    }

    pub fn pass() -> Self {
        Self::Success("Pass".to_string(), "Passed".to_string())
    }

    pub fn fail() -> Self {
        Self::Fail("Fail".to_string(), "Failed".to_string())
    }

    pub fn error() -> Self {
        Self::Error("Error".to_string(), "Errored".to_string())
    }

    pub fn as_verb(&self) -> String {
        match self {
            Self::Success(_, verb) => verb.to_string(),
            Self::Fail(_, verb) => verb.to_string(),
            Self::Error(_, verb) => verb.to_string(),
        }
    }

    pub fn as_noun(&self) -> String {
        match self {
            Self::Success(noun, _) => noun.to_string(),
            Self::Fail(noun, _) => noun.to_string(),
            Self::Error(noun, _) => noun.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GenericResult {
    pub state: GenericResultState,
    pub message: Option<String>,
    pub metadata: Option<IndexMap<String, Metadata>>,
}

impl std::fmt::Display for GenericResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_verb())
    }
}

impl GenericResult {
    pub fn new(state: GenericResultState) -> Self {
        Self {
            state,
            message: None,
            metadata: None,
        }
    }

    pub fn new_success() -> Self {
        Self::new(GenericResultState::success())
    }

    pub fn new_success_with_msg(message: impl std::fmt::Display) -> Self {
        let mut s = Self::new_success();
        s.set_msg(message);
        s
    }

    pub fn new_succeeded() -> Self {
        Self::new(GenericResultState::success())
    }

    pub fn new_succeeded_with_msg(message: impl std::fmt::Display) -> Self {
        let mut s = Self::new_succeeded();
        s.set_msg(message);
        s
    }

    pub fn new_pass() -> Self {
        Self::new(GenericResultState::pass())
    }

    pub fn new_passed() -> Self {
        Self::new(GenericResultState::pass())
    }

    pub fn new_fail() -> Self {
        Self::new(GenericResultState::fail())
    }

    pub fn new_failed() -> Self {
        Self::new(GenericResultState::fail())
    }

    pub fn new_error() -> Self {
        Self::new(GenericResultState::error())
    }

    pub fn new_errored() -> Self {
        Self::new(GenericResultState::error())
    }

    pub fn new_success_or_fail(success: bool) -> Self {
        if success {
            Self::new_success()
        } else {
            Self::new_fail()
        }
    }

    pub fn new_pass_or_fail(pass: bool) -> Self {
        if pass {
            Self::new_pass()
        } else {
            Self::new_fail()
        }
    }

    pub fn succeeded(&self) -> bool {
        match self.state {
            GenericResultState::Success(_, _) => true,
            _ => false,
        }
    }

    pub fn failed(&self) -> bool {
        match self.state {
            GenericResultState::Fail(_, _) => true,
            _ => false,
        }
    }

    pub fn errored(&self) -> bool {
        match self.state {
            GenericResultState::Error(_, _) => true,
            _ => false,
        }
    }

    pub fn set_msg(&mut self, message: impl std::fmt::Display) -> &mut Self {
        self.message = Some(message.to_string());
        self
    }

    pub fn add_metadata(&mut self, key: &str, m: Metadata) -> Result<&mut Self> {
        if self.metadata.is_none() {
            self.metadata = Some(IndexMap::new());
        }

        self.metadata.as_mut().unwrap().insert(key.to_string(), m);
        Ok(self)
    }

    pub fn gist(&self) {
        match &self.state {
            GenericResultState::Success(_, _) => {
                display_greenln!("{}", self.as_verb());
            }
            GenericResultState::Fail(_, _) => {
                display_redln!("{}", self.as_verb());
            }
            GenericResultState::Error(_, _) => {
                display_redln!("{}", self.as_verb());
            }
        }
    }

    pub fn summarize_and_exit(&self) {
        match &self.state {
            GenericResultState::Success(n, _) => {
                display_greenln!("{}", self.as_verb());
                if n == "Pass" {
                    exit_pass!();
                } else {
                    exit_success!();
                }
            }
            GenericResultState::Fail(_, _) => {
                display_redln!("{}", self.as_verb());
                exit_fail!();
            }
            GenericResultState::Error(_, _) => {
                display_redln!("{}", self.as_verb());
                exit_error!();
            }
        }
    }

    pub fn as_verb(&self) -> String {
        if let Some(m) = self.message.as_ref() {
            format!("{} with message: {}", self.state.as_verb(), m)
        } else {
            format!("{}", self.state.as_verb())
        }
    }
}
