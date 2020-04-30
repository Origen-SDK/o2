use super::{Credentials, Progress, RevisionControlAPI};
use crate::Result as OrigenResult;
use crate::USER;
use git2::Repository;
use std::{env,fs};
use std::path::{Path, PathBuf};

use git2::build::{CheckoutBuilder, RepoBuilder};
use git2::{Cred, FetchOptions, RemoteCallbacks, CredentialType};
use std::cell::RefCell;

pub struct Git {
    /// Path to the local directory for the repository
    pub local: PathBuf,
    /// Link to the remote repository
    pub remote: String,
    credentials: Option<Credentials>,
    // There doesn't seem to be anything like an 'on_credentials_failed' callback, so have to keep track of
    // what password was attempted last so that it can be used to blow the caching of a bad password
    last_password_attempt: RefCell<Option<String>>,
    // These are used to keep track of transfer progress during clone and fetch operations
    network_pct: RefCell<usize>,
    index_pct: RefCell<usize>,
    deltas_pct: RefCell<usize>,
    ssh_attempts: RefCell<usize>,
}


impl RevisionControlAPI for Git {
    fn populate(
        &self,
        version: &str,
        mut callback: Option<&mut dyn FnMut(&Progress)>,
    ) -> OrigenResult<Progress> {
        log_info!("Populating {}", &self.local.display());
        self.reset_temps();
        let mut cb = RemoteCallbacks::new();
        cb.transfer_progress(|stats| {
            self.transfer_progress_callback(&stats)
        });
        cb.credentials(|url, username_from_url, allowed_types| {
            self.credentials_callback(url, username_from_url, allowed_types)
        });
        let mut fo = FetchOptions::new();
        fo.remote_callbacks(cb);
        match RepoBuilder::new()
            .fetch_options(fo)
            .clone(&self.remote, &self.local) {
            Ok(_) => {},
            Err(e) => {
                log_error!("{}", e);
                return Err(e.into());
            }
        }

        self.checkout(true, None, version, None)?;

        Ok(Progress::default())
    }

    // This was inspired by:
    // https://stackoverflow.com/questions/55141013/how-to-get-the-behaviour-of-git-checkout-in-rust-git2
    fn checkout(&self, force: bool, path: Option<&Path>, version: &str,
        callback: Option<&mut dyn FnMut(&Progress)>,
    ) -> OrigenResult<Progress> {
        let repo = Repository::open(&self.local)?;
        self.reset_temps();

        let mut commit: Option<git2::Commit> = None;

        // Make sure we have all the latest tags/commits/branches available locally
        self.fetch(None)?;

        if let Ok(_branch) = repo.find_branch(version, git2::BranchType::Local) {
            log_debug!("Checking out branch '{}'", version);
            let head = repo.head()?;
            let oid = head.target().unwrap();
            commit = Some(repo.find_commit(oid)?);

        } else if self.tag_exists_locally(&repo, version) {
            log_debug!("Checking out tag '{}'", version);
            let references = repo.references()?;
            for reference in references {
                if let Ok(r) = reference {
                    if r.is_tag() {
                        if let Some(name) = r.name() {
                            if name.ends_with(&format!("tags/{}", version)) {
                                let oid = r.target().unwrap();
                                commit = Some(repo.find_commit(oid)?);
                            }
                        }
                    }
                }
            }
            // TODO: Should check that a commit was found here

        } else if let Ok(oid) = git2::Oid::from_str(version) {
            log_debug!("Checking out commit '{}'", version);
            match repo.find_commit(oid) {
                Ok(c) => commit = Some(c),
                // TODO: generate a meaningful error
                Err(_e) => {},
            }
        }

        if let Some(commit) = commit {
            let mut co = CheckoutBuilder::new();
            // This is called for every file, cur is the current file number, and total
            // is the total number of files that are being checked out
            co.progress(|path, _cur, _total| {
                if let Some(p) = path {
                    log_debug!("{}  : Success", p.display());
                }
            });
            if force {
                co.force();
            }
            if let Some(p) = path {
                co.path(p);
            }

            let branch = repo.branch(version, &commit, false);
            let obj = repo.revparse_single(&format!("refs/heads/{}", version)).unwrap();
            repo.checkout_tree(
                &obj,
                Some(&mut co)
            );
            let head = format!("refs/heads/{}", version);
            log_debug!("Setting head to '{}'", &head);
            repo.set_head(&head)?;
        } else {
        }

        // TODO: Do a pull here if the version is a branch

        Ok(Progress::default())
    }
}


