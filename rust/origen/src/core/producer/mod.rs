pub mod job;

use crate::Result;
use job::Job;
use std::path::Path;

/// The producer is a singleton instantiated as origen::PRODUCER, it provides static storage and
/// state tracking for all jobs created by an origen command invocation (e.g. origen g blah).
/// A job is created for each source file provided by the user.
pub struct Producer {
    pub jobs: Vec<Job>,
    pub running: Vec<usize>,
    pub completed: Vec<usize>,
    pub queued: Vec<usize>,
    /// Will be set to true if any flow source files (with Flow() blocks) are encountered
    /// during a generation run
    pub flow_generated: bool,
}

impl Producer {
    pub fn new() -> Self {
        Self {
            jobs: vec![],
            running: vec![],
            completed: vec![],
            queued: vec![],
            flow_generated: false,
        }
    }

    /// Creates a new generate job (for either a pattern or a flow)
    pub fn create_job(&mut self, command: &str, file: Option<&Path>) -> Result<&Job> {
        let id = self.jobs.len();
        let mut j = Job {
            command: command.to_string(),
            results: None,
            id: id,
            files: vec![],
        };
        if let Some(f) = file {
            j.files.push(f.to_path_buf());
        }
        self.jobs.push(j);
        Ok(&self.jobs[id])
    }

    pub fn current_job(&self) -> Option<&Job> {
        self.jobs.last()
    }

    pub fn current_job_mut(&mut self) -> Option<&mut Job> {
        self.jobs.last_mut()
    }
}

pub enum OptionType {
    OptionString(String),
    OptionBool(bool),
    OptionNum(usize),
    OptionList(Vec<String>),
}

pub enum JobType {
    Generate,
    Program,
    Misc,
}
