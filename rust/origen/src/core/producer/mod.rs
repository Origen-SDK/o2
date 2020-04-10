pub mod job;

use crate::error::Error;
use job::Job;

pub struct Producer {
    pub jobs: Vec<Job>,
    pub running: Vec<usize>,
    pub completed: Vec<usize>,
    pub queued: Vec<usize>,
}

impl Producer {
    pub fn new() -> Self {
        Self {
            jobs: vec![],
            running: vec![],
            completed: vec![],
            queued: vec![],
        }
    }

    pub fn create_pattern_job(&mut self, command: &str) -> Result<&Job, Error> {
        let id = self.jobs.len();
        let j = Job {
            command: command.to_string(),
            results: None,
            id: id,
        };
        self.jobs.push(j);
        Ok(&self.jobs[id])
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
