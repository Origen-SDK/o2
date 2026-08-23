use super::_prelude::*;
use origen::utility::github::{dispatch_workflow, get_latest_workflow_dispatch, WorkflowRun};
use origen::utility::release_scribe::{ReleaseProduct, ReleaseScribe};
use origen_metal::dialoguer::{Confirm, Input, Select};
use origen_metal::utils::crates_io::is_crate_version_available;
use origen_metal::utils::pypi::is_package_version_available;
use origen_metal::utils::revision_control::supported::git;
use origen_metal::utils::revision_control::RevisionControlAPI;
use origen_metal::utils::version::{ReleaseType, Version, VersionWithTOML};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

pub const BASE_CMD: &'static str = "rc";
const PRODUCTS: [&str; 2] = ["origen", "origen-metal"];
const RELEASE_TYPES: [&str; 6] = [
    "development",
    "patch",
    "minor",
    "major",
    "production",
    "current",
];

gen_core_cmd_funcs__no_exts__no_app_opts!(
    BASE_CMD,
    "Revision-control and release operations",
    { |cmd: App| cmd.arg_required_else_help(true) },
    core_subcmd__no_exts__no_app_opts!("tag", "Prepare and release Origen products", {
        |cmd: App| {
            cmd.arg(
                Arg::new("product")
                    .long("product")
                    .short('p')
                    .value_parser(PRODUCTS)
                    .action(clap::ArgAction::Append)
                    .help("Product to release; repeat to release both products"),
            )
            .arg(Arg::new("type").long("type").short('t').value_parser(RELEASE_TYPES).action(SetArg).conflicts_with_all(&["origen-type", "metal-type"]).help("Release type for a single selected product"))
            .arg(Arg::new("origen-type").long("origen-type").value_parser(RELEASE_TYPES).action(SetArg).help("Origen release type for a combined release"))
            .arg(Arg::new("metal-type").long("metal-type").value_parser(RELEASE_TYPES).action(SetArg).help("Origen Metal release type for a combined release"))
            .arg(Arg::new("note").long("note").short('n').action(SetArg).help("Release-note text"))
            .arg(Arg::new("file").long("file").short('f').action(SetArg).conflicts_with("note").help("Read the release note from this file"))
            .arg(Arg::new("author").long("author").action(SetArg).help("Override the configured Git release author"))
            .arg(Arg::new("dry-run").long("dry-run").action(SetArgTrue).help("Validate and display the release plan without changing files or external state"))
            .arg(Arg::new("local").long("local").action(SetArgTrue).conflicts_with("dry-run").help("Prepare versions and histories locally without commit, tag, push, publication, or deployment"))
            .arg(Arg::new("non-interactive").long("non-interactive").action(SetArgTrue).help("Fail instead of prompting when required release inputs are absent"))
            .arg(Arg::new("yes").long("yes").short('y').action(SetArgTrue).help("Accept the final release plan"))
            .arg(Arg::new("resume").long("resume").action(SetArg).value_name("RELEASE_ID").help("Resume a prepared release by its displayed release ID"))
            .arg(Arg::new("allow-local-changes").long("allow-local-changes").action(SetArgTrue).help("Allow a dirty workspace for local or dry-run simulation only"))
        }
    })
);

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
enum Product {
    Origen,
    Metal,
}

impl Product {
    fn parse(value: &str) -> Self {
        match value {
            "origen" => Self::Origen,
            "origen-metal" => Self::Metal,
            _ => unreachable!(),
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Origen => "Origen",
            Self::Metal => "Origen Metal",
        }
    }

    fn release_product(&self) -> ReleaseProduct {
        match self {
            Self::Origen => ReleaseProduct::Origen,
            Self::Metal => ReleaseProduct::OrigenMetal,
        }
    }
}

struct PlannedProduct {
    product: Product,
    version: VersionWithTOML,
    cargo_versions: Vec<VersionWithTOML>,
    release_type: String,
    workflow: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
struct PublishRequest {
    product: Product,
    tag: String,
    version: String,
    workflow: Option<String>,
}

trait ReleaseEffects {
    fn validate(&mut self) -> Result<()>;
    fn commit(&mut self) -> Result<()>;
    fn tag(&mut self, request: &PublishRequest) -> Result<()>;
    fn publish(&mut self, request: &PublishRequest) -> Result<()>;
    fn verify(&mut self, request: &PublishRequest) -> Result<()>;
    fn deploy_website(&mut self) -> Result<()>;
}

struct ProductionEffects<'a> {
    rc: &'a origen_metal::utils::revision_control::RevisionControl,
    repo: &'a GithubRepository,
    release_branch: String,
    requests: Vec<PublishRequest>,
    paths: Vec<PathBuf>,
    app_root: PathBuf,
}

impl ProductionEffects<'_> {
    fn workflow<'a>(&self, request: &'a PublishRequest) -> Result<&'a str> {
        request
            .workflow
            .as_deref()
            .ok_or_else(|| origen::Error::new("A GitHub Actions workflow was not configured"))
    }

    fn monitor_run(&self, mut run: WorkflowRun) -> Result<()> {
        for _ in 0..120 {
            if run.completed() {
                if run.conclusion.as_deref() == Some("success") {
                    return Ok(());
                }
                bail!(
                    "Publication workflow failed: {} ({:?})",
                    run.html_url,
                    run.conclusion
                )
            }
            thread::sleep(Duration::from_secs(5));
            run = run.refresh()?;
        }
        bail!(
            "Timed out waiting for publication workflow {}",
            run.html_url
        )
    }

    fn wait_for_dispatched_run(&self, workflow: &str, previous_id: Option<u64>) -> Result<()> {
        for _ in 0..120 {
            if let Ok(run) =
                get_latest_workflow_dispatch(&self.repo.owner, &self.repo.name, Some(workflow))
            {
                if previous_id.map_or(true, |id| run.id != id) {
                    return self.monitor_run(run);
                }
            }
            thread::sleep(Duration::from_secs(5));
        }
        bail!(
            "Timed out waiting for dispatched workflow '{}' to start",
            workflow
        )
    }
}

