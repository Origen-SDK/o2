//! The file_handler is responsible for processing the file arguments supplied
//! to Origen commands from the CLI.
//! It provides methods for consumers to retreive one file at a time or all files
//! at once and seamlessly opens up lists to get to the inidividual files inside.

use crate::{Error, Result};
use std::fs;
use std::fs::create_dir_all;
use std::fs::File as StdFile;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

// Trait for extending std::path::PathBuf
use path_slash::PathBufExt;

lazy_static! {
    static ref FILES: Mutex<Files> = Mutex::new(Files::new());
}

#[derive(Debug)]
pub struct File {
    pub file_obj: StdFile,
    pub path: PathBuf,
}

impl std::clone::Clone for File {
    fn clone(&self) -> Self {
        Self::open(self.path.clone()).unwrap()
    }
}

impl File {
    /// Creates or overrides the given file and panics if the creation fails
    pub fn create(f: PathBuf) -> Self {
        Self::mkdir_p(
            &f.parent()
                .expect(&format!("Unable to locate parent of {:?}", f)),
        );
        let file = StdFile::create(f.clone()).expect(&format!("Unable to create file at {:?}", f));
        Self {
            file_obj: file,
            path: f.clone(),
        }
    }

    /// Attempts to open the given file for reading, returning an error if the open fails
    pub fn open(f: PathBuf) -> Result<Self> {
        let file = StdFile::open(f.clone()).expect(&format!("Unable to open file at {:?}", f));
        Ok(Self {
            file_obj: file,
            path: f,
        })
    }

    pub fn mkdir_p(path: &Path) {
        create_dir_all(path).expect(&format!("Unable to create all directories at {:?}", path));
    }

    /// Writes content to the underlying file and panics if the write failed
    pub fn write(&mut self, content: &str) {
        self.file_obj
            .write_all(content.as_bytes())
            .expect(&format!("Error writing file {:?}", self.path));
    }

    pub fn write_ln(&mut self, content: &str) {
        self.file_obj
            .write_all(format!("{}\n", content).as_bytes())
            .expect(&format!("Error writing file {:?}", self.path));
    }
}

#[derive(Debug)]
/// This struct is used as a singleton to store a permanent record of the file arguments
/// given to the current command and the clean list of files this resolves to
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
        // Convert any / paths to \
        if cfg!(target_os = "windows") {
            files = files
                .iter()
                .map(|f| format!("{}", PathBuf::from_slash(f).display()))
                .collect();
        }
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
        let items = self.items.clone();
        for item in &items {
            match Path::new(item).canonicalize() {
                Ok(x) => {
                    if x.is_dir() {
                        self.resolve_dir(&x)?;
                    } else {
                        self.files.push(x);
                    }
                }
                Err(err) => return Err(Error::new(&format!("{} - {}", err.to_string(), item))),
            }
        }
        Ok(())
    }

    fn resolve_dir(&mut self, dir: &Path) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                self.resolve_dir(&path)?;
            } else {
                self.files.push(path);
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
