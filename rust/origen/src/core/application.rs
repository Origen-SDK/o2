pub mod config;
pub mod target;

use super::application::config::Config;
use crate::core::frontend::{BuildResult, GenericResult};
use crate::utility::str_to_bool;
use crate::utility::version::{set_version_in_toml, Version};
use crate::Result;
use indexmap::IndexMap;
use origen_metal::framework::reference_files;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use origen_metal::Outcome;
use origen_metal::utils::revision_control::Status;
use origen_metal::utils::revision_control::RevisionControl;

/// Represents the current application, an instance of this is returned by
/// origen::app().
#[derive(Debug)]
pub struct Application {
    /// The full file system path to the application root (when applicable)
    pub root: PathBuf,
    config: RwLock<Config>,
}

impl Application {
    pub fn new(root: PathBuf) -> Application {
        log_trace!("Building new Application");
        // Configure origen_metal
        reference_files::set_save_ref_dir(root.join(".origen").join("save_refs"));
        // Now create the app instance
        let config = Config::build(&root, false);
        Application {
            root: root,
            config: RwLock::new(config),
        }
    }

    /// Returns the application version, read from config/version.toml
    pub fn version(&self) -> Result<Version> {
        let version_file = self.root.join("pyproject.toml");
        log_trace!("Reading app version");
        if !version_file.exists() {
            return error!("File does not exist '{}'", version_file.display());
        }
        let content = match fs::read_to_string(&version_file) {
            Ok(x) => x,
            Err(e) => return error!("There was a problem reading the app version file: {}", e),
        };
        lazy_static! {
            static ref VERSION_LINE: Regex =
                Regex::new(r#"(?m)^\s*version\s*=\s*['"](.*)['"]"#).unwrap();
        }
        if VERSION_LINE.is_match(&content) {
            let captures = VERSION_LINE.captures(&content).unwrap();
            Ok(Version::new_pep440(&captures[1])?)
        } else {
            error!(
                "Failed to read a version from file '{}'",
                version_file.display()
            )
        }
    }

    pub fn version_file(&self) -> PathBuf {
        self.root.join("pyproject.toml")
    }

    /// Sets the application version by writing it out to config/version.toml
    /// The normal way to do this is to call app.version(), bump the returned version object
    /// as required, then return it back to this function.
    /// See here for the API - https://docs.rs/semver
    pub fn set_version(&self, version: &Version) -> Result<()> {
        log_info!(
            "Updating version file: '{}'",
            self.version_file().into_os_string().into_string()?
        );
        set_version_in_toml(&self.version_file(), version)
    }

    /// Execute the given function with a reference to the application config.
    pub fn with_config<T, F>(&self, mut func: F) -> Result<T>
    where
        F: FnMut(&Config) -> Result<T>,
    {
        let cfg = self.config.read().unwrap();
        func(&cfg)
    }

    /// Execute the given function with a reference to the application config.
    pub fn with_config_mut<T, F>(&self, mut func: F) -> Result<T>
    where
        F: FnMut(&mut Config) -> Result<T>,
    {
        let mut cfg = self.config.write().unwrap();
        func(&mut cfg)
    }

    pub fn config(&self) -> std::sync::RwLockReadGuard<Config> {
        self.config.read().unwrap()
    }

    /// Returns the application name
    pub fn name(&self) -> String {
        self.with_config(|cfg| Ok(cfg.name.to_string())).unwrap()
    }

    /// Returns a path to the current application's 'app' dir which is the root + app name.
    pub fn app_dir(&self) -> PathBuf {
        self.root.join(self.name())
    }

    /// Return an RevisionControl, containing a driver, based on the app's config
    pub fn rc(&self) -> Result<RevisionControl> {
        Ok(self.with_config(|cfg| match cfg.revision_control.as_ref() {
            Some(rc) => Ok(RevisionControl::from_config(rc)?),
            None => error!("No app RC was given. Cannot create RC driver"),
        })?)
    }

    pub fn rc_init(&self) -> Result<Outcome> {
        crate::with_frontend_app(|app| {
            let rc = app.get_rc()?;
            Ok(rc.init()?)
        })
    }

    pub fn rc_status(&self) -> Result<Status> {
        crate::with_frontend_app(|app| {
            let rc = app.get_rc()?;
            Ok(rc.status()?)
        })
    }

    pub fn rc_checkin(
        &self,
        pathspecs: Option<Vec<&Path>>,
        msg: &str,
        dry_run: bool,
    ) -> Result<Outcome> {
        crate::with_frontend_app(|app| {
            let rc = app.get_rc()?;
            Ok(rc.checkin(pathspecs.clone(), msg, dry_run)?)
        })
    }

    pub fn build_package(&self) -> Result<BuildResult> {
        crate::with_frontend_app(|app| {
            let publisher = app.get_publisher()?;
            log_info!("Building Package...");
            publisher.build_package()
        })
    }

    // Note that the default is to publish the app. But,
    // if not publisher configured, this will likely generate
    // an expected error later on.
    pub fn should_package_app(&self) -> Result<bool> {
        self.with_config(|config| {
            if let Some(pc) = &config.publisher {
                if let Some(s) = pc.get("package_app") {
                    return str_to_bool(s);
                }
            }
            Ok(true)
        })
    }

    pub fn publish(
        &self,
        version: Option<Version>,
        release_title: Option<Option<&str>>,
        release_note: Option<&str>,
        dry_run: bool,
    ) -> Result<GenericResult> {
        Ok(crate::with_frontend_app(|app| {
            // log_info!("Performing pre-publish checks...");
            // app.check_production_status()?;

            let v = Version::new_pep440(&self.version()?.to_string())?;
            let new_v = match version.as_ref() {
                Some(ver) => ver.clone(),
                None => v.update_dialogue()?,
            };

            println!("Updating version from {} to {}", v, new_v);
            if dry_run {
                log_info!("(Dry run - not updating version file)");
            } else {
                self.set_version(&new_v)?;
            }
            let mut files = vec![self.version_file()];

            let rs = app.get_release_scribe()?;
            files.append(&mut rs.publish(&new_v, release_title, release_note, dry_run)?);

            if dry_run {
                println!("(Dry run - not checking in any files. Would check in:");
                for f in files.iter() {
                    println!("\t{}", f.display());
                }
            } else {
                let rc = app.get_rc()?;
                rc.checkin(
                    Some(files.iter().map(|f| f.as_path()).collect()),
                    "Recorded new version in the version tracker",
                    false,
                )?;
                rc.tag(&new_v.to_string(), false, None)?;
            }

            if self.should_package_app()? {
                let publisher = app.get_publisher()?;
                log_info!("Building Package...");
                let package_result = publisher.build_package()?;
                if package_result.succeeded {
                    if let Some(m) = &package_result.message {
                        log_info!("{}", m);
                    }
                } else {
                    if let Some(m) = &package_result.message {
                        log_error!("Failed to build package: {}", m);
                        return error!("Failed to build package: {}", m);
                    } else {
                        log_error!("Failed to build package");
                        return error!("Failed to build package");
                    }
                }

                log_info!("Uploading Package...");
                let publish_result = publisher.upload(&package_result, dry_run)?;
                if publish_result.succeeded {
                    if let Some(m) = publish_result.message {
                        log_info!("{}", m);
                    } else {
                        log_info!("Package published successfully!");
                    }
                } else {
                    if let Some(m) = publish_result.message {
                        log_error!("{}", m);
                        return error!("{}", m);
                    } else {
                        log_error!("Failed to upload package!");
                        return error!("Failed to upload package!");
                    }
                }
            } else {
                log_trace!("Bypassing building and uploading app");
            }

            // Mailer here is 'optional', in that we won't hold up the
            // release because of it, but will throw out lots of red text
            // if not explicitly marked as okay to skip
            // if let Some(m) = app.mailer()? {
            //     // let mailer = app.get_mailer_or_none()?;
            //     // mailer.send("...")?;
            // } else {
            //     log_error!("No mailer available - no release email sent");
            // }

            // let website = app.get_website()?;
            // website.publish()?;

            //     Ok(())
            // })?)
            let mut r = GenericResult::new_success();
            r.set_msg("Successfully released application!");
            Ok(r)
        })?)
    }

    pub fn run_publish_checks(&self, stop_at_first_fail: bool) -> Result<ProductionStatus> {
        self.check_production_status(stop_at_first_fail)
    }

    pub fn check_production_status(&self, stop_at_first_fail: bool) -> Result<ProductionStatus> {
        log_info!("Checking production status...");
        let mut stat = ProductionStatus::default();

        crate::with_frontend_app(|app| {
            // log_info!("Running any application-defined checks: pre-Origen checks...");
            // stat.push_checks(app.production_status_checks_pre(stop_at_first_fail)?);

            // Check for modified files
            log_info!("Checking for modified files...");
            let s = app.get_rc()?.status()?;
            stat.push_clean_work_space_check(!s.is_modified(), None);

            if stat.failed() && stop_at_first_fail {
                return Ok(());
            }

            log_info!("Running unit tests...");
            let s = app.get_unit_tester()?.run()?;
            stat.push_unit_test_check(s.passed(), s.text);

            // log_info!("Checking for local dependencies...");
            // let s = app.list_local_dependencies()?;
            // stat.push_local_deps_check(s.empty?(), )

            // TODOs:
            // log_info!("Checking for local dependencies...");
            // log_info!("Checking for lint errors...");
            // log_info!("Ensuring the package builds...")
            // log_info!("Ensuring the website builds...")
            // log_info!("Running any application-defined checks: post-Origen checks...");
            // stat.push_checks(app.production_status_checks_post(stop_at_first_fail)?);

            Ok(())
        })?;
        Ok(stat)
    }

    /// Resolves a directory/file path relative to the application's root.
    /// Accepts an optional 'user_val' and a default. The resulting directory will be resolved from:
    /// 1. If a user value is given, and its absolute, this is the final path.
    /// 2. If a user value is given but its not absolute, then the final path is the user path relative to the application root.
    /// 3. If no user value is given, the final path is the default path relative to the root.
    /// Notes:
    ///   A default is required, but an empty default will point to the application root.
    ///   The default is assumed to be relative. Absolute defaults are not supported.
    pub fn resolve_path(&self, user_val: Option<&String>, default: &str) -> PathBuf {
        let offset;
        if let Some(user_str) = user_val {
            if Path::new(&user_str).is_absolute() {
                return PathBuf::from(user_str);
            } else {
                offset = user_str.to_string();
            }
        } else {
            offset = default.to_string();
        }
        let mut dir = self.root.clone();
        dir.push(offset);
        dir
    }

    /// Don't use this unless you know what you're doing, use origen::STATUS::output_dir() instead, since
    /// that accounts for the output directory being overridden by the current command
    pub fn _output_directory(&self) -> PathBuf {
        self.with_config(|config| Ok(self.resolve_path(config.output_directory.as_ref(), "output")))
            .unwrap()
    }

    /// Don't use this unless you know what you're doing, use origen::STATUS::reference_dir() instead, since
    /// that accounts for the reference directory being overridden by the current command
    pub fn _reference_directory(&self) -> PathBuf {
        self.with_config(
            |config| Ok(self.resolve_path(config.reference_directory.as_ref(), ".ref")),
        )
        .unwrap()
    }

    pub fn website_output_directory(&self) -> PathBuf {
        self.with_config(|config| {
            Ok(self.resolve_path(config.website_output_directory.as_ref(), "output/web"))
        })
        .unwrap()
    }

    pub fn website_source_directory(&self) -> PathBuf {
        self.with_config(|config| {
            Ok(self.resolve_path(config.website_source_directory.as_ref(), "web/source"))
        })
        .unwrap()
    }
}

pub struct ProductionStatus {
    checks: IndexMap<String, (bool, Option<String>, Option<String>)>,
    passed: bool,
}

impl Default for ProductionStatus {
    fn default() -> Self {
        Self {
            checks: IndexMap::new(),
            passed: true,
        }
    }
}

impl ProductionStatus {
    pub fn push_check(
        &mut self,
        check: &str,
        desc: Option<String>,
        result: bool,
        result_text: Option<String>,
    ) {
        self.checks
            .insert(check.to_string(), (result, desc, result_text));
        if !result {
            self.passed = false;
        }
    }

