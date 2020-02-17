use super::pin_collection::PinCollection;
use origen::DUT;
use pyo3::prelude::*;
#[allow(unused_imports)]
use pyo3::types::{PyAny, PyBytes, PyDict, PyIterator, PyList, PySlice, PyTuple};

#[pyclass]
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
    fn set_data(&self, data: u32) -> PyResult<Py<Self>> {
        let mut dut = DUT.lock().unwrap();
        dut.set_pin_group_data(self.model_id, &self.name, data)?;

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

    fn set(&self, data: u32) -> PyResult<Py<Self>> {
        return self.set_data(data);
    }

    // fn with_mask(&self, mask: usize) -> PyResult<Py<Self>> {
    //     let mut dut = DUT.lock().unwrap();
    //     let model = dut.get_mut_model(self.model_id)?;
    //     model.set_pin_group_nonsticky_mask(&self.name, mask)?;

    //     let gil = Python::acquire_gil();
    //     let py = gil.python();
    //     Ok(Py::new(
    //         py,
    //         Self {
    //             name: self.name.clone(),
    //             path: self.path.clone(),
    //             model_id: self.model_id,
    //         },
    //     )
    //     .unwrap())
    // }

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
    fn get_pin_actions(&self) -> PyResult<String> {
        let dut = DUT.lock().unwrap();
        Ok(dut.get_pin_group_actions(self.model_id, &self.name)?)
    }

    fn drive(&self, data: Option<u32>) -> PyResult<()> {
        let mut dut = DUT.lock().unwrap();
        Ok(dut.drive_pin_group(self.model_id, &self.name, data, Option::None)?)
    }

    fn verify(&self, data: Option<u32>) -> PyResult<()> {
        let mut dut = DUT.lock().unwrap();
        Ok(dut.verify_pin_group(self.model_id, &self.name, data, Option::None)?)
    }

    fn capture(&self) -> PyResult<()> {
        let mut dut = DUT.lock().unwrap();
        Ok(dut.capture_pin_group(self.model_id, &self.name, Option::None)?)
    }

    fn highz(&self) -> PyResult<()> {
        let mut dut = DUT.lock().unwrap();
        Ok(dut.highz_pin_group(self.model_id, &self.name, Option::None)?)
    }

    fn reset(&self) -> PyResult<()> {
        let mut dut = DUT.lock().unwrap();
        Ok(dut.reset_pin_group(self.model_id, &self.name)?)
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
    fn get_reset_actions(&self) -> PyResult<String> {
        let dut = DUT.lock().unwrap();
        Ok(dut.get_pin_group_reset_actions(self.model_id, &self.name)?)
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
}

#[pyproto]
impl pyo3::class::sequence::PySequenceProtocol for PinGroup {
    fn __len__(&self) -> PyResult<usize> {
        let dut = DUT.lock().unwrap();
        let grp = dut._get_pin_group(self.model_id, &self.name)?;
        Ok(grp.len())
    }

    fn __contains__(&self, item: &str) -> PyResult<bool> {
        let dut = DUT.lock().unwrap();
        Ok(dut.pin_group_contains(self.model_id, &self.name, item)?)
    }
}

#[pyproto]
impl<'p> pyo3::class::PyMappingProtocol<'p> for PinGroup {
    // Indexing example: https://github.com/PyO3/pyo3/blob/master/tests/test_dunder.rs#L423-L438
    fn __getitem__(&self, idx: &PyAny) -> PyResult<PyObject> {
        let dut = DUT.lock().unwrap();

        let gil = Python::acquire_gil();
        let py = gil.python();
        if let Ok(slice) = idx.cast_as::<PySlice>() {
            // Indices requires (what I think is) a max size. Should be plenty.
            let indices = slice.indices(8192)?;
            let collection = dut.slice_pin_group(
                self.model_id,
                &self.name,
                indices.start as usize,
                indices.stop as usize,
                indices.step as usize,
            )?;
            Ok(Py::new(py, PinCollection::from(collection))
                .unwrap()
                .to_object(py))
        } else {
            let i = idx.extract::<isize>().unwrap();
            let collection = dut.slice_pin_group(self.model_id, &self.name, i as usize, i as usize, 1)?;
            Ok(Py::new(py, PinCollection::from(collection))
                .unwrap()
                .to_object(py))
        }
    }
}

#[pyproto]
impl pyo3::class::iter::PyIterProtocol for PinGroup {
    fn __iter__(slf: PyRefMut<Self>) -> PyResult<PinGroupIter> {
        let dut = DUT.lock().unwrap();
        let grp = dut._get_pin_group(slf.model_id, &slf.name)?;

        Ok(PinGroupIter {
            keys: grp.pin_names.clone(),
            i: 0,
            model_id: slf.model_id,
        })
    }
}

#[pyclass]
pub struct PinGroupIter {
    keys: Vec<String>,
    i: usize,
    model_id: usize,
}

#[pyproto]
impl pyo3::class::iter::PyIterProtocol for PinGroupIter {
    fn __iter__(slf: PyRefMut<Self>) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(slf.to_object(py))
    }

    fn __next__(mut slf: PyRefMut<Self>) -> PyResult<Option<PinCollection>> {
        if slf.i >= slf.keys.len() {
            return Ok(None);
        }
        let name = slf.keys[slf.i].clone();
        let collection = PinCollection::new(slf.model_id, vec![name], Option::None)?;
        slf.i += 1;
        Ok(Some(collection))
    }
}
