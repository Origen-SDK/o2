use super::super::meta::py_like_apis::list_like_api::{ListLikeAPI, ListLikeIter};
use super::pin_actions::PinActions;
use super::pin_collection::PinCollection;
use origen::DUT;
use pyo3::prelude::*;
#[allow(unused_imports)]
use pyo3::types::{PyAny, PyBytes, PyDict, PyIterator, PyList, PySlice, PyTuple};

#[pyclass]
#[derive(Clone)]
pub struct PinGroup {
    pub name: String,
    pub model_id: usize,
}

#[pymethods]
impl PinGroup {
    // Even though we're storing the name in this instance, we're going to go back to the core anyway.
    #[getter]
    fn get_name(&self) -> PyResult<String> {
        let dut = DUT.lock().unwrap();
        let grp = dut._get_pin_group(self.model_id, &self.name)?;
        Ok(grp.name.clone())
    }

    #[getter]
    fn get_data(&self) -> PyResult<u32> {
        let dut = DUT.lock().unwrap();
        Ok(dut.get_pin_group_data(self.model_id, &self.name)?)
    }

    #[setter]
    fn set_data(&self, data: u32) -> PyResult<()> {
        let mut dut = DUT.lock().unwrap();
        dut.set_pin_group_data(self.model_id, &self.name, data)?;
        Ok(())
    }

    fn set(&self, data: u32) -> PyResult<Py<Self>> {
        self.set_data(data)?;
        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(Py::new(
            py,
            Self {
                name: self.name.clone(),
                model_id: self.model_id,
            },
        )
        .unwrap())
    }

    fn with_mask(&self, mask: usize) -> PyResult<Py<Self>> {
        let mut dut = DUT.lock().unwrap();
        dut.set_pin_group_nonsticky_mask(self.model_id, &self.name, mask)?;

        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(Py::new(
            py,
            Self {
                name: self.name.clone(),
                model_id: self.model_id,
            },
        )
        .unwrap())
    }

    #[getter]
    fn get_pin_names(&self) -> PyResult<Vec<String>> {
        let dut = DUT.lock().unwrap();
        let grp = dut._get_pin_group(self.model_id, &self.name)?;

        let mut v: Vec<String> = Vec::new();
        for n in grp.pin_names.iter() {
            v.push(n.clone());
        }
        Ok(v)
    }

    // #[getter]
    // fn get_pins(&self) -> PyResult<Vec<Py<Pin>>> {
    //     let mut dut = DUT.lock().unwrap();
    //     let model = dut.get_mut_model(self.model_id)?;
    //     let grp = model._get_pin_group(&self.name)?;

    //     let gil = Python::acquire_gil();
    //     let py = gil.python();
    //     let mut v: Vec<Py<Pin>> = Vec::new();
    //     for n in grp.pin_names.iter() {
    //         v.push(
    //             Py::new(
    //                 py,
    //                 Pin {
    //                     name: String::from(n),
    //                     path: String::from(&self.path),
    //                     model_id: self.model_id,
    //                 },
    //             )
    //             .unwrap(),
    //         );
    //     }
    //     Ok(v)
    // }

    #[getter]
    fn get_pin_actions(&self) -> PyResult<PyObject> {
        let dut = DUT.lock().unwrap();
        let gil = Python::acquire_gil();
        let py = gil.python();

        let pin_actions = dut.get_pin_group_actions(self.model_id, &self.name)?;
        Ok(PinActions {
            actions: pin_actions,
        }
        .into_py(py))
    }

    fn drive(slf: PyRef<Self>, data: Option<u32>) -> PyResult<Py<Self>> {
        let mut dut = DUT.lock().unwrap();
        dut.drive_pin_group(slf.model_id, &slf.name, data, Option::None)?;
        Ok(slf.into())
    }

    fn verify(slf: PyRef<Self>, data: Option<u32>) -> PyResult<Py<Self>> {
        let mut dut = DUT.lock().unwrap();
        dut.verify_pin_group(slf.model_id, &slf.name, data, Option::None)?;
        Ok(slf.into())
    }

    fn capture(slf: PyRef<Self>) -> PyResult<Py<Self>> {
        let mut dut = DUT.lock().unwrap();
        dut.capture_pin_group(slf.model_id, &slf.name, Option::None)?;
        Ok(slf.into())
    }

    fn highz(slf: PyRef<Self>) -> PyResult<Py<Self>> {
        let mut dut = DUT.lock().unwrap();
        dut.highz_pin_group(slf.model_id, &slf.name, Option::None)?;
        Ok(slf.into())
    }

    #[args(kwargs = "**")]
    fn set_actions(
        slf: PyRef<Self>,
        actions: &PyAny,
        kwargs: Option<&PyDict>,
    ) -> PyResult<Py<Self>> {
        let mut dut = DUT.lock().unwrap();
        let mut mask = None;
        if let Some(args) = kwargs {
            if let Some(i) = args.get_item("mask") {
                mask = Some(i.extract::<usize>()?);
            }
        }
        dut.set_pin_group_symbols(
            slf.model_id,
            &slf.name,
            &extract_pinactions!(actions)?,
            mask,
        )?;
        Ok(slf.into())
    }

    fn reset(slf: PyRef<Self>) -> PyResult<Py<Self>> {
        let mut dut = DUT.lock().unwrap();
        dut.reset_pin_group(slf.model_id, &slf.name)?;
        Ok(slf.into())
    }

