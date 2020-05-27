use crate::Result;
use regex::Regex;
use std::fs::File;
use std::io::{self, prelude::*, BufReader};
use std::path::{Path, PathBuf};

/// A utility for diffing two different files, with the ability to ignore code in comments,
/// and to specify character strings within the file to suspend and resume diffing.
/// By default blank lines will be ignored.
///
/// # Example
///
/// ```
/// use std::io::Write;
/// use tempfile::NamedTempFile;
/// use origen::utility::differ::Differ;
///
/// let mut file_a = NamedTempFile::new().unwrap();
/// let mut file_b = NamedTempFile::new().unwrap();
///
/// let _ = writeln!(file_a, "This part is the same // But this is different");
/// let _ = writeln!(file_b, "This part is the same // But this is not");
/// let _ = writeln!(file_b, "");   // An extra blank line in file B
///
/// let mut differ = Differ::new(file_a.path(), file_b.path());
///
/// assert!(differ.has_diffs().unwrap(), true);
///
/// differ.ignore_comments("//");
///
/// assert!(differ.has_diffs().unwrap(), false);
///
/// differ.ignore_blank_lines = false;
///
/// assert!(differ.has_diffs().unwrap(), true);
/// ```
pub struct Differ {
    file_a: PathBuf,
    file_b: PathBuf,
    pub ignore_blank_lines: bool,
    /// The tuple is the comment identifier, the regex to find a full line comment and a
    /// regex to find a partial line comment and capture the non-commented portion
    comment_chars: Vec<(String, Regex, Regex)>,
    /// Suspend diffing when this string is encountered
    suspend_on: Option<Regex>,
    /// Resume diffing when this string is encountered
    resume_on: Option<Regex>,
}

impl Differ {
    pub fn new(file_a: &Path, file_b: &Path) -> Self {
        Differ {
            file_a: file_a.to_path_buf(),
            file_b: file_b.to_path_buf(),
            ignore_blank_lines: true,
            comment_chars: vec![],
            suspend_on: None,
            resume_on: None,
        }
    }

    /// Set the diff to suspend when the given pattern string is found on a line
    pub fn suspend_on(&mut self, pattern: &str) -> Result<()> {
        let pattern = regex::escape(pattern);
        self.suspend_on = Some(Regex::new(&format!(r#"{}"#, pattern))?);
        Ok(())
    }

    /// Set the diff to resume when the given pattern string is found on a line
    pub fn resume_on(&mut self, pattern: &str) -> Result<()> {
        let pattern = regex::escape(pattern);
        self.resume_on = Some(Regex::new(&format!(r#"{}"#, pattern))?);
        Ok(())
    }

    /// Ignore comments as defined by the given command char(s), this can be called
    /// multiple times to register multiple comment character strings
    pub fn ignore_comments(&mut self, chars: &str) -> Result<()> {
        self.comment_chars.push((
            chars.to_string(),
            Regex::new(&format!(r#"^\s*{}"#, chars))?,
            Regex::new(&format!(r#"(.*)\s*{}.*"#, chars))?,
        ));
        Ok(())
    }

    /// Returns true if diffs are found between the two files based on the differ's current
    /// configuration.
    /// An error will be returned if either of the files doesn't exist or if there is some
    /// other problem with reading them.
    pub fn has_diffs(&self) -> Result<bool> {
        let fa = File::open(&self.file_a)?;
        let mut fa = BufReader::new(fa).lines();

        let fb = File::open(&self.file_b)?;
        let mut fb = BufReader::new(fb).lines();

        loop {
            let a = self.get_next_line(&mut fa)?;
            let b = self.get_next_line(&mut fb)?;
            if a.is_none() && b.is_none() {
                return Ok(false);
            } else if a.is_none() || b.is_none() {
                return Ok(true);
            } else if a.unwrap() != b.unwrap() {
                return Ok(true);
            }
        }
    }

    fn get_next_line(&self, lines: &mut io::Lines<BufReader<File>>) -> Result<Option<String>> {
        let mut line;
        let mut suspended = true;

        line = lines.next();
        while line.is_some() {
            let raw_ln = line.unwrap()?;
            let ln = raw_ln.trim();
            suspended = self.update_suspended(ln, suspended);
            if !suspended {
                let mut next_line: Option<String> = None;
                if !self.comment_chars.is_empty() {
                    for (_char, re1, re2) in &self.comment_chars {
                        // If not a full line comment
                        if !re1.is_match(ln) {
                            // If contains a comment midway through the line, the return the portion
                            // of the line before it
                            if let Some(matches) = re2.captures(ln) {
                                next_line = Some(matches[1].trim().to_string());
                                break;
                            } else {
                                next_line = Some(ln.to_string());
                                break;
                            }
                        }
                    }
                } else {
                    next_line = Some(ln.to_string());
                }
                if let Some(next_line) = next_line {
                    if !self.ignore_blank_lines || next_line != "" {
                        return Ok(Some(next_line));
                    }
                }
            }
            line = lines.next();
        }
        Ok(None)
    }

    fn update_suspended(&self, line: &str, current_suspended: bool) -> bool {
        if let Some(re) = &self.suspend_on {
            if re.is_match(line) {
                return true;
            }
        }
        if let Some(re) = &self.resume_on {
            if re.is_match(line) {
                return false;
            }
        }
        current_suspended
    }
}