impl ReleaseEffects for ProductionEffects<'_> {
    fn validate(&mut self) -> Result<()> {
        validate_remote_preflight(self.rc, &self.release_branch, &self.requests)
    }

    fn commit(&mut self) -> Result<()> {
        let paths: Vec<&Path> = self.paths.iter().map(|p| p.as_path()).collect();
        self.rc
            .checkin(Some(paths), "Update versions and release histories", false)?;
        Ok(())
    }

    fn tag(&mut self, request: &PublishRequest) -> Result<()> {
        self.rc.tag(
            &request.tag,
            false,
            Some(&format!(
                "Release {} {}",
                request.product.name(),
                request.version
            )),
        )
    }

    fn publish(&mut self, request: &PublishRequest) -> Result<()> {
        let workflow = self.workflow(request)?;
        let latest =
            get_latest_workflow_dispatch(&self.repo.owner, &self.repo.name, Some(workflow)).ok();
        if let Some(run) = latest.as_ref() {
            if run.head_branch == request.tag {
                if run.completed() && run.conclusion.as_deref() == Some("success") {
                    return Ok(());
                }
                if !run.completed() {
                    return self.monitor_run(run.refresh()?);
                }
            } else if !run.completed() {
                bail!(
                    "Another '{}' publication workflow is already running at {}; resume after it completes",
                    workflow,
                    run.html_url
                )
            }
        }
        let before = latest.map(|r| r.id);
        let inputs = match request.product {
            Product::Origen => serde_json::json!({
                "release_ref": request.tag,
                "version": request.version,
                "publish_pypi": true,
                "publish_pypi_test": false,
                "publish_github_release": true,
                "prerelease": request.version.contains("dev") || request.version.contains("alpha") || request.version.contains("beta"),
            }),
            Product::Metal => serde_json::json!({
                "release_ref": request.tag,
                "version": request.version,
                "publish_python": true,
                "publish_rust": true,
                "publish_github_release": true,
            }),
        };
        dispatch_workflow(
            &self.repo.owner,
            &self.repo.name,
            workflow,
            &request.tag,
            Some(inputs),
        )?;
        self.wait_for_dispatched_run(workflow, before)
    }

    fn verify(&mut self, request: &PublishRequest) -> Result<()> {
        let package = match request.product {
            Product::Origen => "origen",
            Product::Metal => "origen-metal",
        };
        // Reads backwards, so to be explicit: 'is_package_version_available'
        // answers "is this version number still free on the registry?". After a
        // successful publication it must be taken, so a 'true' here means the
        // upload did not land.
        if is_package_version_available(package, &request.version)? {
            bail!(
                "{} {} was not found on PyPI after publication",
                package,
                request.version
            )
        }
        if request.product == Product::Metal
            && is_crate_version_available("origen_metal", &request.version)?
        {
            bail!(
                "origen_metal {} was not found on crates.io after publication",
                request.version
            )
        }
        Ok(())
    }

    fn deploy_website(&mut self) -> Result<()> {
        let status = Command::new(std::env::current_exe()?)
            .current_dir(&self.app_root)
            .args(["web", "build", "--release"])
            .status()?;
        if !status.success() {
            bail!("Website build/deployment failed")
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
struct GithubRepository {
    owner: String,
    name: String,
}

impl GithubRepository {
    fn from_remote(remote: &str) -> Result<Self> {
        let normalized = normalize_repository(remote);
        let path = normalized
            .strip_prefix("https://github.com/")
            .ok_or_else(|| {
                origen::Error::new(&format!(
                    "GitHub Actions provider requires a github.com remote, found '{}'",
                    remote
                ))
            })?;
        let mut pieces = path.split('/');
        let owner = pieces
            .next()
            .filter(|p| !p.is_empty())
            .ok_or_else(|| origen::Error::new("GitHub repository owner is missing"))?;
        let name = pieces
            .next()
            .filter(|p| !p.is_empty())
            .ok_or_else(|| origen::Error::new("GitHub repository name is missing"))?;
        if pieces.next().is_some() {
            bail!("Unexpected GitHub repository path '{}'", path)
        }
        Ok(Self {
            owner: owner.to_string(),
            name: name.to_string(),
        })
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
struct ReleaseState {
    release_id: String,
    source_remote: String,
    release_branch: String,
    provider: String,
    completed: Vec<String>,
    requests: Vec<PublishRequest>,
}

impl ReleaseState {
    fn new(
        release_id: String,
        requests: Vec<PublishRequest>,
        source_remote: String,
        release_branch: String,
        provider: String,
    ) -> Self {
        Self {
            release_id,
            source_remote,
            release_branch,
            provider,
            completed: Vec::new(),
            requests,
        }
    }

    fn load(path: &Path) -> Result<Self> {
        Ok(toml::from_str(&fs::read_to_string(path)?)?)
    }

    fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, toml::to_string(self)?)?;
        Ok(())
    }

    fn complete(&mut self, phase: &str) {
        if !self.completed.iter().any(|p| p == phase) {
            self.completed.push(phase.to_string());
        }
    }

    fn is_complete(&self, phase: &str) -> bool {
        self.completed.iter().any(|p| p == phase)
    }
}

fn run_phase<F>(state: &mut ReleaseState, state_path: &Path, phase: &str, action: F) -> Result<()>
where
    F: FnOnce() -> Result<()>,
{
    if state.is_complete(phase) {
        displayln!("Skipping completed release phase: {}", phase);
        return Ok(());
    }
    displayln!("Running release phase: {}", phase);
    action()?;
    state.complete(phase);
    state.save(state_path)?;
    Ok(())
}

fn execute_remote_phases<E: ReleaseEffects>(
    effects: &mut E,
    requests: &[PublishRequest],
    state: &mut ReleaseState,
    state_path: &Path,
) -> Result<()> {
    run_phase(state, state_path, "remote_validated", || effects.validate())?;
    run_phase(state, state_path, "committed", || effects.commit())?;
    for request in requests {
        run_phase(
            state,
            state_path,
            &format!("tagged:{}", request.tag),
            || effects.tag(request),
        )?;
    }
    for request in requests {
        run_phase(
            state,
            state_path,
            &format!("published:{}", request.tag),
            || effects.publish(request),
        )?;
        run_phase(
            state,
            state_path,
            &format!("verified:{}", request.tag),
            || effects.verify(request),
        )?;
    }
    run_phase(state, state_path, "website_deployed", || {
        effects.deploy_website()
    })?;
    Ok(())
}

pub(crate) fn run(invocation: &clap::ArgMatches) -> Result<()> {
    // The O2 repository root is a development workspace rather than an Origen
    // application, while the core application configuration lives under
    // python/origen. Make the repository root the stable maintainer entry point
    // by re-entering the command from that application directory. A fresh
    // process is required because application discovery occurs during CLI boot.
    if origen::app().is_none() && origen::STATUS.is_origen_present {
        let core_app = origen::STATUS.origen_wksp_root.join("python/origen");
        if !core_app.join("config/application.toml").is_file() {
            bail!(
                "O2 core application was not found at {}",
                core_app.display()
            )
        }
        let status = Command::new(std::env::current_exe()?)
            .args(std::env::args().skip(1))
            .current_dir(core_app)
            .status()?;
        std::process::exit(status.code().unwrap_or(1));
    }

    let (_, args) = invocation.subcommand().unwrap();
    let non_interactive = *args.get_one::<bool>("non-interactive").unwrap();
    let dry_run = *args.get_one::<bool>("dry-run").unwrap();
    let local = *args.get_one::<bool>("local").unwrap();

    let app =
        origen::app().ok_or_else(|| origen::Error::new("rc tag requires an Origen application"))?;
    let (configured_remote, release_branch, release_config) = app.with_config(|config| {
        let rc = config
            .revision_control
            .as_ref()
            .ok_or_else(|| origen::Error::new("No revision-control configuration was found"))?;
        let remote = rc
            .get("remote")
            .cloned()
            .ok_or_else(|| origen::Error::new("revision_control.remote is required"))?;
        let branch = rc
            .get("release_branch")
            .cloned()
            .ok_or_else(|| origen::Error::new("revision_control.release_branch is required"))?;
        let release = config
            .release
            .clone()
            .ok_or_else(|| origen::Error::new("release configuration is required"))?;
        Ok((remote, branch, release))
    })?;
    let provider = release_config
        .get("provider")
        .cloned()
        .ok_or_else(|| origen::Error::new("release.provider is required"))?;
    let rc = app.rc()?;
    let actual_remote = rc.remote_url()?;
    if normalize_repository(&actual_remote) != normalize_repository(&configured_remote) {
        bail!(
            "Checkout origin '{}' does not match configured revision-control remote '{}'",
            actual_remote,
            configured_remote
        )
    }
    let root = origen::STATUS.origen_wksp_root.clone();
    if let Some(release_id) = args.get_one::<String>("resume") {
        if dry_run || local {
            bail!("--resume continues a prepared remote release and cannot be combined with --dry-run or --local")
        }
        let state_path = app
            .root
            .join(".origen/releases")
            .join(format!("{}.toml", release_id));
        if !state_path.is_file() {
            bail!(
                "No resumable release state exists at {}",
                state_path.display()
            )
        }
        let mut state = ReleaseState::load(&state_path)?;
        if state.release_id != *release_id
            || !state.is_complete("workspace_validated")
            || !state.is_complete("prepared")
        {
            bail!(
                "Release state '{}' is not a completed preparation",
                release_id
            )
        }
        if normalize_repository(&state.source_remote) != normalize_repository(&configured_remote)
            || state.release_branch != release_branch
            || state.provider != provider
        {
            bail!(
                "Configured release source/provider changed since '{}' was prepared",
                release_id
            )
        }
        if provider != "github_actions" {
            bail!("Resume currently requires the github_actions release provider")
        }
        let repository = GithubRepository::from_remote(&configured_remote)?;
        let paths = release_paths(&root, &state.requests)?;
        let requests = state.requests.clone();
        let mut effects = ProductionEffects {
            rc: &rc,
            repo: &repository,
            release_branch: state.release_branch.clone(),
            requests: requests.clone(),
            paths,
            app_root: app.root.clone(),
        };
        execute_remote_phases(&mut effects, &requests, &mut state, &state_path)?;
        displayln!("Release '{}' resumed successfully", release_id);
        return Ok(());
    }
    let status = rc.status(None)?;
    if status.is_modified() && !*args.get_one::<bool>("allow-local-changes").unwrap() {
        bail!("Workspace has local changes; commit/stash them or use --allow-local-changes for a local simulation")
    }

    let products = selected_products(args, non_interactive)?;
    if products.len() > 1 && args.get_one::<String>("type").is_some() {
        bail!("--type is only valid for a single-product release; combined releases use --origen-type and --metal-type")
    }
    let note = release_note(args, non_interactive, &app.root)?;
    let author = args
        .get_one::<String>("author")
        .cloned()
        .or_else(|| git::config("name"))
        .ok_or_else(|| {
            origen::Error::new(
                "Release author is required; configure Git user.name or pass --author",
            )
        })?;

    let github_repository = if provider == "github_actions" {
        Some(GithubRepository::from_remote(&configured_remote)?)
    } else {
        None
    };
    let mut plans = Vec::new();
    for product in products {
        let (path, cargo_paths) = match product {
            Product::Origen => (
                root.join("python/origen/pyproject.toml"),
                vec![
                    root.join("rust/origen/Cargo.toml"),
                    root.join("rust/origen/cli/Cargo.toml"),
                    root.join("rust/pyapi/Cargo.toml"),
                ],
            ),
            Product::Metal => (
                root.join("python/origen_metal/pyproject.toml"),
                vec![
                    root.join("rust/origen_metal/Cargo.toml"),
                    root.join("rust/pyapi_metal/Cargo.toml"),
                ],
            ),
        };
        let release_type = selected_release_type(args, product, non_interactive)?;
        let mut version = Version::from_pyproject_with_toml_handle(path)?;
        let package_repository = version
            .get_other(&["project", "urls", "Repository"])?
            .as_str()
            .ok_or_else(|| origen::Error::new("project.urls.Repository must be a string"))?;
        if normalize_repository(package_repository) != normalize_repository(&configured_remote) {
            bail!(
                "Package repository '{}' does not match configured revision-control remote '{}'",
                package_repository,
                configured_remote,
            )
        }
        if release_type != "current" {
            version.set_new_version(proposed_version(version.orig_version(), &release_type)?)?;
        }
        let mut cargo_versions = Vec::new();
        for cargo_path in cargo_paths {
            let mut cargo_version = Version::from_cargo_with_toml_handle(cargo_path)?;
            if release_type != "current" {
                let mut target = version.version().clone();
                target.convert_to_semver();
                cargo_version.set_new_version(target)?;
            } else {
                let mut cargo_current = cargo_version.orig_version().clone();
                cargo_current.convert_to_pep440();
                if cargo_current.to_string() != version.version().to_string() {
                    bail!(
                        "Cannot release current {} version {} because {} contains {}; increment or align the versions first",
                        product.name(),
                        version.version(),
                        cargo_version.source().display(),
                        cargo_version.orig_version(),
                    )
                }
            }
            cargo_versions.push(cargo_version);
        }
        let workflow = if provider == "github_actions" {
            let key = match product {
                Product::Origen => "origen_workflow",
                Product::Metal => "origen_metal_workflow",
            };
            Some(
                release_config
                    .get(key)
                    .cloned()
                    .ok_or_else(|| origen::Error::new(&format!("release.{} is required", key)))?,
            )
        } else {
            None
        };
        plans.push(PlannedProduct {
            product,
            version,
            cargo_versions,
            release_type,
            workflow,
        });
    }

    let metal_version = plans
        .iter()
        .find(|p| p.product == Product::Metal && p.release_type != "current")
        .map(|p| p.version.version().to_string());
    let updated_metal_requirement = metal_version.clone();
    if let Some(metal_version) = metal_version {
        if let Some(origen_plan) = plans.iter_mut().find(|p| p.product == Product::Origen) {
            if origen_plan.release_type == "current" {
                bail!("A combined release that changes Origen Metal must also increment Origen so its Metal dependency can be updated")
            }
            origen_plan
                .version
                .set_dependency("origen-metal", &format!("~={}", metal_version))?;
        }
    }

    let release_id = plans
        .iter()
        .map(|p| {
            format!(
                "{}-v{}",
                p.product.release_product().tag_prefix(),
                p.version.version()
            )
        })
        .collect::<Vec<String>>()
        .join("__");
    let requests: Vec<PublishRequest> = plans
        .iter()
        .map(|plan| PublishRequest {
            product: plan.product,
            tag: format!(
                "{}-v{}",
                plan.product.release_product().tag_prefix(),
                plan.version.version()
            ),
            version: plan.version.version().to_string(),
            workflow: plan.workflow.clone(),
        })
        .collect();
    let state_path = app
        .root
        .join(".origen/releases")
        .join(format!("{}.toml", release_id));
    let mut release_state = ReleaseState::new(
        release_id,
        requests.clone(),
        configured_remote.clone(),
        release_branch.clone(),
        provider.clone(),
    );

    displayln!("Release Plan");
    displayln!("  Release ID: {}", release_state.release_id);
    for plan in &plans {
        displayln!(
            "  {}: {} -> {} ({})",
            plan.product.name(),
            plan.version.orig_version(),
            plan.version.version(),
            plan.release_type
        );
        displayln!(
            "    tag: {}-v{}",
            plan.product.release_product().tag_prefix(),
            plan.version.version()
        );
        if let Some(workflow) = &plan.workflow {
            displayln!("    workflow: {}", workflow);
        }
    }
    displayln!("  Author: {}", author);
    if plans.iter().any(|p| p.product == Product::Origen) {
        if let Some(version) = &updated_metal_requirement {
            displayln!("  Origen dependency: origen-metal~={}", version);
        }
    }
    displayln!("  Source: {} ({})", configured_remote, release_branch);
    displayln!("  Publication provider: {}", provider);
    if let Some(repo) = &github_repository {
        displayln!("  GitHub repository: {}/{}", repo.owner, repo.name);
    }
    displayln!(
        "  Mode: {}",
        if dry_run {
            "dry-run"
        } else if local {
            "local"
        } else {
            "release"
        }
    );
    if release_state.is_complete("prepared") {
        displayln!("  Resume: local preparation was already completed");
    }

    if !*args.get_one::<bool>("yes").unwrap() {
        if non_interactive {
            bail!("--non-interactive requires --yes after all release inputs are supplied")
        }
        if !Confirm::new()
            .with_prompt("Proceed with this release plan?")
            .default(false)
            .interact()?
        {
            displayln!("Release cancelled");
            return Ok(());
        }
    }

    if dry_run {
        displayln!("Dry run complete; no files, commits, tags, workflows, registries, or websites were changed");
        return Ok(());
    }
    if !local && *args.get_one::<bool>("allow-local-changes").unwrap() {
        bail!("--allow-local-changes is only permitted with --dry-run or --local")
    }
    if !local && provider != "github_actions" {
        bail!(
            "Unsupported remote release provider '{}'; use --local or configure github_actions",
            provider
        )
    }
    validate_uv_version()?;
    run_phase(
        &mut release_state,
        &state_path,
        "workspace_validated",
        || validate_release_workspace(&root),
    )?;

    if !local {
        run_phase(&mut release_state, &state_path, "remote_validated", || {
            validate_remote_preflight(&rc, &release_branch, &requests)
        })?;
    }

    let released = time::now()
        .strftime("%Y-%m-%d")
        .map_err(|e| origen::Error::new(&format!("Could not format release date: {}", e)))?
        .to_string();
    let mut touched = Vec::new();
    for plan in &plans {
        if plan.release_type != "current" {
            touched.push(plan.version.source().clone());
            touched.extend(plan.cargo_versions.iter().map(|v| v.source().clone()));
        }
        touched.push(ReleaseScribe::for_product(plan.product.release_product())?.history_file);
    }
    touched.extend(release_lock_paths(&root, &requests));
    touched.extend(release_uv_lock_paths(&root)?);
    if requests.iter().any(|r| r.product == Product::Origen) {
        touched.push(
            root.join("python/origen/origen/__bin__/bin")
                .join(if cfg!(windows) {
                    "origen.exe"
                } else {
                    "origen"
                }),
        );
    }
    run_phase(&mut release_state, &state_path, "prepared", || {
        let backups = backup_files(&touched)?;
        let result: Result<()> = (|| {
            for mut plan in plans {
                let scribe = ReleaseScribe::for_product(plan.product.release_product())?;
                let anchor = format!(
                    "(release-{}-{})=",
                    plan.product.release_product().tag_prefix(),
                    plan.version.version().to_string().replace('.', "-")
                );
                if scribe.history_file.is_file()
                    && fs::read_to_string(&scribe.history_file)?.contains(&anchor)
                {
                    bail!(
                        "Release history already contains {} {}",
                        plan.product.name(),
                        plan.version.version(),
                    )
                }
                if plan.release_type != "current" {
                    plan.version.write()?;
                    for cargo_version in &mut plan.cargo_versions {
                        cargo_version.write()?;
                    }
                }
                scribe.prepend_release(
                    plan.product.release_product(),
                    plan.version.version(),
                    &author,
                    &note,
                    &released,
                )?;
            }
            update_release_locks(&root, &requests)?;
            refresh_uv_locks(&root)?;
            validate_prepared_artifacts(&root, &requests)?;
            validate_prepared_documentation(&app.root)?;
            Ok(())
        })();
        if let Err(error) = result {
            restore_files(&backups)?;
            run_checked(
                Path::new("uv"),
                &["sync", "--all-groups", "--no-editable"],
                &root.join("python/origen"),
            )?;
            bail!(
                "Release preparation failed and all file changes were rolled back: {}",
                error
            )
        }
        Ok(())
    })?;
    if local {
        displayln!("Local release preparation complete; no commit, tag, push, publication, workflow, or website deployment was performed");
        return Ok(());
    }
    let repo = github_repository.as_ref().ok_or_else(|| {
        origen::Error::new("GitHub Actions provider requires a GitHub repository")
    })?;
    let commit_paths = release_paths(&root, &requests)?;
    let mut effects = ProductionEffects {
        rc: &rc,
        repo,
        release_branch: release_branch.clone(),
        requests: requests.clone(),
        paths: commit_paths,
        app_root: app.root.clone(),
    };
    execute_remote_phases(&mut effects, &requests, &mut release_state, &state_path)?;
    displayln!("Release completed successfully");
    Ok(())
}

fn validate_remote_preflight(
    rc: &origen_metal::utils::revision_control::RevisionControl,
    release_branch: &str,
    requests: &[PublishRequest],
) -> Result<()> {
    if rc.current_branch()? != release_branch {
        bail!("Releases must run from branch '{}'", release_branch)
    }
    let latest = rc.confirm_latest_ref(release_branch)?;
    if !latest.0 {
        bail!(
            "Local HEAD {} does not match upstream {}",
            latest.1[0],
            latest.1[1]
        )
    }
    for request in requests {
        if rc.tag_exists(&request.tag)? {
            bail!("Release tag '{}' already exists", request.tag)
        }
        let package = match request.product {
            Product::Origen => "origen",
            Product::Metal => "origen-metal",
        };
        if !is_package_version_available(package, &request.version)? {
            bail!("{} {} already exists on PyPI", package, request.version)
        }
        if request.product == Product::Metal
            && !is_crate_version_available("origen_metal", &request.version)?
        {
            bail!(
                "origen_metal {} already exists on crates.io",
                request.version
            )
        }
    }
    Ok(())
}

fn run_checked(program: &Path, args: &[&str], cwd: &Path) -> Result<()> {
    let status = Command::new(program).args(args).current_dir(cwd).status()?;
    if !status.success() {
        bail!(
            "Command failed in {}: {} {}",
            cwd.display(),
            program.display(),
            args.join(" ")
        )
    }
    Ok(())
}

fn run_checked_with_env(
    program: &Path,
    args: &[&str],
    cwd: &Path,
    envs: &[(&str, String)],
) -> Result<()> {
    let mut command = Command::new(program);
    command.args(args).current_dir(cwd);
    for (name, value) in envs {
        command.env(name, value);
    }
    let status = command.status()?;
    if !status.success() {
        bail!(
            "Command failed in {}: {} {}",
            cwd.display(),
            program.display(),
            args.join(" ")
        )
    }
    Ok(())
}

fn release_paths(root: &Path, requests: &[PublishRequest]) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    for request in requests {
        match request.product {
            Product::Origen => paths.extend([
                root.join("python/origen/pyproject.toml"),
                root.join("rust/origen/Cargo.toml"),
                root.join("rust/origen/cli/Cargo.toml"),
                root.join("rust/pyapi/Cargo.toml"),
                root.join("python/origen/doc/history"),
            ]),
            Product::Metal => paths.extend([
                root.join("python/origen_metal/pyproject.toml"),
                root.join("rust/origen_metal/Cargo.toml"),
                root.join("rust/pyapi_metal/Cargo.toml"),
                root.join("python/origen/doc/metal/history"),
            ]),
        }
    }
    paths.extend(release_lock_paths(root, requests));
    paths.extend(release_uv_lock_paths(root)?);
    paths.sort();
    paths.dedup();
    for path in &paths {
        if !path.is_file() {
            bail!("Expected release file '{}' does not exist", path.display())
        }
    }
    Ok(paths)
}

fn release_uv_lock_paths(root: &Path) -> Result<Vec<PathBuf>> {
    fn collect(dir: &Path, locks: &mut Vec<PathBuf>) -> Result<()> {
        if !dir.is_dir() {
            return Ok(());
        }
        for entry in fs::read_dir(dir)? {
            let path = entry?.path();
            if path.is_dir() {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if !name.starts_with('.') && name != "target" && name != "build" && name != "output"
                {
                    collect(&path, locks)?;
                }
            } else if path.file_name().and_then(|n| n.to_str()) == Some("uv.lock") {
                locks.push(path);
            }
        }
        Ok(())
    }
    let mut locks = Vec::new();
    collect(&root.join("python"), &mut locks)?;
    collect(&root.join("test_apps"), &mut locks)?;
    locks.sort();
    Ok(locks)
}

fn validate_uv_version() -> Result<()> {
    let output = Command::new("uv").arg("--version").output()?;
    if !output.status.success() {
        bail!("Could not query UV version")
    }
    let text = String::from_utf8(output.stdout)?;
    let version = text
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| origen::Error::new("Could not parse UV version"))?;
    let found = semver::Version::parse(version).map_err(|e| {
        origen::Error::new(&format!("Could not parse UV version '{}': {}", version, e))
    })?;
    let minimum = semver::Version::parse("0.12.0").unwrap();
    if found < minimum {
        bail!(
            "UV {} is too old for release lock generation; install UV 0.12.0 or newer",
            found
        )
    }
    Ok(())
}

