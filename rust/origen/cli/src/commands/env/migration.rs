use crate::commands::_prelude::*;
use similar::TextDiff;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use toml_edit::{value, Array, ArrayOfTables, Document, InlineTable, Item, Table, Value};

const PYPROJECT: &str = "pyproject.toml";
const POETRY_LOCK: &str = "poetry.lock";
const UV_LOCK: &str = "uv.lock";
const HATCHLING_REQUIREMENT: &str = "hatchling>=1.17.1,<1.18";
const HATCHLING_BACKEND: &str = "hatchling.build";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ManifestState {
    PoetryOnly,
    Pep621Only,
    Conflicting,
    Neither,
}

#[derive(Debug, Clone)]
pub(crate) struct MigrationPlan {
    pub(crate) manifest: String,
    project_name: String,
    direct_dependencies: BTreeSet<String>,
    installable: bool,
}

#[derive(Debug)]
struct ConvertedDependency {
    requirement: String,
    normalized_name: String,
    source: Option<InlineTable>,
    optional: bool,
}

#[derive(Debug)]
struct IndexSource {
    name: String,
    url: String,
}

#[derive(Debug, Clone)]
struct FileSnapshot {
    path: PathBuf,
    contents: Option<Vec<u8>>,
}

impl FileSnapshot {
    fn capture(path: PathBuf) -> Result<Self> {
        let contents = if path.exists() {
            Some(fs::read(&path)?)
        } else {
            None
        };
        Ok(Self { path, contents })
    }

    fn restore(&self) -> Result<()> {
        match &self.contents {
            Some(contents) => atomic_write(&self.path, contents),
            None => {
                if self.path.exists() {
                    fs::remove_file(&self.path)?;
                }
                Ok(())
            }
        }
    }
}

pub(crate) fn manifest_state(contents: &str) -> Result<ManifestState> {
    let doc = parse_document(contents)?;
    let has_project = doc.get("project").and_then(Item::as_table).is_some();
    let has_poetry = poetry_table(&doc).is_some();
    Ok(match (has_poetry, has_project) {
        (true, false) => ManifestState::PoetryOnly,
        (false, true) => ManifestState::Pep621Only,
        (true, true) => ManifestState::Conflicting,
        (false, false) => ManifestState::Neither,
    })
}

pub(crate) fn guard_uv_manifest(path: &Path) -> Result<()> {
    if !path.is_file() {
        return Ok(());
    }
    let contents = fs::read_to_string(path)?;
    if manifest_state(&contents)? == ManifestState::PoetryOnly {
        return Err(origen::Error::new(
            r#"This application uses Poetry-only metadata, which O2's UV environment
workflow does not read.

Preview the migration:
    origen env migrate --dry-run

Apply it:
    origen env migrate
    origen env setup"#,
        ));
    }
    Ok(())
}

pub(crate) fn plan_poetry_migration(contents: &str) -> Result<MigrationPlan> {
    let mut doc = parse_document(contents)?;
    match manifest_state(contents)? {
        ManifestState::PoetryOnly => {}
        ManifestState::Pep621Only => {
            return Err(origen::Error::new(
                "project: this manifest already uses PEP 621 metadata",
            ))
        }
        ManifestState::Conflicting => {
            return Err(origen::Error::new(
                "project and tool.poetry: both metadata tables exist; remove or reconcile one before migration",
            ))
        }
        ManifestState::Neither => {
            return Err(origen::Error::new(
                "tool.poetry: no Poetry project metadata was found",
            ))
        }
    }

    let poetry = poetry_table(&doc).unwrap().clone();
    let mut diagnostics = validate_poetry_keys(&poetry);
    validate_metadata_types(&poetry, &mut diagnostics);
    if doc
        .get("tool")
        .and_then(Item::as_table)
        .and_then(|tool| tool.get("uv"))
        .is_some()
    {
        diagnostics.push(
            "tool.uv: existing UV configuration must be reconciled manually before migration"
                .to_string(),
        );
    }
    if doc.get("dependency-groups").is_some() {
        diagnostics.push(
            "dependency-groups: existing dependency groups conflict with Poetry group ownership"
                .to_string(),
        );
    }
    let project_name =
        required_string(&poetry, "name", "tool.poetry.name", &mut diagnostics).unwrap_or_default();
    required_string(&poetry, "version", "tool.poetry.version", &mut diagnostics);

    let index_sources = convert_indexes(&poetry, &mut diagnostics);
    let explicit_indexes: BTreeSet<String> = index_sources
        .iter()
        .map(|index| index.name.clone())
        .collect();

    let mut project = Table::new();
    move_scalar(&poetry, &mut project, "name");
    move_scalar(&poetry, &mut project, "version");
    move_scalar(&poetry, &mut project, "description");
    move_scalar(&poetry, &mut project, "license");
    move_scalar(&poetry, &mut project, "readme");
    move_scalar(&poetry, &mut project, "keywords");
    move_scalar(&poetry, &mut project, "classifiers");
    convert_people(&poetry, &mut project, "authors", &mut diagnostics);
    convert_people(&poetry, &mut project, "maintainers", &mut diagnostics);
    convert_urls(&poetry, &mut project, &mut diagnostics);
    convert_scripts(&poetry, &mut project, &mut diagnostics);
    convert_plugins(&poetry, &mut project, &mut diagnostics);

    let dependencies = poetry
        .get("dependencies")
        .and_then(Item::as_table)
        .cloned()
        .unwrap_or_default();
    let python_requirement = dependencies.get("python").and_then(Item::as_str);
    match python_requirement {
        Some(requirement) => match translate_constraint(requirement) {
            Ok(requirement) if !requirement.is_empty() => {
                project.insert("requires-python", value(requirement));
            }
            Ok(_) => diagnostics.push(
                "tool.poetry.dependencies.python: an unconstrained Python version cannot become project.requires-python"
                    .to_string(),
            ),
            Err(message) => diagnostics.push(format!(
                "tool.poetry.dependencies.python: {}",
                message
            )),
        },
        None => diagnostics.push(
            "tool.poetry.dependencies.python: a Python requirement is required for project.requires-python"
                .to_string(),
        ),
    }

    let mut runtime = Vec::new();
    let mut optional = BTreeMap::<String, ConvertedDependency>::new();
    let mut uv_sources = Table::new();
    let mut direct_dependencies = BTreeSet::new();
    for (name, item) in dependencies.iter().filter(|(name, _)| *name != "python") {
        match convert_dependency(name, item, &explicit_indexes) {
            Ok(converted) => {
                direct_dependencies.insert(converted.normalized_name.clone());
                if let Some(source) = converted.source.clone() {
                    uv_sources.insert(
                        &converted.normalized_name,
                        Item::Value(Value::InlineTable(source)),
                    );
                }
                if converted.optional {
                    optional.insert(canonicalize_name(name), converted);
                } else {
                    runtime.push(converted.requirement);
                }
            }
            Err(message) => {
                diagnostics.push(format!("tool.poetry.dependencies.{}: {}", name, message))
            }
        }
    }
    project.insert("dependencies", string_array_item(runtime));
    convert_extras(&poetry, &optional, &mut project, &mut diagnostics);

    let mut dependency_groups = Table::new();
    let mut default_groups = Vec::new();
    convert_dependency_groups(
        &poetry,
        &explicit_indexes,
        &mut dependency_groups,
        &mut default_groups,
        &mut uv_sources,
        &mut direct_dependencies,
        &mut diagnostics,
    );

    let package_mode = poetry
        .get("package-mode")
        .and_then(Item::as_bool)
        .unwrap_or(true);
    let installable = convert_build_system(&mut doc, package_mode, &mut diagnostics);

    if !diagnostics.is_empty() {
        diagnostics.sort();
        diagnostics.dedup();
        return Err(origen::Error::new(&format!(
            "Cannot migrate pyproject.toml because the following constructs are unsupported or ambiguous:\n{}",
            diagnostics
                .iter()
                .map(|message| format!("- {}", message))
                .collect::<Vec<_>>()
                .join("\n")
        )));
    }

    doc.as_table_mut().insert("project", Item::Table(project));
    let tool = doc
        .get_mut("tool")
        .and_then(Item::as_table_mut)
        .expect("Poetry metadata has a tool table");
    tool.remove("poetry");

    if !dependency_groups.is_empty() {
        doc.as_table_mut()
            .insert("dependency-groups", Item::Table(dependency_groups));
    }

    if !uv_sources.is_empty()
        || !index_sources.is_empty()
        || !default_groups.is_empty()
        || !installable
    {
        let tool = ensure_table(doc.as_table_mut(), "tool");
        let uv = ensure_table(tool, "uv");
        if !installable {
            uv.insert("package", value(false));
        }
        if !default_groups.is_empty() {
            uv.insert("default-groups", string_array_item(default_groups));
        }
        if !uv_sources.is_empty() {
            uv.insert("sources", Item::Table(uv_sources));
        }
        if !index_sources.is_empty() {
            let mut indexes = ArrayOfTables::new();
            for index in index_sources {
                let mut table = Table::new();
                table.insert("name", value(index.name));
                table.insert("url", value(index.url));
                table.insert("explicit", value(true));
                indexes.push(table);
            }
            uv.insert("index", Item::ArrayOfTables(indexes));
        }
    }

    Ok(MigrationPlan {
        manifest: doc.to_string(),
        project_name: canonicalize_name(&project_name),
        direct_dependencies,
        installable,
    })
}

