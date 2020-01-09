use origen::DUT;
use pyo3::prelude::*;
use origen::error::Error;

pub trait DictLikeAPI {
  fn model_id(&self) -> usize;
  fn lookup_key(&self) -> &str;
  fn new_pyitem(&self, py: Python, name: &str, model_id: usize) -> Result<PyObject, Error>;

  fn keys(&self) -> PyResult<Vec<String>> {
    let dut = DUT.lock().unwrap();
    let model = dut.get_model(self.model_id())?;
    let names = model.lookup(self.lookup_key())?;
    Ok(names.iter().map(|(k, _)| k.clone()).collect())
  }

  fn values(&self) -> PyResult<Vec<PyObject>> {
    let mut dut = DUT.lock().unwrap();
    let model = dut.get_mut_model(self.model_id())?;
    let items = model.lookup(self.lookup_key())?;

    let gil = Python::acquire_gil();
    let py = gil.python();
    let mut v: Vec<PyObject> = Vec::new();
    for (n, _item) in items {
        v.push(self.new_pyitem(py, n, self.model_id())?);
    }
    Ok(v)
  }

  fn items(&self) -> PyResult<Vec<(String, PyObject)>> {
    let mut dut = DUT.lock().unwrap();
    let model = dut.get_mut_model(self.model_id())?;
    let items = model.lookup(self.lookup_key())?;

    let gil = Python::acquire_gil();
    let py = gil.python();
    let mut _items: Vec<(String, PyObject)> = Vec::new();
    for (n, _item) in items.iter() {
        _items.push((
            n.clone(),
            self.new_pyitem(py, &n, self.model_id())?
        ));
    }
    Ok(_items)
  }

  fn get(&self, name: &str) -> PyResult<PyObject> {
    let mut dut = DUT.lock().unwrap();
    let model = dut.get_mut_model(self.model_id())?;
    let item = model.lookup(self.lookup_key())?.get(name);

    let gil = Python::acquire_gil();
    let py = gil.python();
    match item {
        Some(_item) => Ok(self.new_pyitem(py, name, self.model_id())?),
        None => Ok(py.None())
      }
  }

  // Functions for PyMappingProtocol
  fn __getitem__(&self, name: &str) -> PyResult<PyObject> {
    let mut dut = DUT.lock().unwrap();
    let model = dut.get_mut_model(self.model_id())?;
    let item = model.lookup(self.lookup_key())?.get(name);

    let gil = Python::acquire_gil();
    let py = gil.python();
    match item {
        Some(_item) => Ok(self.new_pyitem(py, name, self.model_id())?),
        None => Err(pyo3::exceptions::KeyError::py_err(format!(
            "No pin or pin alias found for {}",
            name
        ))),
    }
  }

  fn __len__(&self) -> PyResult<usize> {
    let mut dut = DUT.lock().unwrap();
    let model = dut.get_mut_model(self.model_id())?;
    let items = model.lookup(self.lookup_key())?;
    Ok(items.len())
  }

  fn __iter__(&self) -> PyResult<DictLikeIter> {
    let dut = DUT.lock().unwrap();
    let model = dut.get_model(self.model_id())?;
    let items = model.lookup(self.lookup_key())?;
    Ok(DictLikeIter {
        keys: items.iter().map(|(s, _)| s.clone()).collect(),
        i: 0,
    })
  }
}

#[pyclass]
pub struct DictLikeIter {
    keys: Vec<String>,
    i: usize,
}

#[pyproto]
impl pyo3::class::iter::PyIterProtocol for DictLikeIter {
    fn __iter__(slf: PyRefMut<Self>) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(slf.to_object(py))
    }

    /// The Iterator will be created with an index starting at 0 and the pin names at the time of its creation.
    /// For each call to 'next', we'll create a pin object with the next value in the list, or None, if no more keys are available.
    /// Note: this means that the iterator can become stale if the PinContainer is changed. This can happen if the iterator is stored from Python code
    ///  directly. E.g.: i = dut.pins.__iter__() => iterator with the pin names at the time of creation,
    /// Todo: Fix the above using iterators. My Rust skills aren't there yet though... - Coreyeng
    fn __next__(mut slf: PyRefMut<Self>) -> PyResult<Option<String>> {
        if slf.i >= slf.keys.len() {
            return Ok(None);
        }
        let name = slf.keys[slf.i].clone();
        slf.i += 1;
        Ok(Some(name))
    }
}
