use super::super::meta::py_like_apis::list_like_api::{ListLikeAPI, ListLikeIter};
use super::super::pins::{extract_pin_transaction, unpack_pin_transaction_kwargs};
use super::pin_actions::PinActions;
use super::pin_collection::PinCollection;
use num_bigint::BigUint;
use origen::DUT;
use origen::{Transaction, TransactionAction};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict, PySlice};

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
    fn get_pin_names(&self) -> PyResult<Vec<String>> {
        let dut = DUT.lock().unwrap();
        let grp = dut._get_pin_group(self.model_id, &self.name)?;
        Ok(grp.pin_names(&dut)?)
    }

    #[getter]
    fn get_actions(&self) -> PyResult<PyObject> {
        let dut = DUT.lock().unwrap();
        let gil = Python::acquire_gil();
        let py = gil.python();

        let grp = dut._get_pin_group(self.model_id, &self.name)?;
        let pin_actions = grp.get_actions(&dut)?;
        Ok(PinActions {
            actions: pin_actions,
        }
        .into_py(py))
    }

    #[args(kwargs = "**")]
    fn drive(slf: PyRef<Self>, data: BigUint, kwargs: Option<&PyDict>) -> PyResult<Py<Self>> {
        let dut = DUT.lock().unwrap();
        let grp = dut._get_pin_group(slf.model_id, &slf.name)?;
        let mut t = Transaction::new_write(data, grp.len())?;
        unpack_pin_transaction_kwargs(&mut t, kwargs)?;
        grp.update(&dut, &t)?;
        Ok(slf.into())
    }

    #[args(kwargs = "**")]
    fn verify(slf: PyRef<Self>, data: BigUint, kwargs: Option<&PyDict>) -> PyResult<Py<Self>> {
        let dut = DUT.lock().unwrap();
        let grp = dut._get_pin_group(slf.model_id, &slf.name)?;
        let mut t = Transaction::new_verify(data, grp.len())?;
        unpack_pin_transaction_kwargs(&mut t, kwargs)?;
        grp.update(&dut, &t)?;
        Ok(slf.into())
    }

    fn highz(slf: PyRef<Self>) -> PyResult<Py<Self>> {
        let dut = DUT.lock().unwrap();
        let grp = dut._get_pin_group(slf.model_id, &slf.name)?;
        grp.update(&dut, &Transaction::new_highz(grp.len())?)?;
        Ok(slf.into())
    }

    #[setter]
    fn actions(slf: PyRef<Self>, actions: &PyAny) -> PyResult<()> {
        Self::set_actions(slf, actions, None)?;
        Ok(())
    }

    #[args(kwargs = "**")]
    fn set_actions(
        slf: PyRef<Self>,
        actions: &PyAny,
        kwargs: Option<&PyDict>,
    ) -> PyResult<Py<Self>> {
        let dut = DUT.lock().unwrap();
        let grp = dut._get_pin_group(slf.model_id, &slf.name)?;
        grp.update(
            &dut,
            &extract_pin_transaction(actions, TransactionAction::Set, kwargs)?,
        )?;
        Ok(slf.into())
    }

    fn reset(slf: PyRef<Self>) -> PyResult<Py<Self>> {
        let dut = DUT.lock().unwrap();
        let grp = dut._get_pin_group(slf.model_id, &slf.name)?;
        grp.reset(&dut)?;
        Ok(slf.into())
    }

    #[getter]
    fn get_width(&self) -> PyResult<usize> {
        let dut = DUT.lock().unwrap();
        let grp = dut._get_pin_group(self.model_id, &self.name)?;
        Ok(grp.len())
    }

    #[getter]
    fn get_reset_actions(&self) -> PyResult<PyObject> {
        let dut = DUT.lock().unwrap();
        let gil = Python::acquire_gil();
        let py = gil.python();
        let grp = dut._get_pin_group(self.model_id, &self.name)?;
        let pin_actions = grp.get_reset_actions(&dut)?;
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

    #[args(label = "None", symbol = "None", cycles = "None", mask = "None")]
    fn overlay(
        slf: PyRef<Self>,
        label: Option<String>,
        symbol: Option<String>,
        cycles: Option<usize>,
        mask: Option<BigUint>,
    ) -> PyResult<Py<Self>> {
        let dut = DUT.lock().unwrap();
        let grp = dut._get_pin_group(slf.model_id, &slf.name)?;
        grp.overlay(&mut origen::Overlay::placeholder(
            label, symbol, cycles, mask,
        ))?;
        Ok(slf.into())
    }

    #[args(symbol = "None", cycles = "None", mask = "None")]
    fn capture(
        slf: PyRef<Self>,
        symbol: Option<String>,
        cycles: Option<usize>,
        mask: Option<BigUint>,
    ) -> PyResult<Py<Self>> {
        let dut = DUT.lock().unwrap();
        let grp = dut._get_pin_group(slf.model_id, &slf.name)?;
        grp.capture(&mut origen::Capture::placeholder(symbol, cycles, mask))?;
        Ok(slf.into())
    }

    #[getter]
    #[allow(non_snake_case)]
    fn get___origen_id__(&self) -> PyResult<usize> {
        let dut = DUT.lock().unwrap();
        let grp = dut._get_pin_group(self.model_id, &self.name)?;
        Ok(grp.id)
    }

    #[getter]
    #[allow(non_snake_case)]
    fn get___origen_pin_ids__(&self) -> PyResult<Vec<usize>> {
        let dut = DUT.lock().unwrap();
        let grp = dut._get_pin_group(self.model_id, &self.name)?;
        Ok(grp.pin_ids.clone())
    }

    fn __getitem__(&self, idx: &PyAny) -> PyResult<PyObject> {
        ListLikeAPI::__getitem__(self, idx)
    }

    fn __len__(&self) -> PyResult<usize> {
        ListLikeAPI::__len__(self)
    }

    fn __iter__(slf: PyRefMut<Self>) -> PyResult<ListLikeIter> {
        ListLikeAPI::__iter__(&*slf)
    }

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
        grp.pin_ids.clone()
    }

    // Grabs a single pin and puts it in an anonymous pin collection
    fn new_pyitem(&self, py: Python, idx: usize) -> PyResult<PyObject> {
        let dut = DUT.lock().unwrap();
        let grp = dut._get_pin_group(self.model_id, &self.name)?;
        Ok(Py::new(
            py,
            PinCollection::from_ids_unchecked(vec![grp.pin_ids[idx]], Some(grp.endianness)),
        )?
        .to_object(py))
    }

    fn __iter__(&self) -> PyResult<ListLikeIter> {
        Ok(ListLikeIter {
            parent: Box::new((*self).clone()),
            i: 0,
        })
    }

    fn ___getslice__(&self, slice: &PySlice) -> PyResult<PyObject> {
        let mut ids: Vec<usize> = vec![];
        {
            let dut = DUT.lock().unwrap();
            let grp = dut._get_pin_group(self.model_id, &self.name).unwrap();
            let indices = slice.indices((grp.pin_ids.len() as i32).into())?;

            let mut i = indices.start;
            if indices.step > 0 {
                while i < indices.stop {
                    ids.push(grp.pin_ids[i as usize]);
                    i += indices.step;
                }
            } else {
                while i > indices.stop {
                    ids.push(grp.pin_ids[i as usize]);
                    i += indices.step;
                }
            }
        }
        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(Py::new(py, PinCollection::from_ids_unchecked(ids, None))?.to_object(py))
    }
}
