use crate::{Context, Result};
use regex::Regex;
use std::fs::File;
use std::io::{self, prelude::*, BufReader};
use std::path::{Path, PathBuf};

pub trait Differ {
    /// Returns true if diffs are found between the contained files based on the differ's current
    /// configuration.
    /// An error will be returned if any of the files doesn't exist or if there is some
    /// other problem with reading them.
    fn has_diffs(&mut self) -> Result<bool>;
}

/// A utility for diffing two different files, with the ability to ignore code in comments,
/// and to specify character strings within the file to suspend and resume diffing.
/// Blank lines will be ignored by default.
///
/// # Example
///
/// ```
/// use std::io::Write;
/// # use tempfile::NamedTempFile;
/// use origen_metal::utils::differ::{ASCIIDiffer, Differ};
///
/// # let mut file_a = NamedTempFile::new().unwrap();
/// # let mut file_b = NamedTempFile::new().unwrap();
/// let _ = writeln!(file_a, "This part is the same // But this is different");
/// let _ = writeln!(file_b, "This part is the same // But this is not");
/// let _ = writeln!(file_b, "");   // An extra blank line in file B
///
/// let mut differ = ASCIIDiffer::new(file_a.path(), file_b.path());
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
pub struct ASCIIDiffer {
    file_a: PathBuf,
    file_b: PathBuf,
    pub ignore_blank_lines: bool,
    /// The tuple is the comment identifier, the regex to find a full line comment and a
    /// regex to find a partial line comment and capture the non-commented portion
    comment_chars: Vec<(String, Regex, Regex)>,
    /// Pairs of regexs that define when to stop diffing and then when to resume
    suspends: Vec<Vec<Regex>>,
}

impl Differ for ASCIIDiffer {
    fn has_diffs(&mut self) -> Result<bool> {
        self.run()
    }
}

impl ASCIIDiffer {
    pub fn new(file_a: &Path, file_b: &Path) -> Self {
        ASCIIDiffer {
            file_a: file_a.to_path_buf(),
            file_b: file_b.to_path_buf(),
            ignore_blank_lines: true,
            comment_chars: vec![],
            suspends: vec![],
        }
    }