pub(crate) fn run(invocation: &clap::ArgMatches) -> Result<()> {
    let project_arg = invocation.get_one::<String>("project").map(String::as_str);
    let root = find_project_root(project_arg)?;
    let pyproject = root.join(PYPROJECT);
    let original = fs::read_to_string(&pyproject)?;

    match manifest_state(&original)? {
        ManifestState::Pep621Only => {
            displayln!("Project at {} is already migrated to PEP 621/UV; no changes made.", root.display());
            return Ok(());
        }
        ManifestState::Conflicting => {
            return Err(origen::Error::new(
                "project and tool.poetry: both metadata tables exist; remove or reconcile one before migration",
            ))
        }
        ManifestState::Neither => {
            return Err(origen::Error::new(&format!(
                "No Poetry project metadata was found in {}",
                pyproject.display()
            )))
        }
        ManifestState::PoetryOnly => {}
    }

    let plan = plan_poetry_migration(&original)?;
    if *invocation.get_one::<bool>("dry-run").unwrap_or(&false) {
        let diff = TextDiff::from_lines(&original, &plan.manifest)
            .unified_diff()
            .header("a/pyproject.toml", "b/pyproject.toml")
            .to_string();
        display!("{}", diff);
        displayln!("Dry run only; no files were changed.");
        return Ok(());
    }

    super::ensure_uv_available()?;
    let removed_poetry_lock = root.join(POETRY_LOCK).is_file();
    apply_migration(&root, &plan, |root| super::run_uv(root, &["lock"]))?;
    displayln!("Migrated pyproject.toml from Poetry to PEP 621/UV.");
    displayln!("Generated uv.lock.");
    if removed_poetry_lock {
        displayln!("Removed poetry.lock.");
    }
    displayln!("Run 'origen env setup' to provision the environment.");
    displayln!("Review and commit pyproject.toml and uv.lock together.");
    Ok(())
}

fn apply_migration<F>(root: &Path, plan: &MigrationPlan, lock: F) -> Result<()>
where
    F: FnOnce(&Path) -> Result<()>,
{
    let pyproject = FileSnapshot::capture(root.join(PYPROJECT))?;
    let poetry_lock = FileSnapshot::capture(root.join(POETRY_LOCK))?;
    let uv_lock = FileSnapshot::capture(root.join(UV_LOCK))?;

    if let Some(contents) = &uv_lock.contents {
        validate_existing_lock(contents, &plan.project_name)?;
    }

    let operation = (|| -> Result<()> {
        atomic_write(&pyproject.path, plan.manifest.as_bytes())?;
        lock(root)?;
        validate_generated_lock(&root.join(UV_LOCK), plan)?;
        if poetry_lock.path.exists() {
            fs::remove_file(&poetry_lock.path)?;
        }
        Ok(())
    })();

    if let Err(error) = operation {
        let mut rollback_errors = Vec::new();
        for snapshot in [&pyproject, &poetry_lock, &uv_lock] {
            if let Err(rollback_error) = snapshot.restore() {
                rollback_errors.push(format!("{}: {}", snapshot.path.display(), rollback_error));
            }
        }
        if rollback_errors.is_empty() {
            return Err(error);
        }
        return Err(origen::Error::new(&format!(
            "{}; rollback also failed: {}",
            error,
            rollback_errors.join("; ")
        )));
    }
    Ok(())
}

fn validate_existing_lock(contents: &[u8], project_name: &str) -> Result<()> {
    let text = std::str::from_utf8(contents).map_err(|error| {
        origen::Error::new(&format!("Existing uv.lock is not UTF-8: {}", error))
    })?;
    let value: toml::Value = toml::from_str(text)
        .map_err(|error| origen::Error::new(&format!("Existing uv.lock is invalid: {}", error)))?;
    let packages = lock_package_names(&value);
    if !packages.contains(project_name) {
        return Err(origen::Error::new(&format!(
            "uv.lock: an existing lockfile cannot be established as belonging to project '{}'; remove it or migrate it manually",
            project_name
        )));
    }
    Ok(())
}