fn refresh_uv_locks(root: &Path) -> Result<()> {
    for lock in release_uv_lock_paths(root)? {
        run_checked(Path::new("uv"), &["lock"], lock.parent().unwrap())?;
    }
    Ok(())
}

fn release_lock_paths(root: &Path, requests: &[PublishRequest]) -> Vec<PathBuf> {
    let origen = requests.iter().any(|r| r.product == Product::Origen);
    let metal = requests.iter().any(|r| r.product == Product::Metal);
    let mut paths = Vec::new();
    if origen || metal {
        paths.extend([
            root.join("rust/origen/Cargo.lock"),
            root.join("rust/pyapi/Cargo.lock"),
        ]);
    }
    if metal {
        paths.extend([
            root.join("rust/origen_metal/Cargo.lock"),
            root.join("rust/pyapi_metal/Cargo.lock"),
        ]);
    }
    paths
}

fn update_release_locks(root: &Path, requests: &[PublishRequest]) -> Result<()> {
    let mut updates = std::collections::HashMap::new();
    for request in requests {
        let mut version = Version::new_pep440(&request.version)?;
        version.convert_to_semver();
        let version = version.to_string();
        match request.product {
            Product::Origen => {
                updates.insert("origen", version.clone());
                updates.insert("origen_pyapi", version.clone());
                updates.insert("cli", version);
            }
            Product::Metal => {
                updates.insert("origen_metal", version.clone());
                updates.insert("origen-metal", version);
            }
        }
    }
    for path in release_lock_paths(root, requests) {
        let original = fs::read_to_string(&path)?;
        let mut package: Option<&str> = None;
        let mut changed = false;
        let mut rendered = Vec::new();
        for line in original.lines() {
            if line == "[[package]]" {
                package = None;
            } else if package.is_none() && line.starts_with("name = \"") {
                package = line
                    .strip_prefix("name = \"")
                    .and_then(|v| v.strip_suffix('"'));
            }
            if line.starts_with("version = \"") {
                if let Some(version) = package.and_then(|name| updates.get(name)) {
                    rendered.push(format!("version = \"{}\"", version));
                    changed |= line != rendered.last().unwrap();
                    continue;
                }
            }
            rendered.push(line.to_string());
        }
        if changed {
            fs::write(&path, format!("{}\n", rendered.join("\n")))?;
        }
    }
    Ok(())
}

