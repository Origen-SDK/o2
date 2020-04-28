use super::{Credentials, Progress, RevisionControlAPI};
use crate::Result as OrigenResult;
use crate::USER;
use git2::Repository;
use std::env;
use std::path::{Path, PathBuf};

use git2::build::{CheckoutBuilder, RepoBuilder};
use git2::Progress as G2Progress;
use git2::{Cred, FetchOptions, RemoteCallbacks};
use std::cell::RefCell;
use std::io::{self, Write};

pub struct Git {
    /// Path to the local directory for the repository
    pub local: PathBuf,
    /// Link to the remote repository
    pub remote: String,
    credentials: Option<Credentials>,
    // There doesn't seem to be anything like an 'on_credentials_failed' callback, so have to keep track of
    // what password was attempted last so that it can be used to blow the caching of a bad password
    last_password: RefCell<Option<String>>,
}

impl RevisionControlAPI for Git {
    fn populate(
        &self,
        version: Option<&str>,
        mut callback: Option<&mut dyn FnMut(&Progress)>,
    ) -> OrigenResult<Progress> {
        log_info!("Started populating {}...", &self.remote);
        let state = RefCell::new(Progress::default());
        let mut cb = RemoteCallbacks::new();
        cb.transfer_progress(|stats| {
            let mut state = state.borrow_mut();
            state.total_objects = Some(stats.total_objects());
            state.received_objects = stats.received_objects();
            state.completed_objects = stats.indexed_objects();
            if let Some(f) = &mut callback {
                f(&*state);
            }
            true
        });
        cb.credentials(|url, username_from_url, allowed_types| {
            let password;
            let cred;
            {
                let lastpass = self.last_password.borrow();
                let (c, p) = get_credentials(
                    url,
                    username_from_url,
                    allowed_types,
                    &*lastpass,
                    self.credentials.clone().unwrap_or_default(),
                )?;
                password = p;
                cred = c;
            }
            self.last_password.replace(Some(password));
            Ok(cred)
        });
        let mut co = CheckoutBuilder::new();
        // This is called for every file, cur is the current file number, and total
        // is the total number of files that are being checked out
        co.progress(|path, _cur, _total| {
            if let Some(p) = path {
                log_debug!("{}  : Success", p.display());
            }
        });
        let mut fo = FetchOptions::new();
        fo.remote_callbacks(cb);
        RepoBuilder::new()
            .fetch_options(fo)
            .with_checkout(co)
            .clone(&self.remote, &self.local)?;

        Ok(state.into_inner())
    }

    fn checkout(&self, force: bool, path: Option<&Path>, version: Option<&str>,
        callback: Option<&mut dyn FnMut(&Progress)>,
    ) -> OrigenResult<Progress> {
        let repo = Repository::open(&self.local);
        Ok(Progress::default())
    }
}


impl Git {
    pub fn new(local: &Path, remote: &str, credentials: Option<Credentials>) -> Git {
        Git {
            local: local.to_path_buf(),
            remote: remote.to_string(),
            credentials: credentials,
            last_password: RefCell::new(None),
        }
    }

    //pub fn fetch(&self) -> OrigenResult<()> {
    //}
}

fn get_credentials(
    url: &str,
    _username_from_url: Option<&str>,
    _allowed_types: git2::CredentialType,
    last_attempt: &Option<String>,
    credentials: Credentials,
) -> Result<(Cred, String), git2::Error> {
    //println!("************************************************************");
    //println!("{:?}", url);
    //println!("{:?}", _username_from_url);
    //println!("{:?}", _allowed_types);
    //println!("************************************************************");
    //Cred::ssh_key(
    //  username_from_url.unwrap(),
    //  None,
    //  std::path::Path::new(&format!("{}/.ssh/id_rsa", env::var("HOME").unwrap())),
    //  None,
    //)

    let password = credentials.password.unwrap_or_else(|| {
        USER.password(
            Some(&format!("to access repository '{}'", url)),
            last_attempt.as_deref(),
        )
        .expect("Couldn't prompt for password")
    });
    let username = credentials.username.unwrap_or_else(|| USER.id().unwrap());
    Ok((Cred::userpass_plaintext(&username, &password)?, password))
}
