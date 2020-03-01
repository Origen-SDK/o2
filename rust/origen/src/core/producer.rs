use crate::error::Error;
use std::path::{PathBuf};

pub struct Producer {
  pub jobs: Vec<Job>,
  pub running: Vec<usize>,
  pub completed: Vec<usize>,
  pub queued: Vec<usize>,
}

impl Producer {
  pub fn new() -> Self {
    Self {
      jobs: vec!(),
      running: vec!(),
      completed: vec!(),
      queued: vec!(),
    }
  }

  pub fn create_pattern_job(&mut self, command: PathBuf) -> Result<&Job, Error> {
    let id = self.jobs.len();
    let j = Job {
      command: command,
      results: None,
      id: id,
    };
    self.jobs.push(j);
    Ok(&self.jobs[id])
  }
}

pub struct Job {
  pub command: PathBuf,
  pub results: Option<String>,
  pub id: usize,
}