    #[getter]
    fn get_physical_names(&self) -> PyResult<Vec<String>> {
        let dut = DUT.lock().unwrap();
        let names = dut.resolve_pin_group_names(self.model_id, &self.name)?;
        Ok(names.clone())
    }

    #[getter]
    fn get_width(&self) -> PyResult<usize> {
        let dut = DUT.lock().unwrap();
        let grp = dut._get_pin_group(self.model_id, &self.name)?;
        Ok(grp.len())
    }

    #[getter]
    fn get_reset_data(&self) -> PyResult<u32> {
        let dut = DUT.lock().unwrap();
        Ok(dut.get_pin_group_reset_data(self.model_id, &self.name)?)
    }

    #[getter]
    fn get_reset_actions(&self) -> PyResult<PyObject> {
        let dut = DUT.lock().unwrap();
        let gil = Python::acquire_gil();
        let py = gil.python();
        let pin_actions = dut.get_pin_group_reset_actions(self.model_id, &self.name)?;
        Ok(PinActions {
            actions: pin_actions,
        }
        .into_py(py))
    }

    #[getter]
    fn get_big_endian(&self) -> PyResult<bool> {
        let is_little_endian = self.get_little_endian()?;
        Ok(!is_little_endian)
    }

    #[getter]
    fn get_little_endian(&self) -> PyResult<bool> {
        let dut = DUT.lock().unwrap();
        let grp = dut._get_pin_group(self.model_id, &self.name)?;
        Ok(grp.is_little_endian())
    }

    #[args(kwargs = "**")]
    fn cycle(slf: PyRef<Self>, kwargs: Option<&PyDict>) -> PyResult<Py<Self>> {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let locals = PyDict::new(py);
        locals.set_item("origen", py.import("origen")?)?;
        locals.set_item("kwargs", kwargs.to_object(py))?;

        py.eval(
            &format!("origen.tester.cycle(**(kwargs or {{}}))"),
            None,
            Some(&locals),
        )?;
        Ok(slf.into())
    }

    fn repeat(slf: PyRef<Self>, count: usize) -> PyResult<Py<Self>> {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let locals = PyDict::new(py);
        locals.set_item("origen", py.import("origen")?)?;
        py.eval(
            &format!("origen.tester.repeat({})", count),
            None,
            Some(&locals),
        )?;
        Ok(slf.into())
    }
}

#[pyproto]
impl pyo3::class::sequence::PySequenceProtocol for PinGroup {
    // Need to overwrite contains to account for aliasing
    fn __contains__(&self, item: &PyAny) -> PyResult<bool> {
        if let Ok(s) = item.extract::<String>() {
            let dut = DUT.lock().unwrap();
            Ok(dut.pin_group_contains(self.model_id, &self.name, &s)?)
        } else {
            Ok(false)
        }
    }
}

impl ListLikeAPI for PinGroup {
    fn item_ids(&self, dut: &std::sync::MutexGuard<origen::core::dut::Dut>) -> Vec<usize> {
        let grp = dut._get_pin_group(self.model_id, &self.name).unwrap();
        let mut pin_ids: Vec<usize> = vec![];
        for pname in grp.pin_names.iter() {
            pin_ids.push(dut._get_pin(self.model_id, pname).unwrap().id);
        }
        pin_ids
    }

    // Grabs a single pin and puts it in an anonymous pin collection
    fn new_pyitem(&self, py: Python, idx: usize) -> PyResult<PyObject> {
        let dut = DUT.lock().unwrap();
        let collection = dut.slice_pin_group(self.model_id, &self.name, idx, idx + 1, 1)?;
        Ok(Py::new(py, PinCollection::from(collection))
            .unwrap()
            .to_object(py))
    }

    fn __iter__(&self) -> PyResult<ListLikeIter> {
        Ok(ListLikeIter {
            parent: Box::new((*self).clone()),
            i: 0,
        })
    }

    fn ___getslice__(&self, slice: &PySlice) -> PyResult<PyObject> {
        let mut names: Vec<String> = vec![];
        {
            let (indices, pin_names);
            let dut = DUT.lock().unwrap();
            pin_names = &dut._get_pin_group(self.model_id, &self.name)?.pin_names;
            indices = slice.indices((pin_names.len() as i32).into())?;

            let mut i = indices.start;
            if indices.step > 0 {
                while i < indices.stop {
                    names.push(pin_names[i as usize].clone());
                    i += indices.step;
                }
            } else {
                while i > indices.stop {
                    names.push(pin_names[i as usize].clone());
                    i += indices.step;
                }
            }
        }
        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(Py::new(py, PinCollection::new(self.model_id, names, None)?)
            .unwrap()
            .to_object(py))
    }
}

#[pyproto]
impl pyo3::class::mapping::PyMappingProtocol for PinGroup {
    fn __getitem__(&self, idx: &PyAny) -> PyResult<PyObject> {
        ListLikeAPI::__getitem__(self, idx)
    }

    fn __len__(&self) -> PyResult<usize> {
        ListLikeAPI::__len__(self)
    }
}

#[pyproto]
impl pyo3::class::iter::PyIterProtocol for PinGroup {
    fn __iter__(slf: PyRefMut<Self>) -> PyResult<ListLikeIter> {
        ListLikeAPI::__iter__(&*slf)
    }
}
