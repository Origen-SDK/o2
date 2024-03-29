use origen::DUT;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PySlice};

pub trait ListLikeAPI {
    fn item_ids(&self, dut: &std::sync::MutexGuard<origen::core::dut::Dut>) -> Vec<usize>;
    fn new_pyitem(&self, py: Python, idx: usize) -> PyResult<PyObject>;
    fn __iter__(&self) -> PyResult<ListLikeIter>;

    fn __getitem__(&self, idx: &PyAny) -> PyResult<PyObject> {
        if let Ok(slice) = idx.downcast::<PySlice>() {
            self.___getslice__(slice)
        } else {
            let i = idx.extract::<isize>()?;
            self.___getitem__(i)
        }
    }

    fn ___getitem__(&self, idx: isize) -> PyResult<PyObject> {
        let item_ids;
        {
            let dut = DUT.lock().unwrap();
            item_ids = self.item_ids(&dut);
        }
        if idx >= (item_ids.len() as isize) {
            return Err(pyo3::exceptions::PyIndexError::new_err(format!(
                "Index {} is out range of container of size {}",
                idx,
                item_ids.len()
            )));
        } else if idx.abs() > (item_ids.len() as isize) {
            return Err(pyo3::exceptions::PyIndexError::new_err(format!(
                "Index {} is out range of container of size {}",
                idx,
                item_ids.len()
            )));
        }
        let _idx;
        if idx >= 0 {
            _idx = idx as usize;
        } else {
            _idx = ((item_ids.len() as isize) + idx) as usize;
        }

        Python::with_gil(|py| {
            Ok(self.new_pyitem(py, _idx)?)
        })
    }

    fn ___getslice__(&self, slice: &PySlice) -> PyResult<PyObject> {
        let indices;
        {
            let dut = DUT.lock().unwrap();
            let item_ids = self.item_ids(&dut);
            indices = slice.indices((item_ids.len() as i32).into())?;
        }

        Python::with_gil(|py| {
            let mut rtn: Vec<PyObject> = vec![];
            let mut i = indices.start;
            if indices.step > 0 {
                while i < indices.stop {
                    rtn.push(self.new_pyitem(py, i as usize)?);
                    i += indices.step;
                }
            } else if indices.step < 0 {
                while i > indices.stop {
                    rtn.push(self.new_pyitem(py, i as usize)?);
                    i += indices.step;
                }
            }
            Ok(rtn.to_object(py))
        })
    }

    fn __len__(&self) -> PyResult<usize> {
        let dut = DUT.lock().unwrap();
        let items = self.item_ids(&dut);
        Ok(items.len())
    }
}

#[pyclass]
pub struct ListLikeIter {
    pub parent: Box<dyn ListLikeAPI + std::marker::Send>,
    pub i: usize,
}

#[pymethods]
impl ListLikeIter {
    fn __iter__(slf: PyRefMut<Self>) -> PyResult<Py<Self>> {
        Ok(slf.into())
    }

    fn __next__(mut slf: PyRefMut<Self>) -> PyResult<Option<PyObject>> {
        if slf.i >= slf.parent.__len__().unwrap() {
            return Ok(None);
        }
        slf.i += 1;
        Ok(Some(slf.parent.___getitem__((slf.i - 1) as isize).unwrap()))
    }
}

pub trait GeneralizedListLikeAPI {
    type Contained;

    fn items(&self) -> &Vec<Self::Contained>;
    fn new_pyitem(&self, py: Python, item: &Self::Contained, idx: usize) -> PyResult<PyObject>;
    // fn __iter__(&self) -> PyResult<GeneralizedListLikeIter>;

    fn __getitem__(&self, idx: &PyAny) -> PyResult<PyObject> {
        if let Ok(slice) = idx.downcast::<PySlice>() {
            self.___getslice__(slice)
        } else {
            let i = idx.extract::<isize>()?;
            self.___getitem__(i)
        }
    }

    fn ___getitem__(&self, idx: isize) -> PyResult<PyObject> {
        if idx >= (self.items().len() as isize) {
            return Err(pyo3::exceptions::PyIndexError::new_err(format!(
                "Index {} is out range of container of size {}",
                idx,
                self.items().len()
            )));
        } else if idx.abs() > (self.items().len() as isize) {
            return Err(pyo3::exceptions::PyIndexError::new_err(format!(
                "Index {} is out range of container of size {}",
                idx,
                self.items().len()
            )));
        }
        let _idx;
        if idx >= 0 {
            _idx = idx as usize;
        } else {
            _idx = ((self.items().len() as isize) + idx) as usize;
        }

        Python::with_gil(|py| {
            Ok(self.new_pyitem(py, &self.items()[_idx], _idx)?)
        })
    }

    fn ___getslice__(&self, slice: &PySlice) -> PyResult<PyObject> {
        let indices = slice.indices((self.items().len() as i32).into())?;
        Python::with_gil(|py| {
            let mut rtn: Vec<PyObject> = vec![];
            let mut i = indices.start;
            if indices.step > 0 {
                while i < indices.stop {
                    rtn.push(self.new_pyitem(py, &self.items()[i as usize], i as usize)?);
                    i += indices.step;
                }
            } else if indices.step < 0 {
                while i > indices.stop {
                    rtn.push(self.new_pyitem(py, &self.items()[i as usize], i as usize)?);
                    i += indices.step;
                }
            }
            Ok(rtn.to_object(py))
        })
    }

    fn __len__(&self) -> PyResult<usize> {
        Ok(self.items().len())
    }
}
