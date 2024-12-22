use crate::Result;
use std::fmt::Display;
use std::path::PathBuf;
use std::fs;
use glob;

pub fn resolve_relative_paths_to_strings<D: Display>(
    paths: &Vec<D>,
    relative_to: &PathBuf,
) -> Vec<String> {
    let mut retn: Vec<String> = vec![];
    for p in paths.iter() {
        retn.push(to_abs_path(p, relative_to).display().to_string());
    }
    retn
}

/// Given a relative path and a path its relative to, resolve the absolute path
///
/// In Windows, 'canonicalize' prepends "\\?\".
/// However, upon some google searching, it apparently is
/// dangerous to just remove as, in certain circumstances,
/// it does resolve to a different pattern.
/// But, other libraries, like glob, cannot handle it,
/// even when the ? is escaped.
/// So, instead just mashing the relative piece atop the
/// relative_to directory and users of these paths
/// can decide how to resolve.
/// For example, glob has no problem correctly resolving
/// this, without canonicalize
///
/// TL;DR - this needs some work in the future
pub fn to_abs_path<D: Display>(path: D, relative_to: &PathBuf) -> PathBuf {
    let pb = PathBuf::from(path.to_string());
    if pb.is_relative() {
        let mut resolved = PathBuf::from(relative_to);
        resolved.push(pb);
        resolved
    } else {
        pb
    }
}

/// Render a potential template, or return the contents of the non-template file.
pub fn preprocess_as_template(path: &PathBuf) -> Result<Option<String>> {
    let mut entries = vec![];
    for entry in glob::glob(&path.display().to_string())? {
        match entry {
            Ok(e) => entries.push(e),
            Err(e) => bail!("Error reading entry: {}", e),
        }
    }
    if entries.is_empty() {
        Ok(None)
    } else {
        let name = super::resolve_os_str(path.file_name().unwrap_or(std::ffi::OsStr::new("")))?;
        if entries.len() > 1 {
            // Not supporting multiple paths or having some hierarchy
            // Just report as an error
            let mut entries_list = String::new();
            for entry in &entries {
                entries_list.push_str(&format!("  {:?}\n", entry));
            }
            bail!("Multiple matches found for '{}':\n{}", name, entries_list);
        } else {
            let file = entries.first().unwrap();
            let content = fs::read_to_string(&file)?;

            if file.extension().map_or(false, |ext| ext == "j2" || ext == "jinja") {
                // Evaluate using minijinja
                let mut env = minijinja::Environment::new();
                let file_content = fs::read_to_string(&file)?;
                env.add_template(&name, &file_content)?;
                let tmpl = env.get_template(&name)?;
                Ok(Some(tmpl.render("")?))
            } else {
                Ok(Some(content))
            }
        }
    }
}