fn validate_release_workspace(root: &Path) -> Result<()> {
    run_checked(
        Path::new("cargo"),
        &["test", "-p", "cli", "--locked"],
        &root.join("rust/origen"),
    )?;
    run_checked(
        Path::new("cargo"),
        &["test", "-p", "origen", "--lib", "--locked"],
        &root.join("rust/origen"),
    )?;
    run_checked(
        Path::new("cargo"),
        &["test", "-p", "origen_metal", "--locked"],
        &root.join("rust/origen"),
    )?;
    let executable = std::env::current_exe()?;
    let origen_root = root.join("python/origen");
    run_checked(
        &executable,
        &["develop_origen", "build", "--metal"],
        &origen_root,
    )?;
    run_checked(&executable, &["develop_origen", "build"], &origen_root)?;
    let debug_cli = root.join("rust/origen/target/debug");
    let python_path = [
        root.join("rust/pyapi/target"),
        root.join("python/origen_metal"),
        root.join("python/origen"),
    ]
    .iter()
    .map(|p| p.to_string_lossy().to_string())
    .collect::<Vec<String>>()
    .join(if cfg!(windows) { ";" } else { ":" });
    let path = format!(
        "{}{}{}",
        debug_cli.display(),
        if cfg!(windows) { ";" } else { ":" },
        std::env::var("PATH").unwrap_or_default()
    );
    let envs = [("PATH", path), ("PYTHONPATH", python_path)];
    for test_app in ["test_apps/python_app", "test_apps/python_no_app"] {
        let cwd = root.join(test_app);
        run_checked(
            Path::new("uv"),
            &["sync", "--all-groups", "--no-editable"],
            &cwd,
        )?;
        run_checked_with_env(
            Path::new("uv"),
            &["run", "--no-sync", "--no-editable", "pytest", "-q"],
            &cwd,
            &envs,
        )?;
    }
    Ok(())
}

