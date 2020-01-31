//! The file_handler is responsible for processing the file arguments supplied
//! to Origen commands from the CLI.
//! It provides methods for consumers to retreive one file at a time or all files
//! at once and seamlessly opens up lists to get to the inidividual files inside.

use crate::{Error, Result};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

lazy_static! {
    static ref FILES: Mutex<Files> = Mutex::new(Files::new());
}

#[derive(Debug)]
/// This struct is used as a singleton to store a permanent record of the file arguments
/// given to the current command and the clean list of files this resolves too (lazily evaluated)
struct Files {
    /// The file arguments as originally supplied
    items: Vec<String>,
    /// The resultant list of files from resolving any lists in the original args
    files: Vec<PathBuf>,
}

impl Files {
    fn new() -> Files {
        Files {
            items: Vec::new(),
            files: Vec::new(),
        }
    }

    /// Load a new set of file arguments
    fn init(&mut self, mut files: Vec<String>) -> Result<()> {
        self.items.clear();
        self.items.append(&mut files);
        self.files.clear();
        self.resolve()?;
        Ok(())
    }

    fn file(&mut self, index: usize) -> Option<PathBuf> {
        if index < self.files.len() {
            Some(self.files[index].clone())
        } else {
            None
        }
    }

    // Eventually this will open up list and directory args
    fn resolve(&mut self) -> Result<()> {
        for item in &self.items {
            match Path::new(item).canonicalize() {
                Ok(x) => self.files.push(x),
                Err(err) => return Err(Error::new(&format!("{} - {}", err.to_string(), item))),
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
/// This is an iterator + external API for consuming the file list
pub struct FileHandler {
    i: usize,
}

impl FileHandler {
    pub fn new() -> FileHandler {
        FileHandler { i: 0 }
    }

    pub fn init(&mut self, files: Vec<String>) -> Result<()> {
        let mut f = FILES.lock().unwrap();
        f.init(files)?;
        self.i = 0;
        Ok(())
    }

    pub fn len(&self) -> usize {
        let f = FILES.lock().unwrap();
        f.files.len()
    }
}

impl Iterator for FileHandler {
    type Item = PathBuf;

    fn next(&mut self) -> Option<PathBuf> {
        let mut f = FILES.lock().unwrap();
        let r = f.file(self.i);
        self.i += 1;
        r
    }
}