    /// Set the diff to suspend when the given pattern string is found on a line and remain
    /// suspended until the given resume pattern is found
    pub fn ignore_block(&mut self, suspend_on: &str, resume_on: &str) -> Result<()> {
        let suspend_pattern = regex::escape(suspend_on);
        let resume_pattern = regex::escape(resume_on);
        // The chars after a suspend also need to be captured since we need to check those
        // in case the resume occurs later on the same line
        let suspend_re = Regex::new(&format!(r#"(.*){}(.*)"#, suspend_pattern))?;
        // The chars before a resume can be safely thrown away, no need to capture those
        let resume_re = Regex::new(&format!(r#".*{}(.*)"#, resume_pattern))?;
        self.suspends.push(vec![suspend_re, resume_re]);
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
    pub fn run(&mut self) -> Result<bool> {
        let fa = File::open(&self.file_a)
            .context(format!("When opening '{}'", &self.file_a.display()))?;
        let mut fa = BufReader::new(fa).lines();

        let fb = File::open(&self.file_b)
            .context(format!("When opening '{}'", &self.file_b.display()))?;
        let mut fb = BufReader::new(fb).lines();

        let mut a_suspended = false;
        let mut b_suspended = false;
        let mut a_suspend_index: usize = 0;
        let mut b_suspend_index: usize = 0;

        loop {
            let a = self.get_next_line(&mut fa, a_suspended, a_suspend_index)?;
            let b = self.get_next_line(&mut fb, b_suspended, b_suspend_index)?;
            a_suspended = a.1;
            b_suspended = a.1;
            a_suspend_index = a.2;
            b_suspend_index = b.2;

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
        mut suspend_index: usize,
    ) -> Result<(Option<String>, bool, usize)> {
        let mut line;

        line = lines.next();
        while line.is_some() {
            let raw_ln = line.unwrap()?;
            let mut ln = raw_ln.trim_end();
            let old_suspended = suspended;
            // Returns the portion of the line before/after the suspend/resume key if applicable
            let r = self.update_suspended(ln, suspended, suspend_index);
            suspended = r.0;
            suspend_index = r.1;
            if (!old_suspended && !suspended)
                || (!old_suspended && suspended && r.2.is_some())
                || (old_suspended && !suspended && r.2.is_some())
            {
                let v;
                if r.2.is_some() {
                    v = r.2.unwrap();
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
                        return Ok((Some(next_line), suspended, suspend_index));
                    }
                }
            }
            line = lines.next();
        }
        Ok((None, suspended, suspend_index))
    }

    fn update_suspended(
        &self,
        line: &str,
        currently_suspended: bool,
        current_suspend_index: usize,
    ) -> (bool, usize, Option<String>) {
        let mut suspended;
        let mut index;
        // Parts of a line which are outside of a suspend block and should be sent back from here
        // to be included in the diff
        let mut remainder;
        // Parts of the line which may contain additional suspend/resumes and still need to be checked
        let mut to_be_checked;
        let r = self._update_suspended(line, currently_suspended, current_suspend_index);
        suspended = r.0;
        index = r.1;
        remainder = r.2;
        to_be_checked = r.3;

        while to_be_checked.is_some() {
            let r = self._update_suspended(&to_be_checked.unwrap(), suspended, index);
            suspended = r.0;
            index = r.1;
            to_be_checked = r.3;
            if let Some(rem) = &remainder {
                if let Some(new_rem) = &r.2 {
                    remainder = Some(format!("{}{}", rem, new_rem));
                }
            } else {
                remainder = r.2;
            }
        }
        (suspended, index, remainder)
    }

    fn _update_suspended(
        &self,
        line: &str,
        currently_suspended: bool,
        current_suspend_index: usize,
    ) -> (bool, usize, Option<String>, Option<String>) {
        if currently_suspended {
            let r = self.check_for_resume(line, currently_suspended, current_suspend_index);
            (r.0, r.1, None, r.2)
        } else {
            self.check_for_suspend(line, currently_suspended, current_suspend_index)
        }
    }

    fn check_for_suspend(
        &self,
        line: &str,
        current_suspended: bool,
        current_suspend_index: usize,
    ) -> (bool, usize, Option<String>, Option<String>) {
        let mut potential_match = (
            current_suspended,
            current_suspend_index,
            Some(line.to_string()),
            None,
        );
        for (i, suspend) in self.suspends.iter().enumerate() {
            let re = &suspend[0];
            if re.is_match(line) {
                if let Some(captures) = re.captures(line) {
                    if let Some(c1) = captures.get(1) {
                        // If this suspend leaves a portion remaining from the start of the line, then consider it a better
                        // match if the remaining portion is smaller than what would be left by another suspend rule.
                        // This stuff comes into play for the corner case where suspend chars are embedded within another
                        // suspend block.
                        if c1.as_str().len() < potential_match.2.as_ref().unwrap().len() {
                            if let Some(c2) = captures.get(2) {
                                potential_match = (
                                    true,
                                    i,
                                    Some(c1.as_str().to_string()),
                                    Some(c2.as_str().to_string()),
                                );
                            } else {
                                potential_match = (true, i, Some(c1.as_str().to_string()), None);
                            }
                        }
                    } else if let Some(c2) = captures.get(2) {
                        return (true, i, None, Some(c2.as_str().to_string()));
                    } else {
                        return (true, i, None, None);
                    }
                } else {
                    return (true, i, None, None);
                }
            }
        }
        potential_match
    }

    fn check_for_resume(
        &self,
        line: &str,
        current_suspended: bool,
        current_suspend_index: usize,
    ) -> (bool, usize, Option<String>) {
        let re = &self.suspends[current_suspend_index][1];
        if re.is_match(line) {
            if let Some(captures) = re.captures(line) {
                return (false, 0, Some(captures[1].to_string()));
            } else {
                return (false, 0, None);
            }
        }
        (current_suspended, current_suspend_index, None)
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::differ::{ASCIIDiffer, Differ};
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

        let mut differ = ASCIIDiffer::new(file_a.path(), file_b.path());

        assert_eq!(differ.has_diffs().unwrap(), true);

        let _ = differ.ignore_block("<START>", "<STOP>");

        assert_eq!(differ.has_diffs().unwrap(), false);
    }

    #[test]
    fn ignore_block_with_only_one_file_containing_an_ignore_portion() {
        let mut file_a = NamedTempFile::new().unwrap();
        let mut file_b = NamedTempFile::new().unwrap();
        let _ = writeln!(
            file_a,
            "1. This part is the same
             2. This part <START>Ignore me<STOP>is the same
             3. This part is the same"
        );
        let _ = writeln!(
            file_b,
            "1. This part is the same
             2. This part is the same
             3. This part is the same"
        );

        let mut differ = ASCIIDiffer::new(file_a.path(), file_b.path());

        assert_eq!(differ.has_diffs().unwrap(), true);

        let _ = differ.ignore_block("<START>", "<STOP>");

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

        let mut differ = ASCIIDiffer::new(file_a.path(), file_b.path());

        assert_eq!(differ.has_diffs().unwrap(), true);

        let _ = differ.ignore_block("<START>", "<STOP>");

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

        let mut differ = ASCIIDiffer::new(file_a.path(), file_b.path());

        assert_eq!(differ.has_diffs().unwrap(), true);

        let _ = differ.ignore_block("<START>", "<STOP>");

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

        let mut differ = ASCIIDiffer::new(file_a.path(), file_b.path());

        assert_eq!(differ.has_diffs().unwrap(), false);

        differ.ignore_blank_lines = false;

        assert_eq!(differ.has_diffs().unwrap(), true);
    }

    #[test]
    fn ignore_multiple_blocks_works() {
        let mut file_a = NamedTempFile::new().unwrap();
        let mut file_b = NamedTempFile::new().unwrap();
        let _ = writeln!(
            file_a,
            "This part is the same
             This part /* A comment diff  */is the same
             This part [[ Another comment diff /* An embedded one for luck */]]is the same"
        );
        let _ = writeln!(
            file_b,
            "This part is the same
             This part is the same
             This part is the same"
        );

        let mut differ = ASCIIDiffer::new(file_a.path(), file_b.path());

        assert_eq!(differ.has_diffs().unwrap(), true);

        let _ = differ.ignore_block("/*", "*/");
        let _ = differ.ignore_block("[[", "]]");

        assert_eq!(differ.has_diffs().unwrap(), false);
    }
}