fn validate_prepared_documentation(app_root: &Path) -> Result<()> {
    let executable = std::env::current_exe()?;
    run_checked(
        &executable,
        &["web", "build", "--clean", "--as-release"],
        app_root,
    )
}

fn validate_prepared_artifacts(root: &Path, requests: &[PublishRequest]) -> Result<()> {
    if requests.iter().any(|r| r.product == Product::Origen) {
        run_checked(
            Path::new("cargo"),
            &[
                "build",
                "--manifest-path",
                "rust/origen/cli/Cargo.toml",
                "--release",
                "--bin",
                "origen",
            ],
            root,
        )?;
        let executable = if cfg!(windows) {
            "origen.exe"
        } else {
            "origen"
        };
        let built_cli = root.join("rust/origen/target/release").join(executable);
        let packaged_cli = root
            .join("python/origen/origen/__bin__/bin")
            .join(executable);
        fs::copy(&built_cli, &packaged_cli).map_err(|e| {
            origen::Error::new(&format!(
                "Could not stage release CLI from {} to {}: {}",
                built_cli.display(),
                packaged_cli.display(),
                e
            ))
        })?;
        run_checked(
            Path::new("uv"),
            &["build", "--wheel"],
            &root.join("python/origen"),
        )?;
    }
    if requests.iter().any(|r| r.product == Product::Metal) {
        run_checked(
            Path::new("uv"),
            &["build", "--wheel"],
            &root.join("python/origen_metal"),
        )?;
        run_checked(
            Path::new("cargo"),
            &["publish", "--dry-run", "--locked"],
            &root.join("rust/origen_metal"),
        )?;
    }
    // Refresh the environment used by the documentation command so imported
    // package metadata and native modules match the proposed release versions.
    run_checked(
        Path::new("uv"),
        &["sync", "--all-groups", "--no-editable"],
        &root.join("python/origen"),
    )?;
    Ok(())
}