fn validate_generated_lock(path: &Path, plan: &MigrationPlan) -> Result<()> {
    if !path.is_file() {
        return Err(origen::Error::new(
            "uv lock completed without creating uv.lock",
        ));
    }
    let text = fs::read_to_string(path)?;
    let value: toml::Value = toml::from_str(&text)
        .map_err(|error| origen::Error::new(&format!("Generated uv.lock is invalid: {}", error)))?;
    let packages = lock_package_names(&value);
    let mut missing: Vec<String> = plan
        .direct_dependencies
        .difference(&packages)
        .cloned()
        .collect();
    if plan.installable && !packages.contains(&plan.project_name) {
        missing.push(format!("{} (root project)", plan.project_name));
    }
    if !missing.is_empty() {
        missing.sort();
        return Err(origen::Error::new(&format!(
            "Generated uv.lock is missing declared dependencies: {}",
            missing.join(", ")
        )));
    }
    Ok(())
}

fn lock_package_names(value: &toml::Value) -> BTreeSet<String> {
    value
        .get("package")
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|package| package.get("name").and_then(toml::Value::as_str))
        .map(canonicalize_name)
        .collect()
}

fn atomic_write(path: &Path, contents: &[u8]) -> Result<()> {
    let parent = path.parent().ok_or_else(|| {
        origen::Error::new(&format!("{} has no parent directory", path.display()))
    })?;
    let permissions = fs::metadata(path)
        .ok()
        .map(|metadata| metadata.permissions());
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    temporary.write_all(contents)?;
    temporary.as_file().sync_all()?;
    if let Some(permissions) = permissions {
        temporary.as_file().set_permissions(permissions)?;
    }
    temporary
        .persist(path)
        .map_err(|error| origen::Error::new(&error.to_string()))?;
    Ok(())
}

fn find_project_root(project: Option<&str>) -> Result<PathBuf> {
    let start = match project {
        Some(project) => {
            let path = PathBuf::from(project);
            let path = if path.is_absolute() {
                path
            } else {
                std::env::current_dir()?.join(path)
            };
            if !path.exists() {
                return Err(origen::Error::new(&format!(
                    "Project path '{}' does not exist",
                    path.display()
                )));
            }
            if path.is_file() {
                if path.file_name().and_then(|name| name.to_str()) != Some(PYPROJECT) {
                    return Err(origen::Error::new(&format!(
                        "Project path '{}' is a file but is not pyproject.toml",
                        path.display()
                    )));
                }
                path.parent().unwrap().to_path_buf()
            } else {
                path
            }
        }
        None => std::env::current_dir()?,
    };

    let start = start.canonicalize()?;
    for directory in start.ancestors() {
        if directory.join(PYPROJECT).is_file() {
            return Ok(directory.to_path_buf());
        }
    }
    Err(origen::Error::new(&format!(
        "Could not find pyproject.toml from {} or any parent directory",
        start.display()
    )))
}

fn parse_document(contents: &str) -> Result<Document> {
    contents
        .parse::<Document>()
        .map_err(|error| origen::Error::new(&format!("Could not parse pyproject.toml: {}", error)))
}

fn poetry_table(doc: &Document) -> Option<&Table> {
    doc.get("tool")
        .and_then(Item::as_table)
        .and_then(|tool| tool.get("poetry"))
        .and_then(Item::as_table)
}

fn ensure_table<'a>(parent: &'a mut Table, key: &str) -> &'a mut Table {
    if parent.get(key).and_then(Item::as_table).is_none() {
        parent.insert(key, Item::Table(Table::new()));
    }
    parent.get_mut(key).unwrap().as_table_mut().unwrap()
}

fn validate_poetry_keys(poetry: &Table) -> Vec<String> {
    let supported: BTreeSet<&str> = [
        "name",
        "version",
        "description",
        "authors",
        "maintainers",
        "license",
        "readme",
        "homepage",
        "repository",
        "documentation",
        "keywords",
        "classifiers",
        "scripts",
        "plugins",
        "dependencies",
        "dev-dependencies",
        "group",
        "extras",
        "source",
        "package-mode",
    ]
    .into_iter()
    .collect();
    poetry
        .iter()
        .filter(|(key, _)| !supported.contains(*key))
        .map(|(key, _)| {
            let reason = match key {
                "packages" | "include" | "exclude" => {
                    "custom package/include/exclude rules require a manual Hatchling configuration"
                }
                _ => "no exact PEP 621/UV mapping is implemented",
            };
            format!("tool.poetry.{}: {}", key, reason)
        })
        .collect()
}

fn validate_metadata_types(poetry: &Table, diagnostics: &mut Vec<String>) {
    for key in ["description", "license", "readme"] {
        if poetry.get(key).is_some() && poetry.get(key).and_then(Item::as_str).is_none() {
            diagnostics.push(format!("tool.poetry.{}: expected a string", key));
        }
    }
    for key in ["keywords", "classifiers"] {
        if let Some(item) = poetry.get(key) {
            let valid = item
                .as_array()
                .map(|values| values.iter().all(|value| value.as_str().is_some()))
                .unwrap_or(false);
            if !valid {
                diagnostics.push(format!("tool.poetry.{}: expected an array of strings", key));
            }
        }
    }
    if poetry.get("package-mode").is_some()
        && poetry.get("package-mode").and_then(Item::as_bool).is_none()
    {
        diagnostics.push("tool.poetry.package-mode: expected a boolean".to_string());
    }
    if poetry.get("dependencies").is_some()
        && poetry
            .get("dependencies")
            .and_then(Item::as_table)
            .is_none()
    {
        diagnostics.push("tool.poetry.dependencies: expected a table".to_string());
    }
}

fn required_string(
    table: &Table,
    key: &str,
    path: &str,
    diagnostics: &mut Vec<String>,
) -> Option<String> {
    match table.get(key).and_then(Item::as_str) {
        Some(value) if !value.trim().is_empty() => Some(value.to_string()),
        _ => {
            diagnostics.push(format!("{}: a non-empty string is required", path));
            None
        }
    }
}

fn move_scalar(from: &Table, to: &mut Table, key: &str) {
    if let Some(item) = from.get(key) {
        to.insert(key, item.clone());
    }
}

fn convert_people(poetry: &Table, project: &mut Table, key: &str, diagnostics: &mut Vec<String>) {
    let Some(item) = poetry.get(key) else {
        return;
    };
    let Some(values) = item.as_array() else {
        diagnostics.push(format!("tool.poetry.{}: expected an array of strings", key));
        return;
    };
    let mut people = Array::new();
    for (index, value) in values.iter().enumerate() {
        let Some(person) = value.as_str() else {
            diagnostics.push(format!("tool.poetry.{}[{}]: expected a string", key, index));
            continue;
        };
        let (name, email) = parse_person(person);
        if name.is_empty() && email.is_none() {
            diagnostics.push(format!(
                "tool.poetry.{}[{}]: author name or email is required",
                key, index
            ));
            continue;
        }
        let mut output = InlineTable::new();
        if !name.is_empty() {
            output.insert("name", Value::from(name));
        }
        if let Some(email) = email {
            output.insert("email", Value::from(email));
        }
        people.push(Value::InlineTable(output));
    }
    project.insert(key, Item::Value(Value::Array(people)));
}

