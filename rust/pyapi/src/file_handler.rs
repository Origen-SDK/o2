use origen::core::file_handler::FileHandler as CoreFileHandler;
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

pub fn define(m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(file_handler))?;
    Ok(())
}

/// Returns a file handler object (iterable) for consuming the file arguments
/// given to the CLI
#[pyfunction]
fn file_handler() -> PyResult<FileHandler> {
    Ok(FileHandler::new())
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct FileHandler {
    inner: CoreFileHandler,
}

impl FileHandler {
    pub fn new() -> FileHandler {
        FileHandler {
            inner: CoreFileHandler::new(),
        }
    }
}

#[pymethods]
impl FileHandler {
    /// Entry point for the Python process to supply Rust with the file arguments
    /// for the current command that were collected from the CLI, should only be called
    /// once at the start of an Origen invocation
    fn init(&mut self, files: Vec<String>) -> PyResult<()> {
        self.inner.init(files)?;
        Ok(())
    }

    fn len(&self) -> PyResult<usize> {
        Ok(self.inner.len())
    }

    fn __len__(&self) -> PyResult<usize> {
        Ok(self.inner.len())
    }

    fn __iter__(slf: PyRefMut<Self>) -> PyResult<FileHandler> {
        Ok(slf.clone())
    }

    fn __next__(mut slf: PyRefMut<Self>) -> PyResult<Option<String>> {
        match slf.inner.next() {
            Some(x) => Ok(Some(x.display().to_string())),
            None => Ok(None),
        }
    }
}