    pub fn push_clean_work_space_check(&mut self, clean: bool, txt: Option<String>) {
        self.push_check(
            "Clean Workspace",
            Some("Fails if there are modified or untracked, but not ignored, files in the workspace.".to_string()),
            clean,
            txt
        );
    }

    pub fn push_unit_test_check(&mut self, passed: bool, txt: Option<String>) {
        self.push_check(
            "Unit Tests",
            Some("Fails if any of the unit tests fail".to_string()),
            passed,
            txt,
        );
    }

    pub fn passed(&self) -> bool {
        self.passed
    }

    pub fn failed(&self) -> bool {
        !self.passed
    }

    pub fn summarize_and_exit(&self) {
        if self.passed() {
            exit_pass!();
        } else {
            for (n, check) in self.checks.iter() {
                if check.0 {
                    display_greenln!("{}... PASSED!", n);
                } else {
                    if let Some(m) = check.2.as_ref() {
                        display_redln!("{}... FAILED with message: {}", n, m);
                    } else {
                        display_redln!("{}... FAILED", n);
                    }
                }
            }
            exit_fail!();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::application::Application;
    use crate::utility::version::Version;
    use crate::STATUS;

    #[test]
    fn reading_and_writing_version() {
        let app_root = STATUS.origen_wksp_root.join("test_apps").join("python_app");
        let app = Application::new(app_root);

        let v = Version::new_pep440("2.21.5-dev7").unwrap();
        let _res = app.set_version(&v);
        assert_eq!(app.version().unwrap(), v);

        let v = Version::new_pep440("1.2.3").unwrap();
        let _res = app.set_version(&v);
        assert_eq!(app.version().unwrap(), v);
    }
}