fn selected_products(args: &clap::ArgMatches, non_interactive: bool) -> Result<Vec<Product>> {
    if let Some(values) = args.get_many::<String>("product") {
        let mut products = Vec::new();
        for value in values {
            let product = Product::parse(value);
            if !products.contains(&product) {
                products.push(product);
            }
        }
        return Ok(products);
    }
    if non_interactive {
        bail!("--product is required in non-interactive mode")
    }
    let choices = ["Origen", "Origen Metal", "Both"];
    Ok(
        match Select::new()
            .with_prompt("Product to release")
            .items(&choices)
            .default(0)
            .interact()?
        {
            0 => vec![Product::Origen],
            1 => vec![Product::Metal],
            _ => vec![Product::Origen, Product::Metal],
        },
    )
}

fn selected_release_type(
    args: &clap::ArgMatches,
    product: Product,
    non_interactive: bool,
) -> Result<String> {
    let specific = match product {
        Product::Origen => "origen-type",
        Product::Metal => "metal-type",
    };
    if let Some(value) = args
        .get_one::<String>(specific)
        .or_else(|| args.get_one::<String>("type"))
    {
        return Ok(value.to_string());
    }
    if non_interactive {
        bail!(
            "A release type is required for {} in non-interactive mode",
            product.name()
        )
    }
    let selected = Select::new()
        .with_prompt(format!("{} release type", product.name()))
        .items(&RELEASE_TYPES)
        .default(0)
        .interact()?;
    Ok(RELEASE_TYPES[selected].to_string())
}