impl Git {
    pub fn new(local: &Path, remote: &str, credentials: Option<Credentials>) -> Git {
        Git {
            local: local.to_path_buf(),
            remote: remote.to_string(),
            credentials: credentials,
            last_password_attempt: RefCell::new(None),
            network_pct: RefCell::new(0),
            index_pct: RefCell::new(0),
            deltas_pct: RefCell::new(0),
            ssh_attempts: RefCell::new(0),
        }
    }

    fn credentials_callback(&self, url: &str, username: Option<&str>, allowed_types: CredentialType) -> Result<Cred, git2::Error> {
        if allowed_types.contains(CredentialType::SSH_KEY) {
            let mut ssh_attempts = self.ssh_attempts.borrow_mut();
            let ssh_keys = ssh_keys();
            if *ssh_attempts < ssh_keys.len() {
                //let key = Cred::ssh_key_from_agent(username.unwrap());
                //if key.is_ok() {
                //    return key;
                //}
                let key = Cred::ssh_key(
                    username.unwrap(),
                  None,
                  &ssh_keys[*ssh_attempts],
                  None,
                )
                *ssh_attempts += 1;
                return key;
            }
        }
        if allowed_types.contains(CredentialType::USER_PASS_PLAINTEXT) {
            let last_password_attempt = self.last_password_attempt.borrow();
            let password = {
                if self.credentials.is_some() && self.credentials.as_ref().unwrap().password.is_some() {
                    self.credentials.as_ref().unwrap().password.as_ref().unwrap().clone()
                } else {
                    USER.password(
                        Some(&format!("to access repository '{}'", url)),
                        last_password_attempt.as_deref(),
                    )
                    .expect("Couldn't prompt for password")
                }
            };
            let username = {
                if self.credentials.is_some() && self.credentials.as_ref().unwrap().username.is_some() {
                    self.credentials.as_ref().unwrap().username.as_ref().unwrap().clone()
                } else {
                    USER.id().unwrap()
                }
            };
            self.last_password_attempt.replace(Some(password.to_string()));
            return Ok(Cred::userpass_plaintext(&username, &password)?);
        }

        // We tried our best
        log_warning!("Unhandled Git credential type requested '{:?}'",  allowed_types);
        Err(git2::Error::from_str("no authentication available"))
    }

    fn reset_temps(&self) {
        let mut network_pct = self.network_pct.borrow_mut();
        let mut index_pct = self.index_pct.borrow_mut();
        let mut deltas_pct = self.deltas_pct.borrow_mut();
        let mut ssh_attempts = self.ssh_attempts.borrow_mut();
        *network_pct = 0;
        *index_pct = 0;
        *deltas_pct = 0;
        *ssh_attempts = 0;
    }

    /// This will output a debug log entry every 5% of progress
    fn transfer_progress_callback(&self, stats: &git2::Progress) -> bool {
        let mut network_pct = self.network_pct.borrow_mut();
        let mut index_pct = self.index_pct.borrow_mut();
        if *network_pct != 100 && *index_pct != 100 {
            let new_network_pct = (100 * stats.received_objects()) / stats.total_objects();
            let new_index_pct = (100 * stats.indexed_objects()) / stats.total_objects();
            if new_network_pct / 5 != *network_pct / 5 || new_index_pct / 5 != *index_pct / 5 {
                *network_pct = new_network_pct;
                *index_pct = new_index_pct;
                log_debug!("Received {:3}% : Indexed {:3}%", *network_pct, *index_pct);
            }
        } else {
            if stats.total_deltas() != 0 {
                let mut deltas_pct = self.deltas_pct.borrow_mut();
                let new_deltas_pct = (100 * stats.indexed_deltas()) / stats.total_deltas();
                if new_deltas_pct / 5 != *deltas_pct / 5 {
                    *deltas_pct = new_deltas_pct;
                    log_debug!("Resolving deltas {:3}%", *deltas_pct);
                }
            }
        }
        true
    }

