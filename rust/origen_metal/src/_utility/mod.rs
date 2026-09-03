pub mod file_utils;

/// Internal utility functions, not meant for user consumption and definitely not meant to be
/// (directly) wrapped by any frontend utility.
use crate::Result;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::iter::FromIterator;

/// Checks the given values of a vector against an enumerated set of accepted values.
/// Optionally, check for duplicate items as well.
pub fn validate_input_list<'a, S, T, U>(
    input: T,
    allowed_values: Option<U>,
    allow_duplicates: bool,
    duplicate_error_customizer: Option<&dyn Fn(&S, usize, usize) -> String>,
    value_error_customizer: Option<&dyn Fn(&Vec<&&S>) -> String>,
) -> Result<()>
where
    S: 'a + std::cmp::Eq + std::hash::Hash + std::fmt::Display,
    T: IntoIterator<Item = &'a S>,
    U: IntoIterator<Item = &'a S>,
{
    let input_vector: Vec<&S> = input.into_iter().collect();
    let mut input_hash: HashSet<&S> = HashSet::new();
    for (idx, i) in input_vector.iter().enumerate() {
        let added = input_hash.insert(i);
        if !allow_duplicates & !added {
            let f_idx = input_vector
                .iter()
                .position(|x| x == i)
                .expect(&format!("Expected element {} in input", i.to_string()));
            if let Some(f) = duplicate_error_customizer {
                bail!(&f(i, f_idx, idx));
            } else {
                bail!(
                    "Input contains duplicate value '{}' at index {}, which is not allowed in the this context (first occurrence at index {})",
                    i.to_string(),
                    idx,
                    f_idx
                );
            }
        }
    }

    if let Some(allowed) = allowed_values {
        let hash_allowed = HashSet::from_iter(allowed.into_iter());
        let diff: Vec<&&S> = input_hash.difference(&hash_allowed).into_iter().collect();
        if diff.len() > 0 {
            if let Some(f) = value_error_customizer {
                bail!(&f(&diff));
            } else {
                bail!(
                    "Input contains values which are not allowed in this context: {}",
                    diff.iter()
                        .map(|d| format!("'{}'", d))
                        .collect::<Vec<String>>()
                        .join(", ")
                );
            }
        }
    }
    Ok(())
}

pub fn str_to_bool(s: &str) -> Result<bool> {
    match s {
        "true" | "True" => Ok(true),
        "false" | "False" => Ok(false),
        _ => bail!("Could not convert string {} to boolean value", s),
    }
}

pub fn str_from_byte_array(bytes: &[u8]) -> Result<String> {
    let mut s = String::new();
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    Ok(s)
}

pub fn resolve_os_str(s: &OsStr) -> Result<String> {
    Ok(s.to_os_string().into_string()?)
}

/// For convenience in converting to/from configs, allow bytes to be converted
/// to a string representation of byte values (NOT characters themselves).
/// This allows for invalid UTF-8 characters to still be encoded in a string.
pub fn bytes_from_str_of_bytes(s: &str) -> Result<Vec<u8>> {
    let mut bytes = vec![];
    for chrs in s.chars().collect::<Vec<char>>().chunks(2) {
        let s = format!("{}{}", chrs[0], chrs[1]);
        bytes.push(
            u8::from_str_radix(&s, 16)
                .expect(&format!("Could not interpret byte {} as a hex value", s)),
        );
    }
    Ok(bytes)
}

pub fn unsorted_dedup<T: Eq + std::hash::Hash + Clone>(v: &mut Vec<T>) {
    let mut uniques = std::collections::HashSet::new();
    v.retain(|e| uniques.insert(e.clone()));
}