fn release_note(args: &clap::ArgMatches, non_interactive: bool, app_root: &Path) -> Result<String> {
    let note = if let Some(note) = args.get_one::<String>("note") {
        note.to_string()
    } else {
        let path = args
            .get_one::<String>("file")
            .map(PathBuf::from)
            .or_else(|| {
                let default = app_root.join("release_note.txt");
                if default.is_file() {
                    Some(default)
                } else {
                    None
                }
            });
        if let Some(path) = path {
            fs::read_to_string(path)?
        } else if non_interactive {
            bail!("A release note is required in non-interactive mode (--note or --file)")
        } else {
            Input::<String>::new()
                .with_prompt("Release note")
                .interact_text()?
        }
    };
    if note.trim().is_empty() {
        bail!("Release note cannot be empty")
    }
    Ok(note)
}

fn to_release_type(value: &str) -> Result<ReleaseType> {
    Ok(match value {
        "development" => ReleaseType::Dev,
        "patch" => ReleaseType::Patch,
        "minor" => ReleaseType::Minor,
        "major" => ReleaseType::Major,
        _ => bail!("Unsupported release type '{}'", value),
    })
}

fn normalize_repository(value: &str) -> String {
    value
        .trim()
        .trim_end_matches('/')
        .trim_end_matches(".git")
        .replace("git@github.com:", "https://github.com/")
        .to_lowercase()
}

fn backup_files(paths: &[PathBuf]) -> Result<Vec<(PathBuf, Option<Vec<u8>>)>> {
    let mut seen = HashSet::new();
    let mut backups = Vec::new();
    for path in paths {
        if seen.insert(path.clone()) {
            backups.push((
                path.clone(),
                if path.exists() {
                    Some(fs::read(path)?)
                } else {
                    None
                },
            ));
        }
    }
    Ok(backups)
}

fn restore_files(backups: &[(PathBuf, Option<Vec<u8>>)]) -> Result<()> {
    for (path, contents) in backups {
        match contents {
            Some(contents) => {
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(path, contents)?;
            }
            None if path.exists() => fs::remove_file(path)?,
            None => {}
        }
    }
    Ok(())
}

fn production_version(
    version: &origen_metal::utils::version::Version,
) -> Result<origen_metal::utils::version::Version> {
    let text = version.to_string();
    let pieces: Vec<&str> = text.split(|c| c == '.' || c == '-').collect();
    if pieces.len() < 3 {
        bail!("Could not derive a production version from '{}'", text)
    }
    origen_metal::utils::version::Version::new_pep440(&pieces[..3].join("."))
}

