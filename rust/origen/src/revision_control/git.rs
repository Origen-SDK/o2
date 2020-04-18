use super::RevisionControlAPI;
use crate::Result as OrigenResult;
use crate::USER;
use git2::Repository;
use std::env;
use std::path::{Path, PathBuf};

use git2::build::{CheckoutBuilder, RepoBuilder};
use git2::{Cred, FetchOptions, Progress, RemoteCallbacks};
use std::cell::RefCell;
use std::io::{self, Write};

pub struct Git {
    /// Path to the local directory for the repository
    pub local: PathBuf,
    /// Link to the remote repository
    pub remote: String,
}

impl Git {
    pub fn new(local: &Path, remote: &str) -> Git {
        Git {
            local: local.to_path_buf(),
            remote: remote.to_string(),
        }
    }
}

impl RevisionControlAPI for Git {
    //fn populate(&self, version: Option<String>) -> Result<()> {
    //    let repo = match Repository::clone(&self.remote, &self.local) {
    //        Ok(repo) => repo,
    //        Err(e) => panic!("failed to clone: {}", e),
    //    };
    //    Ok(())
    //}

    fn populate(&self, version: Option<String>) -> OrigenResult<()> {
        let state = RefCell::new(State {
            progress: None,
            total: 0,
            current: 0,
            path: None,
            newline: false,
        });
        let mut cb = RemoteCallbacks::new();
        cb.transfer_progress(|stats| {
            let mut state = state.borrow_mut();
            state.progress = Some(stats.to_owned());
            print(&mut *state);
            true
        });
        let mut attempts = 0;
        cb.credentials(|url, username_from_url, allowed_types| {
            attempts += 1;
            get_credentials(url, username_from_url, allowed_types, attempts)
        });
        let mut co = CheckoutBuilder::new();
        co.progress(|path, cur, total| {
            let mut state = state.borrow_mut();
            state.path = path.map(|p| p.to_path_buf());
            state.current = cur;
            state.total = total;
            print(&mut *state);
        });

        let mut fo = FetchOptions::new();
        fo.remote_callbacks(cb);
        RepoBuilder::new()
            .fetch_options(fo)
            .with_checkout(co)
            .clone(&self.remote, &self.local)
            .expect("It went OK");
        println!();

        Ok(())
    }
}

fn get_credentials(
    url: &str,
    _username_from_url: Option<&str>,
    _allowed_types: git2::CredentialType,
    attempts: i32,
) -> Result<Cred, git2::Error> {
    //println!("************************************************************");
    //println!("{:?}", url);
    //println!("{:?}", _username_from_url);
    //println!("{:?}", _allowed_types);
    //println!("{:?}", attempts);
    //println!("************************************************************");
    //Cred::ssh_key(
    //  username_from_url.unwrap(),
    //  None,
    //  std::path::Path::new(&format!("{}/.ssh/id_rsa", env::var("HOME").unwrap())),
    //  None,
    //)
    let password = USER
        .password(
            Some(format!("to access repository '{}'", url)),
            attempts > 1,
        )
        .expect("FUuuuuuuu");
    let id = USER.id().unwrap();
    Cred::userpass_plaintext(&id, &password)
}

struct State {
    progress: Option<Progress<'static>>,
    total: usize,
    current: usize,
    path: Option<PathBuf>,
    newline: bool,
}

fn print(state: &mut State) {
    let stats = state.progress.as_ref().unwrap();
    let network_pct = (100 * stats.received_objects()) / stats.total_objects();
    let index_pct = (100 * stats.indexed_objects()) / stats.total_objects();
    let co_pct = if state.total > 0 {
        (100 * state.current) / state.total
    } else {
        0
    };
    let kbytes = stats.received_bytes() / 1024;
    if stats.received_objects() == stats.total_objects() {
        if !state.newline {
            println!();
            state.newline = true;
        }
        print!(
            "Resolving deltas {}/{}\r",
            stats.indexed_deltas(),
            stats.total_deltas()
        );
    } else {
        print!(
            "net {:3}% ({:4} kb, {:5}/{:5})  /  idx {:3}% ({:5}/{:5})  \
             /  chk {:3}% ({:4}/{:4}) {}\r",
            network_pct,
            kbytes,
            stats.received_objects(),
            stats.total_objects(),
            index_pct,
            stats.indexed_objects(),
            stats.total_objects(),
            co_pct,
            state.current,
            state.total,
            state
                .path
                .as_ref()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default()
        )
    }
    io::stdout().flush().unwrap();
}