fn parse_person(person: &str) -> (String, Option<String>) {
    let person = person.trim();
    if let Some(open) = person.rfind('<') {
        if person.ends_with('>') {
            let name = person[..open].trim().to_string();
            let email = person[open + 1..person.len() - 1].trim();
            if !email.is_empty() {
                return (name, Some(email.to_string()));
            }
        }
    }
    (person.to_string(), None)
}

fn convert_urls(poetry: &Table, project: &mut Table, diagnostics: &mut Vec<String>) {
    let mappings = [
        ("homepage", "Homepage"),
        ("repository", "Repository"),
        ("documentation", "Documentation"),
    ];
    let mut urls = Table::new();
    for (poetry_key, project_key) in mappings {
        if let Some(item) = poetry.get(poetry_key) {
            if item.as_str().is_some() {
                urls.insert(project_key, item.clone());
            } else {
                diagnostics.push(format!("tool.poetry.{}: expected a URL string", poetry_key));
            }
        }
    }
    if !urls.is_empty() {
        project.insert("urls", Item::Table(urls));
    }
}

fn convert_scripts(poetry: &Table, project: &mut Table, diagnostics: &mut Vec<String>) {
    let Some(scripts) = poetry.get("scripts").and_then(Item::as_table) else {
        if poetry.get("scripts").is_some() {
            diagnostics.push("tool.poetry.scripts: expected a table".to_string());
        }
        return;
    };
    let mut output = Table::new();
    for (name, item) in scripts {
        if item.as_str().is_some() {
            output.insert(name, item.clone());
        } else {
            diagnostics.push(format!(
                "tool.poetry.scripts.{}: only string console-script entries are supported",
                name
            ));
        }
    }
    if !output.is_empty() {
        project.insert("scripts", Item::Table(output));
    }
}

fn convert_plugins(poetry: &Table, project: &mut Table, diagnostics: &mut Vec<String>) {
    let Some(plugins) = poetry.get("plugins").and_then(Item::as_table) else {
        if poetry.get("plugins").is_some() {
            diagnostics.push("tool.poetry.plugins: expected a table".to_string());
        }
        return;
    };
    let mut groups = Table::new();
    for (group_name, group_item) in plugins {
        let Some(group) = group_item.as_table() else {
            diagnostics.push(format!(
                "tool.poetry.plugins.{}: expected a table",
                group_name
            ));
            continue;
        };
        let mut output = Table::new();
        for (name, item) in group {
            if item.as_str().is_some() {
                output.insert(name, item.clone());
            } else {
                diagnostics.push(format!(
                    "tool.poetry.plugins.{}.{}: expected an import string",
                    group_name, name
                ));
            }
        }
        groups.insert(group_name, Item::Table(output));
    }
    if !groups.is_empty() {
        project.insert("entry-points", Item::Table(groups));
    }
}

fn convert_indexes(poetry: &Table, diagnostics: &mut Vec<String>) -> Vec<IndexSource> {
    let Some(sources) = poetry.get("source") else {
        return Vec::new();
    };
    let Some(sources) = sources.as_array_of_tables() else {
        diagnostics.push("tool.poetry.source: expected an array of tables".to_string());
        return Vec::new();
    };
    let mut indexes = Vec::new();
    for (index, source) in sources.iter().enumerate() {
        let path = format!("tool.poetry.source[{}]", index);
        let name = source.get("name").and_then(Item::as_str);
        let url = source.get("url").and_then(Item::as_str);
        let priority = source.get("priority").and_then(Item::as_str);
        for (key, _) in source.iter() {
            if !matches!(key, "name" | "url" | "priority") {
                diagnostics.push(format!("{}.{}: unsupported source setting", path, key));
            }
        }
        if priority != Some("explicit") {
            diagnostics.push(format!(
                "{}.priority: only Poetry priority = \"explicit\" has an exact UV mapping",
                path
            ));
        }
        match (name, url) {
            (Some(name), Some(url)) => indexes.push(IndexSource {
                name: name.to_string(),
                url: url.to_string(),
            }),
            _ => diagnostics.push(format!("{}: name and url strings are required", path)),
        }
    }
    indexes
}

fn convert_dependency(
    name: &str,
    item: &Item,
    explicit_indexes: &BTreeSet<String>,
) -> std::result::Result<ConvertedDependency, String> {
    let normalized_name = canonicalize_name(name);
    if let Some(constraint) = item.as_str() {
        let constraint = translate_constraint(constraint)?;
        return Ok(ConvertedDependency {
            requirement: format_requirement(&normalized_name, &[], &constraint, &[]),
            normalized_name,
            source: None,
            optional: false,
        });
    }
    if item.as_bool() == Some(false) {
        return Err("disabled dependencies cannot be represented in PEP 621".to_string());
    }
    let fields = dependency_fields(item)?;
    let supported: BTreeSet<&str> = [
        "version",
        "extras",
        "markers",
        "python",
        "platform",
        "optional",
        "path",
        "develop",
        "git",
        "branch",
        "tag",
        "rev",
        "subdirectory",
        "url",
        "source",
    ]
    .into_iter()
    .collect();
    let unknown: Vec<String> = fields
        .keys()
        .filter(|key| !supported.contains(key.as_str()))
        .cloned()
        .collect();
    if !unknown.is_empty() {
        return Err(format!("unsupported fields: {}", unknown.join(", ")));
    }

    let version = fields
        .get("version")
        .and_then(Value::as_str)
        .map(translate_constraint)
        .transpose()?
        .unwrap_or_default();
    let extras = string_array_field(&fields, "extras")?;
    let optional = bool_field(&fields, "optional")?.unwrap_or(false);
    let develop = bool_field(&fields, "develop")?.unwrap_or(false);
    let mut markers = Vec::new();
    if let Some(marker) = fields.get("markers").and_then(Value::as_str) {
        markers.push(format!("({})", marker.trim()));
    } else if fields.contains_key("markers") {
        return Err("markers must be a string".to_string());
    }
    if let Some(python) = fields.get("python").and_then(Value::as_str) {
        markers.extend(python_markers(python)?);
    } else if fields.contains_key("python") {
        return Err("python must be a version-constraint string".to_string());
    }
    if let Some(platform) = fields.get("platform").and_then(Value::as_str) {
        markers.push(format!("sys_platform == '{}'", escape_marker(platform)));
    } else if fields.contains_key("platform") {
        return Err("platform must be a string".to_string());
    }

    let source_fields = ["path", "git", "url", "source"]
        .iter()
        .filter(|key| fields.contains_key(**key))
        .count();
    if source_fields > 1 {
        return Err("path, git, url, and source are mutually exclusive".to_string());
    }
    let mut source = None;
    if let Some(path) = fields.get("path").and_then(Value::as_str) {
        let mut table = InlineTable::new();
        table.insert("path", Value::from(path));
        if develop {
            table.insert("editable", Value::from(true));
        }
        source = Some(table);
    } else if let Some(git) = fields.get("git").and_then(Value::as_str) {
        let mut table = InlineTable::new();
        table.insert("git", Value::from(git));
        for key in ["branch", "tag", "rev", "subdirectory"] {
            if let Some(value) = fields.get(key).and_then(Value::as_str) {
                table.insert(key, Value::from(value));
            } else if fields.contains_key(key) {
                return Err(format!("{} must be a string", key));
            }
        }
        source = Some(table);
    } else if let Some(url) = fields.get("url").and_then(Value::as_str) {
        let mut table = InlineTable::new();
        table.insert("url", Value::from(url));
        source = Some(table);
    } else if let Some(index) = fields.get("source").and_then(Value::as_str) {
        if !explicit_indexes.contains(index) {
            return Err(format!(
                "source '{}' is not declared with Poetry priority = \"explicit\"",
                index
            ));
        }
        let mut table = InlineTable::new();
        table.insert("index", Value::from(index));
        source = Some(table);
    }
    if develop && !fields.contains_key("path") {
        return Err("develop is only valid for a local path dependency".to_string());
    }
    if !fields.contains_key("git") {
        for key in ["branch", "tag", "rev", "subdirectory"] {
            if fields.contains_key(key) {
                return Err(format!("{} requires a git source", key));
            }
        }
    }

    Ok(ConvertedDependency {
        requirement: format_requirement(&normalized_name, &extras, &version, &markers),
        normalized_name,
        source,
        optional,
    })
}

