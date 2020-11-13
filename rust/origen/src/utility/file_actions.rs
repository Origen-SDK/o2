//! A collection of utilities for making modifications to existing files, e.g. to
//! add/remove lines from them, etc.

use crate::Result;
use regex::Regex;
use std::path::Path;

#[derive(Copy, Clone, PartialEq)]
enum Operation {
    InsertBefore,
    InsertBeforeAll,
    InsertAfter,
    InsertAfterAll,
    Prepend,
    Append,
    RemoveLine,
    RemoveLineAll,
    Replace,
    ReplaceAll,
}

/// Insert the given text into the given file immediately before the first text matched
/// by the given regular expression.
/// Returns an error if any error is encountered when trying to make the update, e.g. if the given
/// file doesn't exist.
/// Otherwise, it will return true if the text was inserted and false if not, i.e. when the given
/// regular expression did not match any line in the file.
///
/// Note: The current implementation does the insertion in memory, so this could run into issues
/// if used on a particularly large file.
pub fn insert_before(file: &Path, regex: &Regex, text: &str) -> Result<bool> {
    modify(file, Some(regex), Some(text), Operation::InsertBefore)
}

/// Like insert_before, but this function will insert the given text before every match rather than
/// just the first one
pub fn insert_before_all(file: &Path, regex: &Regex, text: &str) -> Result<bool> {
    modify(file, Some(regex), Some(text), Operation::InsertBeforeAll)
}

/// Insert the given text into the given file immediately after the first text matched
/// by the given regular expression.
/// Returns an error if any error is encountered when trying to make the update, e.g. if the given
/// file doesn't exist.
/// Otherwise, it will return true if the text was inserted and false if not, i.e. when the given
/// regular expression did not match any line in the file.
///
/// Note: The current implementation does the insertion in memory, so this could run into issues
/// if used on a particularly large file.
pub fn insert_after(file: &Path, regex: &Regex, text: &str) -> Result<bool> {
    modify(file, Some(regex), Some(text), Operation::InsertAfter)
}

/// Like insert_after, but this function will insert the given text after every match rather than
/// just the first one
pub fn insert_after_all(file: &Path, regex: &Regex, text: &str) -> Result<bool> {
    modify(file, Some(regex), Some(text), Operation::InsertBeforeAll)
}

/// Prepend the given text at the start of the given file.
/// Returns an error if any error is encountered when trying to make the update, e.g. if the given
/// file doesn't exist.
pub fn prepend(file: &Path, text: &str) -> Result<()> {
    modify(file, None, Some(text), Operation::Prepend)?;
    Ok(())
}

/// Append the given text at the end of the given file.
/// Returns an error if any error is encountered when trying to make the update, e.g. if the given
/// file doesn't exist.
pub fn append(file: &Path, text: &str) -> Result<()> {
    modify(file, None, Some(text), Operation::Append)?;
    Ok(())
}

/// Remove the first line in the given file that matches the given regex. Returns true if the file
/// was modified.
pub fn remove_line(file: &Path, regex: &Regex) -> Result<bool> {
    modify(file, Some(regex), None, Operation::RemoveLine)
}

/// Like remove_line, but removes all matching lines in the given file.
pub fn remove_line_all(file: &Path, regex: &Regex) -> Result<bool> {
    modify(file, Some(regex), None, Operation::RemoveLineAll)
}

/// Returns Ok(true) if any line in the given file matches the given regex
pub fn contains(file: &Path, regex: &Regex) -> Result<bool> {
    let content = std::fs::read_to_string(file)?;
    for line in content.lines() {
        if regex.is_match(line) {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Replaces the first occurrence of the given regex with the given text, note that the regex is evaluated
/// on a per line basis are therefore cannot span multiple lines
pub fn replace(file: &Path, regex: &Regex, text: &str) -> Result<bool> {
    modify(file, Some(regex), Some(text), Operation::Replace)
}

/// Like replace, but replaces occurences of the given regex on all lines in the file
pub fn replace_all(file: &Path, regex: &Regex, text: &str) -> Result<bool> {
    modify(file, Some(regex), Some(text), Operation::ReplaceAll)
}

fn modify(
    file: &Path,
    regex: Option<&Regex>,
    text: Option<&str>,
    operation: Operation,
) -> Result<bool> {
    let mut output = String::new();
    let orig = std::fs::read_to_string(file)?;
    let mut modified = false;

    let do_all = match operation {
        Operation::InsertAfterAll
        | Operation::InsertBeforeAll
        | Operation::RemoveLineAll
        | Operation::ReplaceAll => true,
        _ => false,
    };

    for line in orig.lines() {
        if modified && !do_all {
            output += line;
        } else {
            match operation {
                Operation::Prepend => {
                    output += text.unwrap();
                    output += line;
                    modified = true;
                }
                Operation::Append => {
                    modified = true;
                }
                _ => {
                    if regex.unwrap().is_match(line) {
                        let m = regex.unwrap().find(line).unwrap();
                        match operation {
                            Operation::InsertBefore | Operation::InsertBeforeAll => {
                                let (first, second) = line.split_at(m.start());
                                output += &format!("{}{}{}", first, text.unwrap(), second);
                            }
                            Operation::InsertAfter | Operation::InsertAfterAll => {
                                let (first, second) = line.split_at(m.end());
                                output += &format!("{}{}{}", first, text.unwrap(), second);
                            }
                            Operation::RemoveLine | Operation::RemoveLineAll => {
                                continue;
                            }
                            Operation::Replace => {
                                output += &regex.unwrap().replace(line, text.unwrap());
                            }
                            Operation::ReplaceAll => {
                                output += &regex.unwrap().replace_all(line, text.unwrap());
                            }
                            _ => unreachable!(),
                        }
                        modified = true;
                    } else {
                        output += line;
                    }
                }
            }
        }
        output += "\n";
    }
    if operation == Operation::Append {
        output += text.unwrap();
    }

    std::fs::write(file, output)?;

    Ok(modified)
}