    /// Returns true if the given tag name exists in the local repo
    pub fn tag_exists_locally(&self, repo: &Repository, tag: &str) -> bool {
        if let Ok(tags) = repo.tag_names(None) {
            if tags.iter().any(|topt| {
                if let Some(t) = topt {
                    t == tag
                } else {
                    false
                }
            }) {
                return true;
            }
        }
        false
    }

    /// Equivalent to calling 'git fetch' within a repo
    pub fn fetch(&self, remote_name: Option<&str>) -> OrigenResult<()> {
        let repo = Repository::open(&self.local)?;
        let remote = remote_name.unwrap_or("origin");

        // Figure out whether it's a named remote or a URL
        log_debug!("Fetching '{}' for Git repo in '{}'", remote, self.local.display());
        let mut cb = RemoteCallbacks::new();
        let mut remote = repo
            .find_remote(remote)
            .or_else(|_| repo.remote_anonymous(remote))?;
        cb.sideband_progress(|data| {
            log_trace!("remote: {}", std::str::from_utf8(data).unwrap());
            true
        });

        cb.credentials(|url, username_from_url, allowed_types| {
            self.credentials_callback(url, username_from_url, allowed_types)
        });

        // This callback gets called for each remote-tracking branch that gets
        // updated. The message we output depends on whether it's a new one or an
        // update.
        cb.update_tips(|refname, a, b| {
            if a.is_zero() {
                log_debug!("[new]     {:20} {}", b, refname);
            } else {
                log_debug!("[updated] {:10}..{:10} {}", a, b, refname);
            }
            true
        });

        cb.transfer_progress(|stats| {
            self.transfer_progress_callback(&stats)
        });

        // Download the packfile and index it. This function updates the amount of
        // received data and the indexer stats which lets you inform the user about
        // progress.
        let mut fo = FetchOptions::new();
        fo.remote_callbacks(cb);
        fo.download_tags(git2::AutotagOption::All);
        remote.download(&[] as &[&str], Some(&mut fo))?;

        // Disconnect the underlying connection to prevent from idling.
        remote.disconnect();

        // Update the references in the remote's namespace to point to the right
        // commits. This may be needed even if there was no packfile to download,
        // which can happen e.g. when the branches have been changed but all the
        // needed objects are available locally.
        remote.update_tips(None, true, git2::AutotagOption::Unspecified, None)?;

        log_debug!("Fetch completed");

        Ok(())
    }
}

fn ssh_keys() -> Vec<PathBuf> {
    let home = match dirs::home_dir() {
        Some(x) => x,
        None => {
            log_warning!("Could not determine the HOME directory to find ssh keys");
            return vec![];
        }
    };
    let mut keys: Vec<PathBuf> = vec![];
    let dir = home.join(".ssh");
    if dir.exists() {
        let paths = fs::read_dir(dir).unwrap();
        paths.filter_map(|p| p.ok())
            .map(|p| p.path())
            .for_each(|path| {
                match path.extension() {
                    None => {},
                    Some(x) => {
                        if x == "pub" {
                            let key = path.parent().unwrap().join(path.file_stem().unwrap());
                            if key.exists() {
                                keys.push(key);
                            }
                        }
                    }
                }
            });
    } else {
        log_warning!("Could not find the $HOME/.ssh directory to obtain ssh keys");
    }
    keys
}