fn dependency_fields(item: &Item) -> std::result::Result<BTreeMap<String, Value>, String> {
    if let Some(table) = item.as_inline_table() {
        return Ok(table
            .iter()
            .map(|(key, value)| (key.to_string(), value.clone()))
            .collect());
    }
    if let Some(table) = item.as_table() {
        let mut fields = BTreeMap::new();
        for (key, item) in table {
            let Some(value) = item.as_value() else {
                return Err(format!("{} must be a scalar or array value", key));
            };
            fields.insert(key.to_string(), value.clone());
        }
        return Ok(fields);
    }
    Err("expected a version string or dependency table".to_string())
}

fn string_array_field(
    fields: &BTreeMap<String, Value>,
    key: &str,
) -> std::result::Result<Vec<String>, String> {
    let Some(value) = fields.get(key) else {
        return Ok(Vec::new());
    };
    let Some(array) = value.as_array() else {
        return Err(format!("{} must be an array of strings", key));
    };
    array
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_string)
                .ok_or_else(|| format!("{} must contain only strings", key))
        })
        .collect()
}

fn bool_field(
    fields: &BTreeMap<String, Value>,
    key: &str,
) -> std::result::Result<Option<bool>, String> {
    match fields.get(key) {
        Some(value) => value
            .as_bool()
            .map(Some)
            .ok_or_else(|| format!("{} must be a boolean", key)),
        None => Ok(None),
    }
}

fn format_requirement(name: &str, extras: &[String], version: &str, markers: &[String]) -> String {
    let mut requirement = name.to_string();
    if !extras.is_empty() {
        requirement.push('[');
        requirement.push_str(&extras.join(","));
        requirement.push(']');
    }
    requirement.push_str(version);
    if !markers.is_empty() {
        requirement.push_str("; ");
        requirement.push_str(&markers.join(" and "));
    }
    requirement
}

fn convert_extras(
    poetry: &Table,
    optional: &BTreeMap<String, ConvertedDependency>,
    project: &mut Table,
    diagnostics: &mut Vec<String>,
) {
    let Some(extras) = poetry.get("extras").and_then(Item::as_table) else {
        if poetry.get("extras").is_some() {
            diagnostics.push("tool.poetry.extras: expected a table".to_string());
        }
        for name in optional.keys() {
            diagnostics.push(format!(
                "tool.poetry.dependencies.{}: optional dependency is not assigned to a Poetry extra",
                name
            ));
        }
        return;
    };
    let mut output = Table::new();
    let mut referenced = BTreeSet::new();
    for (extra, item) in extras {
        let Some(names) = item.as_array() else {
            diagnostics.push(format!(
                "tool.poetry.extras.{}: expected an array of dependency names",
                extra
            ));
            continue;
        };
        let mut requirements = Vec::new();
        for value in names.iter() {
            let Some(name) = value.as_str() else {
                diagnostics.push(format!(
                    "tool.poetry.extras.{}: expected only dependency-name strings",
                    extra
                ));
                continue;
            };
            let normalized = canonicalize_name(name);
            match optional.get(&normalized) {
                Some(dependency) => {
                    referenced.insert(normalized);
                    requirements.push(dependency.requirement.clone());
                }
                None => diagnostics.push(format!(
                    "tool.poetry.extras.{}: '{}' is not an optional runtime dependency",
                    extra, name
                )),
            }
        }
        output.insert(extra, string_array_item(requirements));
    }
    for name in optional.keys().filter(|name| !referenced.contains(*name)) {
        diagnostics.push(format!(
            "tool.poetry.dependencies.{}: optional dependency is not assigned to a Poetry extra",
            name
        ));
    }
    if !output.is_empty() {
        project.insert("optional-dependencies", Item::Table(output));
    }
}

