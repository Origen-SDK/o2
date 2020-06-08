use super::{Credentials, RevisionControlAPI, Status};
use crate::utility::command_helpers::log_stdout_and_stderr;
use crate::Result as OrigenResult;
use crate::USER;
use git2::Repository;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use git2::build::{CheckoutBuilder, RepoBuilder};
use git2::{Cred, CredentialType, FetchOptions, RemoteCallbacks};
use std::cell::RefCell;

enum VersionType {
    Branch,
    Tag,
    Commit,
    Head,
    Unknown,
}

/// Attempts to get the given attribute from the Git config.
/// Not sure if it's possible to do this via libgit2, so this currently
/// uses regular Git unlike most of this driver.
/// If Git is not available, or any other issue, then it will silently
/// fail and simply return None.
/// e.g. call git::config("email") to get the user's email address.
pub fn config(attr_name: &str) -> Option<String> {
    let process = Command::new("git")
        .args(&["config", &format!("user.{}", attr_name)])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    let mut result: Option<String> = None;

    if let Ok(mut p) = process {
        log_stdout_and_stderr(
            &mut p,
            Some(&mut |line: &str| {
                if result.is_none() {
                    result = Some(line.trim().to_string());
                }
            }),
            Some(&mut |_line: &str| {
                // Just swallow stderr
            }),
        );
        let r = p.wait();
        if r.is_ok() {
            if r.unwrap().success() {
                result
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}

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
    fn populate(&self, version: &str) -> OrigenResult<()> {
        log_info!("Populating '{}' from '{}'", &self.local.display(), &self.remote);
        self.reset_temps();
        let mut cb = RemoteCallbacks::new();
        cb.transfer_progress(|stats| self.transfer_progress_callback(&stats));
        cb.credentials(|url, username_from_url, allowed_types| {
            self.credentials_callback(url, username_from_url, allowed_types)
        });
        let mut fo = FetchOptions::new();
        fo.remote_callbacks(cb);
        match RepoBuilder::new()
            .fetch_options(fo)
            .clone(&self.remote, &self.local)
        {
            Ok(_) => {
                self.password_was_good();
            }
            Err(e) => {
                return Err(e.into());
            }
        }

        self.checkout(true, None, version)?;

        Ok(())
    }

    fn revert(&self, path: Option<&Path>) -> OrigenResult<()> {
        let _ = self._checkout(true, path, "HEAD", false)?;
        Ok(())
    }

    fn checkout(&self, force: bool, path: Option<&Path>, version: &str) -> OrigenResult<bool> {
        self._checkout(force, path, version, true)
    }

    /// Returns an Origen RC status, which does not go into as much detail as a full Git status,
    /// mainly that there is no differentiation between changes that are staged vs unstaged.
    /// It also doesn't bother to track renamed files, simply reporting them as a deleted file
    /// and an added file.
    fn status(&self, _path: Option<&Path>) -> OrigenResult<Status> {
        let mut status = Status::default();
        log_trace!("Checking status of '{}'", &self.local.display());
        let repo = Repository::open(&self.local)?;
        let stat = repo.statuses(None)?;
        for entry in stat.iter() {
            //dbg!(entry.status());

            if entry.status().contains(git2::Status::WT_NEW) {
                let old = entry.index_to_workdir().unwrap().old_file().path();
                let new = entry.index_to_workdir().unwrap().new_file().path();
                status.added.push(self.local.join(old.or(new).unwrap()));
                continue;
            }

            if entry.status().contains(git2::Status::INDEX_NEW) {
                let old = entry.head_to_index().unwrap().old_file().path();
                let new = entry.head_to_index().unwrap().new_file().path();
                status.added.push(self.local.join(old.or(new).unwrap()));
                continue;
            }

            if entry.status().contains(git2::Status::WT_DELETED) {
                let old = entry.index_to_workdir().unwrap().old_file().path();
                let new = entry.index_to_workdir().unwrap().new_file().path();
                status.removed.push(self.local.join(old.or(new).unwrap()));
                continue;
            }

            if entry.status().contains(git2::Status::INDEX_DELETED) {
                let old = entry.head_to_index().unwrap().old_file().path();
                let new = entry.head_to_index().unwrap().new_file().path();
                status.removed.push(self.local.join(old.or(new).unwrap()));
                continue;
            }

            if entry.status().contains(git2::Status::WT_MODIFIED)
                || entry.status().contains(git2::Status::WT_TYPECHANGE)
            {
                let old = entry.index_to_workdir().unwrap().old_file().path();
                let new = entry.index_to_workdir().unwrap().new_file().path();
                status.changed.push(self.local.join(old.or(new).unwrap()));
                continue;
            }

            if entry.status().contains(git2::Status::INDEX_MODIFIED)
                || entry.status().contains(git2::Status::INDEX_TYPECHANGE)
            {
                let old = entry.head_to_index().unwrap().old_file().path();
                let new = entry.head_to_index().unwrap().new_file().path();
                status.changed.push(self.local.join(old.or(new).unwrap()));
                continue;
            }
            if entry.status().contains(git2::Status::CONFLICTED) {
                let old = entry.head_to_index().unwrap().old_file().path();
                let new = entry.head_to_index().unwrap().new_file().path();
                status
                    .conflicted
                    .push(self.local.join(old.or(new).unwrap()));
                continue;
            }
        }
        Ok(status)
    }

    fn tag(&self, tagname: &str, force: bool, message: Option<&str>) -> OrigenResult<()> {
        log_trace!("Tagging '{}' with '{}'", &self.local.display(), tagname);
        let repo = Repository::open(&self.local)?;
        let target = "HEAD";
        let obj = repo.revparse_single(target)?;

        if let Some(ref msg) = message {
            let sig = Git::signature(&repo)?;
            repo.tag(tagname, &obj, &sig, msg, force)?;
        } else {
            repo.tag_lightweight(tagname, &obj, force)?;
        }
        Ok(())
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

    // Returns Ok(true) if merge conflicts are encountered when force = false.
    fn _checkout(
        &self,
        force: bool,
        path: Option<&Path>,
        version: &str,
        prefetch: bool,
    ) -> OrigenResult<bool> {
        log_info!(
            "Checking out version '{}' in '{}'",
            version,
            &self.local.display()
        );
        self.reset_temps();

        if prefetch {
            // Make sure we have all the latest tags/commits/branches available locally
            self.fetch(None)?;
        }

        // Now the first part of this operation is to find the SHA (Oid) corresponding to the given version
        // reference, and identify whether it is a reference to branch, tag or commit

        let mut repo = Repository::open(&self.local)?;
        let mut oid: Option<git2::Oid> = None;
        let mut version_type = VersionType::Unknown;
        let mut stash_pop_required = false;

        // Means (re)checkout the current version, used for blowing away local edits
        if version == "HEAD" || version == "head" || version == "Head" {
            version_type = VersionType::Head;
            let head = repo.head().unwrap();
            oid = Some(head.target().unwrap());

        // If the version reference is a branch...
        } else if let Ok(branch) =
            repo.find_branch(&format!("origin/{}", version), git2::BranchType::Remote)
        {
            //if self.remote_branch_exists(&repo, version) {
            log_debug!("Checking out branch '{}'", version);
            let head = branch.get();
            // This gets a reference to the tip of the remote branch in question
            oid = Some(head.target().unwrap());
            version_type = VersionType::Branch;

        // If the version reference is a tag...
        } else if self.tag_exists_locally(&repo, version) {
            log_debug!("Checking out tag '{}'", version);
            let references = repo.references()?;
            for reference in references {
                if let Ok(r) = reference {
                    if r.is_tag() {
                        if let Some(name) = r.name() {
                            if name.ends_with(&format!("tags/{}", version)) {
                                oid = Some(r.target().unwrap());
                                version_type = VersionType::Tag;
                            }
                        }
                    }
                }
            }
            if oid.is_none() {
                return error!(
                    "Something went wrong, the commit for tag '{}' was not found",
                    version
                );
            }
        // If the version reference is a commit SHA...
        } else if let Ok(oid_) = git2::Oid::from_str(version) {
            log_debug!("Checking out commit '{}'", version);
            oid = Some(oid_);
            version_type = VersionType::Commit;
        }

        // If a SHA and reference type was successfully found, now do the checkout

        if let Some(oid) = oid {
            let mut co = CheckoutBuilder::new();
            // This is called for every file, cur is the current file number, and total
            // is the total number of files that are being checked out
            co.progress(|path, _cur, _total| {
                if let Some(p) = path {
                    log_debug!("{}  : Success", p.display());
                }
            });
            // We always force, which means make the local view look exactly like the target version
            co.force();
            if force {
                // Also remove new files when forcing
                co.remove_untracked(true);
            } else if self.status(None)?.is_modified() {
                // If we want to preserve local edits then we will stash them and merge afterwards
                let signature = repo
                    .signature()
                    .unwrap_or(git2::Signature::now("Origen", "noreply@origen-sdk.org")?);
                repo.stash_save(
                    &signature,
                    "Preserving during checkout",
                    Some(git2::StashFlags::INCLUDE_UNTRACKED),
                )?;
                stash_pop_required = true;
            }
            if let Some(p) = path {
                co.path(p);
            }

            match version_type {
                VersionType::Branch => {
                    // This checkout makes sure that we have the target commit on disk in the workspace
                    let commit = repo.find_commit(oid)?;
                    repo.checkout_tree(commit.as_object(), Some(&mut co))?;
                    // This forces the target of the local version of the requested branch to the target commit,
                    // creating the local branch if it doesn't already exist
                    repo.set_head_detached(oid)?; // Need to do this otherwise the force can fail if the
                                                  // branch is the current HEAD
                    repo.branch(version, &commit, true)?;
                    // Finally, this switches the current workspace branch to the requested branch
                    let head = format!("refs/heads/{}", version);
                    repo.set_head(&head)?;
                }
                VersionType::Head => {
                    let object = repo.find_object(oid, None)?;
                    repo.checkout_tree(&object, Some(&mut co))?;
                }
                VersionType::Tag => {
                    let object = repo.find_object(oid, None)?;
                    repo.checkout_tree(&object, Some(&mut co))?;
                    // May want to use set_head_detached_from_annotated in future (not available in current version),
                    // as it may give a better indication of sitting at a tag when 'git status' is run
                    repo.set_head_detached(oid)?;
                }
                VersionType::Commit => {
                    let error = match repo.find_commit(oid) {
                        Ok(commit) => {
                            repo.checkout_tree(commit.as_object(), Some(&mut co))?;
                            repo.set_head_detached(oid)?;
                            None
                        }
                        Err(_e) => {
                            Some(error!("No matching commit found for version '{}'", version))
                        }
                    };
                    if let Some(e) = error {
                        if stash_pop_required {
                            repo.stash_pop(0, None)?;
                        }
                        return e;
                    }
                }
                _ => unreachable!(),
            }
        } else {
            if stash_pop_required {
                repo.stash_pop(0, None)?;
            }
            return error!(
                "Could not resolve version '{}' to a commit, tag or branch reference",
                version
            );
        }

        if stash_pop_required {
            repo.stash_pop(0, None)?;
        }
        Ok(!self.status(None)?.conflicted.is_empty())
    }

    // Returns a signature (to attribute commits etc.) for the current user, falling
    // back to a default Origen signature if necessary
    fn signature(repo: &Repository) -> OrigenResult<git2::Signature> {
        Ok(repo
            .signature()
            .unwrap_or(git2::Signature::now("Origen", "noreply@origen-sdk.org")?))
    }

    fn password_was_good(&self) {
        self.last_password_attempt.replace(None);
    }

    fn credentials_callback(
        &self,
        url: &str,
        username: Option<&str>,
        allowed_types: CredentialType,
    ) -> Result<Cred, git2::Error> {
        if allowed_types.contains(CredentialType::SSH_KEY) {
            let mut ssh_attempts = self.ssh_attempts.borrow_mut();
            let ssh_keys = ssh_keys();
            if *ssh_attempts < ssh_keys.len() {
                //let key = Cred::ssh_key_from_agent(username.unwrap());
                //if key.is_ok() {
                //    return key;
                //}
                let key = Cred::ssh_key(username.unwrap(), None, &ssh_keys[*ssh_attempts], None);
                *ssh_attempts += 1;
                return key;
            }
        }
        if allowed_types.contains(CredentialType::USER_PASS_PLAINTEXT) {
            let username;
            let password;
            {
                let last_password_attempt = self.last_password_attempt.borrow();
                password = {
                    if self.credentials.is_some()
                        && self.credentials.as_ref().unwrap().password.is_some()
                    {
                        self.credentials
                            .as_ref()
                            .unwrap()
                            .password
                            .as_ref()
                            .unwrap()
                            .clone()
                    } else {
                        USER.password(
                            Some(&format!("to access repository '{}'", url)),
                            last_password_attempt.as_deref(),
                        )
                        .expect("Couldn't prompt for password")
                    }
                };
                username = {
                    if self.credentials.is_some()
                        && self.credentials.as_ref().unwrap().username.is_some()
                    {
                        self.credentials
                            .as_ref()
                            .unwrap()
                            .username
                            .as_ref()
                            .unwrap()
                            .clone()
                    } else {
                        USER.id().unwrap()
                    }
                };
            }
            self.last_password_attempt
                .replace(Some(password.to_string()));
            return Ok(Cred::userpass_plaintext(&username, &password)?);
        }

        // We tried our best
        log_warning!(
            "Unhandled Git credential type requested '{:?}'",
            allowed_types
        );
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
            if tags
                .iter()
                .any(|topt| if let Some(t) = topt { t == tag } else { false })
            {
                return true;
            }
        }
        false
    }

    /// Returns true if the given branch name exists in the remote repo
    pub fn remote_branch_exists(&self, repo: &Repository, name: &str) -> bool {
        let branches = repo.branches(Some(git2::BranchType::Remote)).unwrap();
        for branch in branches {
            if let Ok((branch, _branch_type)) = branch {
                if let Ok(Some(branch_name)) = branch.name() {
                    if branch_name == format!("origin/{}", name) {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Equivalent to calling 'git fetch' within a repo, this will download all available remote
    /// branches, tags and commits
    pub fn fetch(&self, remote_name: Option<&str>) -> OrigenResult<()> {
        let repo = Repository::open(&self.local)?;
        let remote = remote_name.unwrap_or("origin");

        // Figure out whether it's a named remote or a URL
        log_debug!(
            "Fetching '{}' for Git repo in '{}'",
            remote,
            self.local.display()
        );
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

        cb.transfer_progress(|stats| self.transfer_progress_callback(&stats));

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

        log_debug!("Fetch completed successfully");

        self.password_was_good();

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
        paths
            .filter_map(|p| p.ok())
            .map(|p| p.path())
            .for_each(|path| match path.extension() {
                None => {}
                Some(x) => {
                    if x == "pub" {
                        let key = path.parent().unwrap().join(path.file_stem().unwrap());
                        if key.exists() {
                            keys.push(key);
                        }
                    }
                }
            });
    } else {
        log_warning!("Could not find the $HOME/.ssh directory to obtain ssh keys");
    }
    keys
}
