use std::path::PathBuf;
use std::fmt::Display;

pub fn resolve_relative_paths_to_strings<D: Display>(paths: &Vec<D>, relative_to: &PathBuf) -> Vec<String> {
    let mut retn: Vec<String> = vec!();
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
