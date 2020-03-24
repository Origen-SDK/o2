use crate::error::Error;
use std::path::{PathBuf};
use indexmap::IndexMap;

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

pub struct Job {
  // /// The raw input string, as given on the command line.
  // pub __command__: String,

  /// The command itself. E.g., 'generate', 'program', etc.
  pub command: String,
  //pub job_type: JobType,

  // /// Command line options.
  // pub options: IndexMap<String, OptionType>,

  // /// Parameters given along with the input.
  // pub parameters: IndexMap<String, Vec<String>>,

  pub results: Option<String>,
  pub id: usize,

  // /// The output files which this job is/has/will render.
  // /// Sidenote: this is an IndexMap so filenames can be retrieved either by an index or by an identifier; the latter of which is easier
  // ///  to follow when a single job may produce output in multiple phases.
  // pub output_filenames: Vec<PathBuf>,

  // /// The ouptut directory in which the created files will be created.
  // /// For patterns, this will just be the application's output directory and tester name. For programs, or other collateral,
  // ///  this may also contain the job or source name and all output files will be generated into that subdirectory.
  // pub job_output_root: Option<PathBuf>,
}

// impl Job {
//   /// Creates a new pattern job with inputs only relevant to pattern rendering.
//   pub fn new_pattern_job() -> Self {
//     Self {
//       __command__: "",
//       command: "",
//       job_type: JobType::Pattern,
//       options: IndexMap::new(),
//       parameters::IndexMap::new(),

//       results: Option::None,
//       id: 0,

//       // ...
//       output_filenames: vec!(),
//       job_output_root: None,
//     }
//   }

  // /// Creates a new pattern job with inputs only relevant to program rendering.
  // pub fn new_program_job() -> Self {}

  // /// Creates a new job with no predetermined fields.
  // pub fn new_misc_job() -> Self {}
// }
