use crate::resolve_transaction;
use num_bigint::BigUint;
use origen::Transaction as Trans;
use origen::TransactionAction;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict, PyType};

#[pyclass]
pub struct Transaction {
    pub transaction: Trans,
}

#[pymethods]
impl Transaction {
    #[new]
    #[args(kwargs = "**")]
    fn new(bc_or_val: &PyAny, kwargs: Option<&PyDict>) -> PyResult<Self> {
        let dut = origen::dut();
        Ok(Self {
            transaction: resolve_transaction(&dut, bc_or_val, None, kwargs)?,
        })
    }

    #[classmethod]
    #[args(kwargs = "**")]
    fn new_write(_cls: &PyType, bc_or_val: &PyAny, kwargs: Option<&PyDict>) -> PyResult<Self> {
        let dut = origen::dut();
        Ok(Self {
            transaction: resolve_transaction(
                &dut,
                bc_or_val,
                Some(TransactionAction::Write),
                kwargs,
            )?,
        })
    }

    #[classmethod]
    #[args(kwargs = "**")]
    fn new_verify(_cls: &PyType, bc_or_val: &PyAny, kwargs: Option<&PyDict>) -> PyResult<Self> {
        let dut = origen::dut();
        Ok(Self {
            transaction: resolve_transaction(
                &dut,
                bc_or_val,
                Some(TransactionAction::Verify),
                kwargs,
            )?,
        })
    }

    fn chunk_data(&self, chunk_width: usize) -> PyResult<Vec<BigUint>> {
        Ok(self.transaction.chunk_data(chunk_width)?)
    }

    fn chunk_addr(&self, chunk_width: usize) -> PyResult<Vec<BigUint>> {
        Ok(self.transaction.chunk_addr(chunk_width)?)
    }

    #[getter]
    fn data(&self) -> PyResult<BigUint> {
        Ok(self.transaction.data.clone())
    }

    #[getter]
    fn address(&self, py: Python) -> PyResult<PyObject> {
        if let Some(a) = &self.transaction.address {
            Ok(a.to_object(py))
        } else {
            Ok(py.None())
        }
    }

    #[getter]
    fn addr(&self, py: Python) -> PyResult<PyObject> {
        self.address(py)
    }

    #[getter]
    fn width(&self) -> PyResult<usize> {
        Ok(self.transaction.width)
    }

    #[getter]
    fn address_width(&self, py: Python) -> PyResult<PyObject> {
        if let Some(a) = self.transaction.address_width {
            Ok(a.to_object(py))
        } else {
            Ok(py.None())
        }
    }

    #[getter]
    fn addr_width(&self, py: Python) -> PyResult<PyObject> {
        self.address_width(py)
    }

    #[getter]
    fn register(&self, py: Python) -> PyResult<PyObject> {
        let dut = origen::dut();
        if let Some(id) = self.transaction.reg_id {
            Ok(crate::registers::bit_collection::BitCollection::from_reg_id(id, &dut).into_py(py))
        } else {
            Ok(py.None())
        }
    }

    #[getter]
    fn reg(&self, py: Python) -> PyResult<PyObject> {
        self.register(py)
    }
}