#[allow(clippy::too_many_arguments)]
fn convert_dependency_groups(
    poetry: &Table,
    explicit_indexes: &BTreeSet<String>,
    output: &mut Table,
    default_groups: &mut Vec<String>,
    uv_sources: &mut Table,
    direct_dependencies: &mut BTreeSet<String>,
    diagnostics: &mut Vec<String>,
) {
    let legacy_dev = poetry.get("dev-dependencies").and_then(Item::as_table);
    let groups = poetry.get("group").and_then(Item::as_table);
    if poetry.get("dev-dependencies").is_some() && legacy_dev.is_none() {
        diagnostics.push("tool.poetry.dev-dependencies: expected a table".to_string());
    }
    if poetry.get("group").is_some() && groups.is_none() {
        diagnostics.push("tool.poetry.group: expected a table".to_string());
    }
    if legacy_dev.is_some() && groups.and_then(|groups| groups.get("dev")).is_some() {
        diagnostics.push(
            "tool.poetry.dev-dependencies and tool.poetry.group.dev.dependencies: both define the dev group"
                .to_string(),
        );
    }
    if let Some(dependencies) = legacy_dev {
        convert_one_group(
            "dev",
            dependencies,
            false,
            explicit_indexes,
            output,
            default_groups,
            uv_sources,
            direct_dependencies,
            diagnostics,
        );
    }
    if let Some(groups) = groups {
        for (name, group_item) in groups {
            let Some(group) = group_item.as_table() else {
                diagnostics.push(format!("tool.poetry.group.{}: expected a table", name));
                continue;
            };
            for (key, _) in group {
                if !matches!(key, "dependencies" | "optional") {
                    diagnostics.push(format!(
                        "tool.poetry.group.{}.{}: unsupported group setting",
                        name, key
                    ));
                }
            }
            let optional = group
                .get("optional")
                .and_then(Item::as_bool)
                .unwrap_or(false);
            if group.get("optional").is_some()
                && group.get("optional").and_then(Item::as_bool).is_none()
            {
                diagnostics.push(format!(
                    "tool.poetry.group.{}.optional: expected a boolean",
                    name
                ));
            }
            let Some(dependencies) = group.get("dependencies").and_then(Item::as_table) else {
                diagnostics.push(format!(
                    "tool.poetry.group.{}.dependencies: expected a table",
                    name
                ));
                continue;
            };
            convert_one_group(
                name,
                dependencies,
                optional,
                explicit_indexes,
                output,
                default_groups,
                uv_sources,
                direct_dependencies,
                diagnostics,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn convert_one_group(
    name: &str,
    dependencies: &Table,
    optional_group: bool,
    explicit_indexes: &BTreeSet<String>,
    output: &mut Table,
    default_groups: &mut Vec<String>,
    uv_sources: &mut Table,
    direct_dependencies: &mut BTreeSet<String>,
    diagnostics: &mut Vec<String>,
) {
    let mut requirements = Vec::new();
    for (dependency_name, item) in dependencies {
        match convert_dependency(dependency_name, item, explicit_indexes) {
            Ok(dependency) => {
                if dependency.optional {
                    diagnostics.push(format!(
                        "tool.poetry.group.{}.dependencies.{}: dependency-level optional is ambiguous inside a group",
                        name, dependency_name
                    ));
                    continue;
                }
                if let Some(source) = dependency.source {
                    let existing = uv_sources.get(&dependency.normalized_name);
                    let candidate = Item::Value(Value::InlineTable(source));
                    if let Some(existing) = existing {
                        if existing.to_string() != candidate.to_string() {
                            diagnostics.push(format!(
                                "tool.poetry.group.{}.dependencies.{}: source conflicts with another dependency declaration",
                                name, dependency_name
                            ));
                        }
                    } else {
                        uv_sources.insert(&dependency.normalized_name, candidate);
                    }
                }
                direct_dependencies.insert(dependency.normalized_name);
                requirements.push(dependency.requirement);
            }
            Err(message) => diagnostics.push(format!(
                "tool.poetry.group.{}.dependencies.{}: {}",
                name, dependency_name, message
            )),
        }
    }
    output.insert(name, string_array_item(requirements));
    if !optional_group {
        default_groups.push(name.to_string());
    }
}

fn convert_build_system(
    doc: &mut Document,
    package_mode: bool,
    diagnostics: &mut Vec<String>,
) -> bool {
    if !package_mode {
        if doc.get("build-system").is_some() {
            diagnostics.push(
                "tool.poetry.package-mode and build-system: package-mode = false conflicts with an installable build system"
                    .to_string(),
            );
        }
        return false;
    }
    let Some(build) = doc.get_mut("build-system") else {
        return false;
    };
    let Some(build) = build.as_table_mut() else {
        diagnostics.push("build-system: expected a table".to_string());
        return false;
    };
    let backend_supported =
        build.get("build-backend").and_then(Item::as_str) == Some("poetry.core.masonry.api");
    if !backend_supported {
        diagnostics.push(
            "build-system.build-backend: only poetry.core.masonry.api can be migrated automatically"
                .to_string(),
        );
    }
    let poetry_core_required = build
        .get("requires")
        .and_then(Item::as_array)
        .map(|requirements| {
            requirements
                .iter()
                .all(|requirement| requirement.as_str().is_some())
                && requirements
                    .iter()
                    .filter_map(Value::as_str)
                    .any(|requirement| canonicalize_name(requirement).starts_with("poetry-core"))
        })
        .unwrap_or(false);
    if !poetry_core_required {
        diagnostics.push(
            "build-system.requires: expected a string array containing poetry-core".to_string(),
        );
    }
    for (key, _) in build.iter() {
        if !matches!(key, "requires" | "build-backend" | "backend-path") {
            diagnostics.push(format!("build-system.{}: unsupported build setting", key));
        }
    }
    if build.get("backend-path").is_some() {
        diagnostics.push(
            "build-system.backend-path: Poetry-to-Hatchling backend paths are ambiguous"
                .to_string(),
        );
    }
    if backend_supported && poetry_core_required {
        build.insert(
            "requires",
            string_array_item(vec![HATCHLING_REQUIREMENT.to_string()]),
        );
        build.insert("build-backend", value(HATCHLING_BACKEND));
        true
    } else {
        false
    }
}

fn string_array_item(values: Vec<String>) -> Item {
    let mut array = Array::new();
    for value in values {
        array.push(value);
    }
    Item::Value(Value::Array(array))
}

fn translate_constraint(constraint: &str) -> std::result::Result<String, String> {
    let constraint = constraint.trim();
    if constraint.is_empty() || constraint == "*" {
        return Ok(String::new());
    }
    if constraint.contains("||") {
        return Err(
            "union constraints using || do not have a single unambiguous PEP 508 mapping"
                .to_string(),
        );
    }
    let mut compact = constraint.to_string();
    for operator in ["===", ">=", "<=", "!=", "==", "~=", "^", "~", ">", "<"] {
        compact = compact.replace(&format!("{} ", operator), operator);
    }
    let parts: Vec<&str> = compact
        .split(',')
        .flat_map(|part| part.split_whitespace())
        .filter(|part| !part.is_empty())
        .collect();
    let mut translated = Vec::new();
    for part in parts {
        if let Some(version) = part.strip_prefix('^') {
            translated.push(format!(">={}", version));
            translated.push(format!("<{}", caret_upper_bound(version)?));
        } else if let Some(version) = part.strip_prefix('~') {
            if part.starts_with("~=") {
                translated.push(compact_operator_spacing(part));
            } else {
                translated.push(format!(">={}", version));
                translated.push(format!("<{}", tilde_upper_bound(version)?));
            }
        } else if starts_with_comparator(part) {
            translated.push(compact_operator_spacing(part));
        } else if part.contains('*') || part.contains('x') || part.contains('X') {
            translated.push(format!("=={}", part.replace(['x', 'X'], "*")));
        } else if is_version_literal(part) {
            translated.push(format!("=={}", part));
        } else {
            return Err(format!("constraint '{}' is not supported", part));
        }
    }
    Ok(translated.join(","))
}

fn starts_with_comparator(value: &str) -> bool {
    ["===", ">=", "<=", "!=", "==", "~=", ">", "<"]
        .iter()
        .any(|operator| value.starts_with(operator))
}

fn compact_operator_spacing(value: &str) -> String {
    let value = value.trim();
    for operator in ["===", ">=", "<=", "!=", "==", "~=", ">", "<"] {
        if let Some(rest) = value.strip_prefix(operator) {
            return format!("{}{}", operator, rest.trim());
        }
    }
    value.to_string()
}

fn is_version_literal(value: &str) -> bool {
    value
        .chars()
        .next()
        .map(|character| character.is_ascii_digit())
        .unwrap_or(false)
        && !value.chars().any(char::is_whitespace)
}

fn release_components(version: &str) -> std::result::Result<Vec<u64>, String> {
    let release = version
        .split(|character: char| !character.is_ascii_digit() && character != '.')
        .next()
        .unwrap_or_default();
    if release.is_empty() {
        return Err(format!(
            "version '{}' has no numeric release segment",
            version
        ));
    }
    release
        .split('.')
        .map(|part| {
            part.parse::<u64>()
                .map_err(|_| format!("version '{}' has an invalid release segment", version))
        })
        .collect()
}

fn caret_upper_bound(version: &str) -> std::result::Result<String, String> {
    let mut parts = release_components(version)?;
    let index = parts
        .iter()
        .position(|part| *part != 0)
        .unwrap_or(parts.len() - 1);
    parts[index] += 1;
    for part in parts.iter_mut().skip(index + 1) {
        *part = 0;
    }
    while parts.len() < 3 {
        parts.push(0);
    }
    Ok(parts
        .iter()
        .map(u64::to_string)
        .collect::<Vec<_>>()
        .join("."))
}

fn tilde_upper_bound(version: &str) -> std::result::Result<String, String> {
    let mut parts = release_components(version)?;
    let index = if parts.len() == 1 { 0 } else { 1 };
    while parts.len() <= index {
        parts.push(0);
    }
    parts[index] += 1;
    for part in parts.iter_mut().skip(index + 1) {
        *part = 0;
    }
    Ok(parts
        .iter()
        .map(u64::to_string)
        .collect::<Vec<_>>()
        .join("."))
}

fn python_markers(constraint: &str) -> std::result::Result<Vec<String>, String> {
    let translated = translate_constraint(constraint)?;
    if translated.is_empty() {
        return Ok(Vec::new());
    }
    translated
        .split(',')
        .map(|specifier| {
            for operator in ["===", ">=", "<=", "!=", "==", "~=", ">", "<"] {
                if let Some(version) = specifier.strip_prefix(operator) {
                    return Ok(format!(
                        "python_full_version {} '{}'",
                        operator,
                        escape_marker(version)
                    ));
                }
            }
            Err(format!(
                "Python constraint '{}' cannot be expressed as an environment marker",
                specifier
            ))
        })
        .collect()
}

fn escape_marker(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\'', "\\'")
}

fn canonicalize_name(name: &str) -> String {
    let mut output = String::new();
    let mut separator = false;
    for character in name.chars() {
        if matches!(character, '-' | '_' | '.') {
            separator = !output.is_empty();
        } else {
            if separator {
                output.push('-');
                separator = false;
            }
            output.extend(character.to_lowercase());
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    const HISTORICAL_FIXTURES: &[(&str, &str, bool)] = &[
        (
            "python_app",
            include_str!("../../../tests/fixtures/poetry/python_app.toml"),
            true,
        ),
        (
            "python_no_app",
            include_str!("../../../tests/fixtures/poetry/python_no_app.toml"),
            false,
        ),
        (
            "python_plugin",
            include_str!("../../../tests/fixtures/poetry/python_plugin.toml"),
            true,
        ),
        (
            "python_plugin_the_second",
            include_str!("../../../tests/fixtures/poetry/python_plugin_the_second.toml"),
            true,
        ),
        (
            "python_plugin_no_cmds",
            include_str!("../../../tests/fixtures/poetry/python_plugin_no_cmds.toml"),
            true,
        ),
        (
            "pl_ext_cmds",
            include_str!("../../../tests/fixtures/poetry/pl_ext_cmds.toml"),
            true,
        ),
        (
            "test_apps_shared_test_helpers",
            include_str!("../../../tests/fixtures/poetry/test_apps_shared_test_helpers.toml"),
            true,
        ),
        (
            "user_install",
            include_str!("../../../tests/fixtures/poetry/user_install.toml"),
            false,
        ),
        (
            "rendered_user_template",
            include_str!("../../../tests/fixtures/poetry/rendered_user_template.toml"),
            false,
        ),
    ];

    #[test]
    fn converts_every_historical_manifest_without_dropping_dependencies() {
        for (fixture_name, fixture, installable) in HISTORICAL_FIXTURES {
            let plan = plan_poetry_migration(fixture)
                .unwrap_or_else(|error| panic!("{} failed: {}", fixture_name, error));
            let converted: toml::Value = toml::from_str(&plan.manifest).unwrap();
            assert!(
                converted.get("project").is_some(),
                "{} has no project table",
                fixture_name
            );
            assert!(
                converted
                    .get("tool")
                    .and_then(|tool| tool.get("poetry"))
                    .is_none(),
                "{} retained Poetry metadata",
                fixture_name
            );
            assert!(
                converted
                    .get("project")
                    .and_then(|project| project.get("requires-python"))
                    .and_then(toml::Value::as_str)
                    .is_some(),
                "{} lost its Python requirement",
                fixture_name
            );
            assert_eq!(
                plan.installable, *installable,
                "{} installability changed",
                fixture_name
            );
            if *installable {
                assert_eq!(
                    converted["build-system"]["build-backend"].as_str(),
                    Some(HATCHLING_BACKEND),
                    "{} did not receive Hatchling",
                    fixture_name
                );
            } else {
                assert_eq!(
                    converted["tool"]["uv"]["package"].as_bool(),
                    Some(false),
                    "{} was not marked virtual",
                    fixture_name
                );
            }
            for dependency in &plan.direct_dependencies {
                assert!(
                    plan.manifest.contains(dependency),
                    "{} dropped {}",
                    fixture_name,
                    dependency
                );
            }
            if fixture.contains("[tool.pytest.ini_options]") {
                assert!(
                    converted["tool"].get("pytest").is_some(),
                    "{} lost unrelated pytest configuration",
                    fixture_name
                );
            }
            if fixture.contains("develop = true") {
                let sources = converted["tool"]["uv"]["sources"].as_table().unwrap();
                assert!(
                    sources.values().all(|source| {
                        source.get("editable").and_then(toml::Value::as_bool) == Some(true)
                    }),
                    "{} did not preserve Poetry develop intent",
                    fixture_name
                );
            }
        }
    }

    #[test]
    fn translates_boundary_sensitive_poetry_constraints() {
        let cases = [
            ("^1.2.3", ">=1.2.3,<2.0.0"),
            ("^0.2.3", ">=0.2.3,<0.3.0"),
            ("^0.0.3", ">=0.0.3,<0.0.4"),
            ("^3", ">=3,<4.0.0"),
            ("~1.2", ">=1.2,<1.3"),
            ("~1", ">=1,<2"),
            ("1.2.3", "==1.2.3"),
            ("1.2.*", "==1.2.*"),
            (">= 1.0, != 1.5, < 2", ">=1.0,!=1.5,<2"),
            (">=1.0 <2.0", ">=1.0,<2.0"),
            ("1.2.x", "==1.2.*"),
            ("^1.2.3rc1", ">=1.2.3rc1,<2.0.0"),
        ];
        for (poetry, pep440) in cases {
            assert_eq!(translate_constraint(poetry).unwrap(), pep440, "{}", poetry);
        }
    }

    #[test]
    fn converts_sources_extras_markers_groups_scripts_and_plugins() {
        let input = r#"
[tool.poetry]
name = "rich-project"
version = "1.0.0"
description = "Rich fixture"
authors = ["Example User <user@example.com>"]
homepage = "https://example.com"

[tool.poetry.dependencies]
python = "^3.8"
local-lib = { path = "../local", develop = true, extras = ["speed"], python = "^3.9", platform = "linux" }
git-lib = { git = "https://example.com/lib.git", tag = "v1", subdirectory = "python" }
url-lib = { url = "https://example.com/url-lib.whl" }
index-lib = { version = "^2", source = "private" }
optional-lib = { version = "~1.4", optional = true }

[tool.poetry.extras]
feature = ["optional-lib"]

[tool.poetry.group.docs]
optional = true

[tool.poetry.group.docs.dependencies]
sphinx = "^7"

[tool.poetry.scripts]
rich = "rich_project.cli:main"

[tool.poetry.plugins."origen.plugins"]
rich = "rich_project.plugin:Plugin"

[[tool.poetry.source]]
name = "private"
url = "https://packages.example.com/simple"
priority = "explicit"

[build-system]
requires = ["poetry-core>=1"]
build-backend = "poetry.core.masonry.api"
"#;
        let plan = plan_poetry_migration(input).unwrap();
        let converted: toml::Value = toml::from_str(&plan.manifest).unwrap();
        assert_eq!(
            converted["project"]["authors"][0]["email"].as_str(),
            Some("user@example.com")
        );
        assert_eq!(
            converted["project"]["scripts"]["rich"].as_str(),
            Some("rich_project.cli:main")
        );
        assert_eq!(
            converted["project"]["entry-points"]["origen.plugins"]["rich"].as_str(),
            Some("rich_project.plugin:Plugin")
        );
        assert_eq!(
            converted["tool"]["uv"]["sources"]["local-lib"]["editable"].as_bool(),
            Some(true)
        );
        assert_eq!(
            converted["tool"]["uv"]["sources"]["git-lib"]["tag"].as_str(),
            Some("v1")
        );
        assert_eq!(
            converted["tool"]["uv"]["sources"]["index-lib"]["index"].as_str(),
            Some("private")
        );
        assert_eq!(
            converted["tool"]["uv"]["index"][0]["explicit"].as_bool(),
            Some(true)
        );
        assert!(converted["project"]["dependencies"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(toml::Value::as_str)
            .any(|dependency| {
                dependency.contains("python_full_version >= '3.9'")
                    && dependency.contains("sys_platform == 'linux'")
            }));
        assert_eq!(
            converted["project"]["optional-dependencies"]["feature"][0].as_str(),
            Some("optional-lib>=1.4,<1.5")
        );
        assert_eq!(
            converted["dependency-groups"]["docs"][0].as_str(),
            Some("sphinx>=7,<8.0.0")
        );
        assert!(converted["tool"]["uv"].get("default-groups").is_none());
    }

    #[test]
    fn reports_all_unsupported_constructs_before_conversion() {
        let input = r#"
[tool.poetry]
name = "unsupported"
version = "1.0.0"
description = ""
authors = ["Origen-SDK"]
packages = [{ include = "src" }]
include = ["data"]

[tool.poetry.dependencies]
python = ">=3.8"
variant = [{ version = "^1" }, { version = "^2" }]
"#;
        let error = plan_poetry_migration(input).unwrap_err().to_string();
        assert!(error.contains("tool.poetry.packages"));
        assert!(error.contains("tool.poetry.include"));
        assert!(error.contains("tool.poetry.dependencies.variant"));
    }

    #[test]
    fn successful_transaction_generates_lock_and_removes_poetry_lock() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        let original = HISTORICAL_FIXTURES[2].1;
        fs::write(root.join(PYPROJECT), original).unwrap();
        fs::write(root.join(POETRY_LOCK), b"original poetry lock").unwrap();
        let plan = plan_poetry_migration(original).unwrap();

        apply_migration(root, &plan, |root| {
            fs::write(
                root.join(UV_LOCK),
                fake_lock(
                    std::iter::once(plan.project_name.as_str())
                        .chain(plan.direct_dependencies.iter().map(String::as_str)),
                ),
            )?;
            Ok(())
        })
        .unwrap();

        assert_eq!(
            manifest_state(&fs::read_to_string(root.join(PYPROJECT)).unwrap()).unwrap(),
            ManifestState::Pep621Only
        );
        assert!(root.join(UV_LOCK).is_file());
        assert!(!root.join(POETRY_LOCK).exists());
    }

    #[test]
    fn failed_transaction_restores_every_file_byte_for_byte() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        let original_manifest = HISTORICAL_FIXTURES[2].1.as_bytes();
        let original_poetry_lock = b"original poetry lock";
        let plan = plan_poetry_migration(HISTORICAL_FIXTURES[2].1).unwrap();
        let original_uv_lock = fake_lock([plan.project_name.as_str()]);
        fs::write(root.join(PYPROJECT), original_manifest).unwrap();
        fs::write(root.join(POETRY_LOCK), original_poetry_lock).unwrap();
        fs::write(root.join(UV_LOCK), &original_uv_lock).unwrap();

        let error = apply_migration(root, &plan, |root| {
            fs::write(root.join(UV_LOCK), b"partial lock")?;
            Err(origen::Error::new("forced lock failure"))
        })
        .unwrap_err();

        assert!(error.to_string().contains("forced lock failure"));
        assert_eq!(fs::read(root.join(PYPROJECT)).unwrap(), original_manifest);
        assert_eq!(
            fs::read(root.join(POETRY_LOCK)).unwrap(),
            original_poetry_lock
        );
        assert_eq!(fs::read(root.join(UV_LOCK)).unwrap(), original_uv_lock);
    }

    #[test]
    fn poetry_guard_stops_uv_backed_commands_with_migration_instructions() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join(PYPROJECT);
        fs::write(&path, HISTORICAL_FIXTURES[0].1).unwrap();
        let error = guard_uv_manifest(&path).unwrap_err().to_string();
        assert!(error.contains("origen env migrate --dry-run"));
        assert!(error.contains("origen env migrate"));
        assert!(error.contains("origen env setup"));
    }

    fn fake_lock<'a>(names: impl IntoIterator<Item = &'a str>) -> Vec<u8> {
        let mut lock = String::from("version = 1\n");
        for name in names {
            lock.push_str(&format!(
                "\n[[package]]\nname = {:?}\nversion = \"1.0.0\"\n",
                name
            ));
        }
        lock.into_bytes()
    }
}