fn proposed_version(current: &Version, release_type: &str) -> Result<Version> {
    if release_type == "current" {
        return Ok(current.clone());
    }
    if release_type == "production" {
        if !current.is_prerelease()? {
            bail!("{} is already a production version", current)
        }
        return production_version(current);
    }
    current.increment(&to_release_type(release_type)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use origen_metal::utils::version::Version;

    #[derive(Default)]
    struct FakeEffects {
        calls: Vec<String>,
        fail_on: Option<String>,
    }

    impl FakeEffects {
        fn call(&mut self, value: String) -> Result<()> {
            self.calls.push(value.clone());
            if self.fail_on.as_ref() == Some(&value) {
                bail!("simulated failure at {}", value)
            }
            Ok(())
        }
    }

    impl ReleaseEffects for FakeEffects {
        fn validate(&mut self) -> Result<()> {
            self.call("validate".to_string())
        }
        fn commit(&mut self) -> Result<()> {
            self.call("commit".to_string())
        }
        fn tag(&mut self, request: &PublishRequest) -> Result<()> {
            self.call(format!("tag:{}", request.tag))
        }
        fn publish(&mut self, request: &PublishRequest) -> Result<()> {
            self.call(format!("publish:{}", request.tag))
        }
        fn verify(&mut self, request: &PublishRequest) -> Result<()> {
            self.call(format!("verify:{}", request.tag))
        }
        fn deploy_website(&mut self) -> Result<()> {
            self.call("website".to_string())
        }
    }

    fn requests() -> Vec<PublishRequest> {
        vec![
            PublishRequest {
                product: Product::Origen,
                tag: "origen-v2.0.0.dev9".to_string(),
                version: "2.0.0.dev9".to_string(),
                workflow: Some("publish.yml".to_string()),
            },
            PublishRequest {
                product: Product::Metal,
                tag: "origen-metal-v1.6.0".to_string(),
                version: "1.6.0".to_string(),
                workflow: Some("publish_metal.yml".to_string()),
            },
        ]
    }

    fn state(id: &str, requests: Vec<PublishRequest>) -> ReleaseState {
        ReleaseState::new(
            id.to_string(),
            requests,
            "https://github.com/example/repo.git".to_string(),
            "main".to_string(),
            "fake".to_string(),
        )
    }

    #[test]
    fn repository_urls_are_compared_independent_of_git_transport() {
        assert_eq!(
            normalize_repository("git@github.com:Origen-SDK/o2.git"),
            normalize_repository("https://github.com/Origen-SDK/o2/"),
        );
    }

    #[test]
    fn github_repository_is_derived_from_configured_remote() -> Result<()> {
        assert_eq!(
            GithubRepository::from_remote("git@github.com:Origen-SDK/o2.git")?,
            GithubRepository {
                owner: "origen-sdk".to_string(),
                name: "o2".to_string()
            },
        );
        assert!(GithubRepository::from_remote("https://example.com/O2/o2.git").is_err());
        Ok(())
    }

    #[test]
    fn prerelease_versions_can_be_promoted() -> Result<()> {
        let version = Version::new_pep440("2.0.0.dev8")?;
        assert_eq!(production_version(&version)?.to_string(), "2.0.0");
        Ok(())
    }

    #[test]
    fn all_release_types_calculate_expected_versions() -> Result<()> {
        let production = Version::new_pep440("1.5.0")?;
        assert_eq!(
            proposed_version(&production, "current")?.to_string(),
            "1.5.0"
        );
        assert_eq!(
            proposed_version(&production, "development")?.to_string(),
            "1.5.1.dev0"
        );
        assert_eq!(proposed_version(&production, "patch")?.to_string(), "1.5.1");
        assert_eq!(proposed_version(&production, "minor")?.to_string(), "1.6.0");
        assert_eq!(proposed_version(&production, "major")?.to_string(), "2.0.0");
        let development = Version::new_pep440("2.0.0.dev8")?;
        assert_eq!(
            proposed_version(&development, "development")?.to_string(),
            "2.0.0.dev9"
        );
        assert_eq!(
            proposed_version(&development, "production")?.to_string(),
            "2.0.0"
        );
        assert!(proposed_version(&production, "production").is_err());
        Ok(())
    }

    #[test]
    fn release_lock_versions_are_updated_without_reformatting() -> Result<()> {
        let root = tempfile::tempdir()?;
        for relative in ["rust/origen/Cargo.lock", "rust/pyapi/Cargo.lock"] {
            let path = root.path().join(relative);
            fs::create_dir_all(path.parent().unwrap())?;
            fs::write(
                &path,
                "version = 3\n\n[[package]]\nname = \"origen\"\nversion = \"2.0.0-dev.8\"\n\n[[package]]\nname = \"unrelated\"\nversion = \"9.9.9\"\n",
            )?;
        }
        let request = PublishRequest {
            product: Product::Origen,
            tag: "origen-v2.0.0.dev9".to_string(),
            version: "2.0.0.dev9".to_string(),
            workflow: Some("publish.yml".to_string()),
        };
        update_release_locks(root.path(), &[request])?;
        let lock = fs::read_to_string(root.path().join("rust/origen/Cargo.lock"))?;
        assert!(lock.contains("name = \"origen\"\nversion = \"2.0.0-dev.9\""));
        assert!(lock.contains("name = \"unrelated\"\nversion = \"9.9.9\""));
        assert!(lock.starts_with("version = 3\n"));
        Ok(())
    }

    #[test]
    fn uv_lock_collection_ignores_generated_and_hidden_directories() -> Result<()> {
        let root = tempfile::tempdir()?;
        for relative in [
            "python/origen/uv.lock",
            "test_apps/app/uv.lock",
            "test_apps/app/.venv/uv.lock",
            "python/origen/build/uv.lock",
        ] {
            let path = root.path().join(relative);
            fs::create_dir_all(path.parent().unwrap())?;
            fs::write(path, "")?;
        }
        assert_eq!(release_uv_lock_paths(root.path())?.len(), 2);
        Ok(())
    }

    #[test]
    fn file_backups_restore_modified_and_new_files() -> Result<()> {
        let root = tempfile::tempdir()?;
        let existing = root.path().join("existing");
        let created = root.path().join("created");
        fs::write(&existing, "before")?;
        let backups = backup_files(&[existing.clone(), created.clone(), existing.clone()])?;
        fs::write(&existing, "after")?;
        fs::write(&created, "new")?;
        restore_files(&backups)?;
        assert_eq!(fs::read_to_string(existing)?, "before");
        assert!(!created.exists());
        Ok(())
    }

    #[test]
    fn release_state_round_trips_and_phases_are_idempotent() -> Result<()> {
        let root = tempfile::tempdir()?;
        let path = root.path().join(".origen/releases/origen-v2.0.0.toml");
        let mut state = state("origen-v2.0.0", requests());
        state.complete("prepared");
        state.complete("prepared");
        state.save(&path)?;
        let loaded = ReleaseState::load(&path)?;
        assert_eq!(loaded.release_id, "origen-v2.0.0");
        assert_eq!(loaded.completed, vec!["prepared"]);
        assert!(loaded.is_complete("prepared"));
        assert!(!loaded.is_complete("published"));
        Ok(())
    }

    #[test]
    fn completed_release_phases_are_not_repeated() -> Result<()> {
        let root = tempfile::tempdir()?;
        let path = root.path().join("state.toml");
        let mut state = state("fixture", requests());
        let mut calls = 0;
        run_phase(&mut state, &path, "published", || {
            calls += 1;
            Ok(())
        })?;
        run_phase(&mut state, &path, "published", || {
            calls += 1;
            Ok(())
        })?;
        assert_eq!(calls, 1);
        assert!(ReleaseState::load(&path)?.is_complete("published"));
        Ok(())
    }

    #[test]
    fn remote_phases_are_ordered_and_combined_release_deploys_once() -> Result<()> {
        let root = tempfile::tempdir()?;
        let path = root.path().join("state.toml");
        let mut state = state("combined", requests());
        let mut effects = FakeEffects::default();
        execute_remote_phases(&mut effects, &requests(), &mut state, &path)?;
        assert_eq!(
            effects.calls,
            vec![
                "validate",
                "commit",
                "tag:origen-v2.0.0.dev9",
                "tag:origen-metal-v1.6.0",
                "publish:origen-v2.0.0.dev9",
                "verify:origen-v2.0.0.dev9",
                "publish:origen-metal-v1.6.0",
                "verify:origen-metal-v1.6.0",
                "website",
            ]
        );
        Ok(())
    }

    #[test]
    fn failed_remote_phase_resumes_without_repeating_completed_work() -> Result<()> {
        let root = tempfile::tempdir()?;
        let path = root.path().join("state.toml");
        let requests = requests();
        let mut state = state("combined", requests.clone());
        let mut first = FakeEffects {
            calls: Vec::new(),
            fail_on: Some("publish:origen-metal-v1.6.0".to_string()),
        };
        assert!(execute_remote_phases(&mut first, &requests, &mut state, &path).is_err());
        let mut state = ReleaseState::load(&path)?;
        let mut resumed = FakeEffects::default();
        execute_remote_phases(&mut resumed, &requests, &mut state, &path)?;
        assert_eq!(
            resumed.calls,
            vec![
                "publish:origen-metal-v1.6.0",
                "verify:origen-metal-v1.6.0",
                "website",
            ]
        );
        Ok(())
    }

    #[test]
    fn failed_website_deployment_resumes_without_republishing() -> Result<()> {
        let root = tempfile::tempdir()?;
        let path = root.path().join("state.toml");
        let requests = vec![requests().remove(0)];
        let mut state = state("origen", requests.clone());
        let mut first = FakeEffects {
            calls: Vec::new(),
            fail_on: Some("website".to_string()),
        };
        assert!(execute_remote_phases(&mut first, &requests, &mut state, &path).is_err());
        let mut state = ReleaseState::load(&path)?;
        let mut resumed = FakeEffects::default();
        execute_remote_phases(&mut resumed, &requests, &mut state, &path)?;
        assert_eq!(resumed.calls, vec!["website"]);
        Ok(())
    }
}
