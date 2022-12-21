use crate::Result;

pub mod callbacks;
use crate::utility::version::Version;
use std::path::PathBuf;
use origen_metal::{Outcome, TypedValueVec, TypedValueMap};

use origen_metal::prelude::frontend::*;

pub fn with_frontend_app<T, F>(mut func: F) -> Result<T>
where
    F: FnMut(&dyn App) -> Result<T>,
{
    let handle = crate::FRONTEND.read().unwrap();
    match handle.frontend() {
        Some(fe) => match fe.app()? {
            Some(app) => func(app.as_ref()),
            None => bail!("No application is currently available!"),
        },
        None => bail!("No frontend is currently available!"),
    }
}

pub fn emit_callback(
    callback: &str,
    args: Option<TypedValueVec>,
    kwargs: Option<TypedValueMap>,
    opts: Option<TypedValueMap>,
) -> Result<TypedValueVec> {
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
            None => bail!("No frontend is currently available!"),
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
        args: Option<&TypedValueVec>,
        kwargs: Option<&TypedValueMap>,
        opts: Option<&TypedValueMap>,
    ) -> Result<TypedValueVec>;
    fn register_callback(&self, callback: &str, description: &str) -> Result<()>;
    fn list_local_dependencies(&self) -> Result<Vec<String>>;
    fn on_dut_change(&self) -> Result<()>;
}

pub trait App {
    fn rc(&self) -> Result<Option<&dyn RcAPI>>;
    fn unit_tester(&self) -> Result<Option<&dyn UnitTester>>;
    fn publisher(&self) -> Result<Option<&dyn Publisher>>;
    fn linter(&self) -> Result<Option<&dyn Linter>>;
    fn website(&self) -> Result<Option<&dyn Website>>;
    // TODO
    // fn mailer(&self) -> Result<Option<&dyn Mailer>>;
    fn release_scribe(&self) -> Result<Option<&dyn ReleaseScribe>>;

    fn get_rc(&self) -> Result<&dyn RcAPI> {
        match self.rc()? {
            Some(rc) => Ok(rc),
            None => bail!("No RC is available on the application!"),
        }
    }

    fn get_unit_tester(&self) -> Result<&dyn UnitTester> {
        match self.unit_tester()? {
            Some(ut) => Ok(ut),
            None => bail!("No unit tester is available on the application!"),
        }
    }

    fn get_publisher(&self) -> Result<&dyn Publisher> {
        match self.publisher()? {
            Some(pb) => Ok(pb),
            None => bail!("No publisher is available on the application!"),
        }
    }

    fn get_linter(&self) -> Result<&dyn Linter> {
        match self.linter()? {
            Some(l) => Ok(l),
            None => bail!("No linter is available on the application!"),
        }
    }

    fn get_website(&self) -> Result<&dyn Website> {
        match self.website()? {
            Some(w) => Ok(w),
            None => bail!("No website is available on the application!"),
        }
    }

    // TODO
    // fn get_mailer(&self) -> Result<&dyn Mailer> {
    //     match self.mailer()? {
    //         Some(m) => Ok(m),
    //         None => bail!("No mailer is available on the application!"),
    //     }
    // }

    fn get_release_scribe(&self) -> Result<&dyn ReleaseScribe> {
        match self.release_scribe()? {
            Some(rs) => Ok(rs),
            None => bail!("No release scribe is available on the application!"),
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
    fn run(&self) -> Result<Outcome>;
}

pub trait Publisher {
    fn build_package(&self) -> Result<Outcome>;
    fn upload(&self, build: &Outcome, dry_run: bool) -> Result<Outcome>;

    fn build_and_upload(&self, dry_run: bool) -> Result<(Outcome, Outcome)> {
        let br = self.build_package()?;
        Ok((br.clone(), self.upload(&br, dry_run)?))
    }
}

pub trait Website {
    fn build(&self) -> Result<Outcome>;
}

// TODO
// pub trait Mailer {
//     /// Returns the mailer's configuration
//     fn get_config(&self) -> Result<TypedValueMap>;

//     /// Sends an email
//     fn send(
//         &self,
//         from: &str,
//         to: Vec<&str>,
//         subject: Option<&str>,
//         body: Option<&str>,
//         include_origen_signature: bool,
//     ) -> Result<Outcome>;

//     /// Sends a test email. By default, sends only to the current user
//     fn test(&self, to: Option<Vec<&str>>) -> Result<Outcome>;
// }

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

