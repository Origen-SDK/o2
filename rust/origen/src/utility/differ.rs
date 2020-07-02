use crate::Result;
use regex::Regex;
use std::fs::File;
use std::io::{self, prelude::*, BufReader};
use std::path::{Path, PathBuf};

/// A utility for diffing two different files, with the ability to ignore code in comments,
/// and to specify character strings within the file to suspend and resume diffing.
/// Blank lines will be ignored by default.
///
/// # Example
///
/// ```
/// use std::io::Write;
/// # use tempfile::NamedTempFile;
/// use origen::utility::differ::Differ;
///
/// # let mut file_a = NamedTempFile::new().unwrap();
/// # let mut file_b = NamedTempFile::new().unwrap();
/// let _ = writeln!(file_a, "This part is the same // But this is different");
/// let _ = writeln!(file_b, "This part is the same // But this is not");
/// let _ = writeln!(file_b, "");   // An extra blank line in file B
///
/// let mut differ = Differ::new(file_a.path(), file_b.path());
///
/// assert_eq!(differ.has_diffs().unwrap(), true);
///
/// differ.ignore_comments("//").expect("Valid comment char");
///
/// assert_eq!(differ.has_diffs().unwrap(), false);
///
/// differ.ignore_blank_lines = false;
///
/// assert_eq!(differ.has_diffs().unwrap(), true);
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
        self.suspend_on = Some(Regex::new(&format!(r#"(.*){}.*"#, pattern))?);
        Ok(())
    }

    /// Set the diff to resume when the given pattern string is found on a line
    pub fn resume_on(&mut self, pattern: &str) -> Result<()> {
        let pattern = regex::escape(pattern);
        self.resume_on = Some(Regex::new(&format!(r#".*{}(.*)"#, pattern))?);
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
    pub fn has_diffs(&mut self) -> Result<bool> {
        let fa = File::open(&self.file_a)?;
        let mut fa = BufReader::new(fa).lines();

        let fb = File::open(&self.file_b)?;
        let mut fb = BufReader::new(fb).lines();

        let mut a_suspended = false;
        let mut b_suspended = false;

        loop {
            let a = self.get_next_line(&mut fa, a_suspended)?;
            let b = self.get_next_line(&mut fb, b_suspended)?;
            a_suspended = a.1;
            b_suspended = a.1;

            if a.0.is_none() && b.0.is_none() {
                return Ok(false);
            } else if a.0.is_none() || b.0.is_none() {
                return Ok(true);
            } else if a.0.unwrap() != b.0.unwrap() {
                return Ok(true);
            }
        }
    }

    fn get_next_line(
        &self,
        lines: &mut io::Lines<BufReader<File>>,
        mut suspended: bool,
    ) -> Result<(Option<String>, bool)> {
        let mut line;

        line = lines.next();
        while line.is_some() {
            let raw_ln = line.unwrap()?;
            let mut ln = raw_ln.trim_end();
            let old_suspended = suspended;
            // Returns the portion of the line before/after the suspend/resume key if applicable
            let r = self.update_suspended(ln, suspended);
            suspended = r.0;
            if (!old_suspended && !suspended)
                || (!old_suspended && suspended && r.1.is_some())
                || (old_suspended && !suspended && r.1.is_some())
            {
                let v;
                if r.1.is_some() {
                    v = r.1.unwrap();
                    ln = &v;
                }
                let mut next_line: Option<String> = None;
                if !self.comment_chars.is_empty() {
                    for (_char, re1, re2) in &self.comment_chars {
                        // If not a full line comment
                        if !re1.is_match(ln) {
                            // If contains a comment midway through the line, the return the portion
                            // of the line before it
                            if let Some(captures) = re2.captures(ln) {
                                next_line = Some(captures[1].trim_end().to_string());
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
                        return Ok((Some(next_line), suspended));
                    }
                }
            }
            line = lines.next();
        }
        Ok((None, suspended))
    }

    fn update_suspended(&self, line: &str, current_suspended: bool) -> (bool, Option<String>) {
        if let Some(re) = &self.suspend_on {
            if re.is_match(line) {
                if let Some(captures) = re.captures(line) {
                    return (true, Some(captures[1].to_string()));
                } else {
                    return (true, None);
                }
            }
        }
        if let Some(re) = &self.resume_on {
            if re.is_match(line) {
                if let Some(captures) = re.captures(line) {
                    return (false, Some(captures[1].to_string()));
                } else {
                    return (false, None);
                }
            }
        }
        (current_suspended, None)
    }
}

#[cfg(test)]
mod tests {
    use crate::utility::differ::Differ;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn basic_suspend_works() {
        let mut file_a = NamedTempFile::new().unwrap();
        let mut file_b = NamedTempFile::new().unwrap();
        let _ = writeln!(
            file_a,
            "This part is the same
             until here<START> dfodisgsg
             soemghow dosghsg
             iewg<STOP>
             and now we are back"
        );
        let _ = writeln!(
            file_b,
            "This part is the same
             until here<START>
             some diff
             <STOP>
             and now we are back"
        );

        let mut differ = Differ::new(file_a.path(), file_b.path());

        assert_eq!(differ.has_diffs().unwrap(), true);

        let _ = differ.suspend_on("<START>");
        let _ = differ.resume_on("<STOP>");

        assert_eq!(differ.has_diffs().unwrap(), false);
    }

    #[test]
    fn diff_pre_suspend_works() {
        let mut file_a = NamedTempFile::new().unwrap();
        let mut file_b = NamedTempFile::new().unwrap();
        let _ = writeln!(
            file_a,
            "This part is the same
             until here is a pre diff<START> dfodisgsg
             soemghow dosghsg
             iewg<STOP>
             and now we are back"
        );
        let _ = writeln!(
            file_b,
            "This part is the same
             until here<START>
             some diff
             <STOP>
             and now we are back"
        );

        let mut differ = Differ::new(file_a.path(), file_b.path());

        assert_eq!(differ.has_diffs().unwrap(), true);

        let _ = differ.suspend_on("<START>");
        let _ = differ.resume_on("<STOP>");

        assert_eq!(differ.has_diffs().unwrap(), true);
    }

    #[test]
    fn diff_post_suspend_works() {
        let mut file_a = NamedTempFile::new().unwrap();
        let mut file_b = NamedTempFile::new().unwrap();
        let _ = writeln!(
            file_a,
            "This part is the same
             until here is a pre diff<START> dfodisgsg
             soemghow dosghsg
             iewg<STOP>
             and now we are back"
        );
        let _ = writeln!(
            file_b,
            "This part is the same
             until here<START>
             some diff
             <STOP> here is a diff!
             and now we are back"
        );

        let mut differ = Differ::new(file_a.path(), file_b.path());

        assert_eq!(differ.has_diffs().unwrap(), true);

        let _ = differ.suspend_on("<START>");
        let _ = differ.resume_on("<STOP>");

        assert_eq!(differ.has_diffs().unwrap(), true);
    }

    #[test]
    fn diff_blank_lines_works() {
        let mut file_a = NamedTempFile::new().unwrap();
        let mut file_b = NamedTempFile::new().unwrap();
        let _ = writeln!(
            file_a,
            "This part is the same
             This part is the same
             This part is the same"
        );
        let _ = writeln!(
            file_b,
            "This part is the same
             This part is the same

             This part is the same"
        );

        let mut differ = Differ::new(file_a.path(), file_b.path());

        assert_eq!(differ.has_diffs().unwrap(), false);

        differ.ignore_blank_lines = false;

        assert_eq!(differ.has_diffs().unwrap(), true);
    }
}
