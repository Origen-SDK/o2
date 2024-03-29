use super::super::meta::py_like_apis::list_like_api::GeneralizedListLikeAPI;
use origen::core::model::pins::pin::PinAction as OrigenPinAction;
use pyo3::class::basic::CompareOp;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict, PySlice, PyTuple, PyType};

macro_rules! extract_pinactions {
    ($actions: ident) => {{
        if let Ok(s) = $actions.extract::<String>() {
            let mut acts = origen::core::model::pins::pin::PinAction::from_action_str(&s)?;
            acts.reverse();
            Ok(acts)
        } else if $actions.get_type().name()? == "PinActions" {
            let acts = $actions.extract::<PyRef<crate::pins::pin_actions::PinActions>>()?;
            Ok(acts.actions.clone())
        } else {
            Err(pyo3::exceptions::PyTypeError::new_err(format!(
                "Cannot extract _origen.pin.PinActions from type {}",
                $actions.get_type().name()?
            )))
        }
    }};
}

#[pyclass]
#[derive(Clone)]
/// Represents a pin group/collection's *action string*.
/// Customized the data structure to display and index as a "number"
/// Using ``str`` values, this would lead to indexing requiring reversing the return value (or vice versa)
///
/// >> pin.data = 0xC
/// >> pins.actions = "VVVV"
///
/// >> pins.actions
///    => HHLL
///
/// >> pins.actions[0]
///    => L
///
/// >> str(pins.actions)
///    => "HHLL"
///
/// >> str(pins.actions)[0]
///    => "H"
///
/// Note that this represents the pin values **at that instant** and will **not** reflect changes
/// that occur to the underlying pins:
///
/// .. code:: python
///
///     # From the above
///     state = pins.actions
///         #=> HHLL
///
/// .. code:: python
///
///     pins.data = "0xF"
///     pins.drive()
///     pins.actions
///         #=> "1111"
///
/// >> state
///    => HHLL
pub struct PinActions {
    pub actions: Vec<OrigenPinAction>,
}

#[pymethods]
impl PinActions {
    #[allow(non_snake_case)]
    #[classmethod]
    fn DriveHigh(_cls: &PyType, py: Python) -> PyResult<PyObject> {
        Ok(PinActions {
            actions: vec![OrigenPinAction::drive_high()],
        }
        .into_py(py))
    }

    #[allow(non_snake_case)]
    #[classmethod]
    fn DriveLow(_cls: &PyType, py: Python) -> PyResult<PyObject> {
        Ok(PinActions {
            actions: vec![OrigenPinAction::drive_low()],
        }
        .into_py(py))
    }

    #[allow(non_snake_case)]
    #[classmethod]
    fn VerifyHigh(_cls: &PyType, py: Python) -> PyResult<PyObject> {
        Ok(PinActions {
            actions: vec![OrigenPinAction::verify_high()],
        }
        .into_py(py))
    }

    #[allow(non_snake_case)]
    #[classmethod]
    fn VerifyLow(_cls: &PyType, py: Python) -> PyResult<PyObject> {
        Ok(PinActions {
            actions: vec![OrigenPinAction::verify_low()],
        }
        .into_py(py))
    }

    #[allow(non_snake_case)]
    #[classmethod]
    fn Capture(_cls: &PyType, py: Python) -> PyResult<PyObject> {
        Ok(PinActions {
            actions: vec![OrigenPinAction::capture()],
        }
        .into_py(py))
    }

    #[allow(non_snake_case)]
    #[classmethod]
    fn HighZ(_cls: &PyType, py: Python) -> PyResult<PyObject> {
        Ok(PinActions {
            actions: vec![OrigenPinAction::highz()],
        }
        .into_py(py))
    }

    #[allow(non_snake_case)]
    #[classmethod]
    fn Multichar(_cls: &PyType, py: Python, symbol: String) -> PyResult<PyObject> {
        Ok(PinActions {
            actions: vec![OrigenPinAction::new(&symbol)],
        }
        .into_py(py))
    }

    #[classmethod]
    fn standard_actions(cls: &PyType, py: Python) -> PyResult<PyObject> {
        let retn = PyDict::new(py);
        retn.set_item("DriveHigh", Self::DriveHigh(cls, py)?)?;
        retn.set_item("DriveLow", Self::DriveLow(cls, py)?)?;
        retn.set_item("VerifyHigh", Self::VerifyHigh(cls, py)?)?;
        retn.set_item("VerifyLow", Self::VerifyLow(cls, py)?)?;
        retn.set_item("Capture", Self::Capture(cls, py)?)?;
        retn.set_item("HighZ", Self::HighZ(cls, py)?)?;
        Ok(retn.into())
    }

    #[new]
    #[pyo3(signature=(*actions, **_kwargs))]
    fn new(actions: &PyTuple, _kwargs: Option<&PyDict>) -> PyResult<Self> {
        let mut temp: Vec<OrigenPinAction> = vec![];
        // if let Some(actions_) = actions {
        for a in actions.iter() {
            if let Ok(s) = a.extract::<String>() {
                temp.extend(OrigenPinAction::from_action_str(&s)?);
            } else if a.get_type().name()? == "PinActions" {
                let s = a.extract::<PyRef<Self>>().unwrap();
                let mut s_ = s.actions.clone();
                s_.reverse();
                temp.extend(s_);
            } else {
                return super::super::type_error!(&format!(
                    "Cannot cast type {} to a valid PinAction",
                    a.get_type().name()?
                ));
            }
            // }
        }
        temp.reverse();
        let obj = Self { actions: temp };
        Ok(obj)
    }

    #[getter]
    fn all_standard(&self) -> PyResult<bool> {
        Ok(self.actions.iter().all(|a| a.is_standard()))
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(OrigenPinAction::to_action_string(&self.actions)?)
    }

    /// Comparing PinActions boils down to comparing their current actions, which we can
    /// do by just *to-string-ing* the actions and comparing them.
    /// Comparisons are only valid for *equal* and *not equal*. Can't compare if one
    /// pin action is *greater than* or *less than* another.
    /// Example of richcmp: https://github.com/PyO3/pyo3/blob/a5e3d4e7c8d80f7020510cf630ab01001612c6a7/tests/test_arithmetics.rs#L358-L373
    fn __richcmp__(&self, py: Python, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        // Support comparing either to a str or another Actions object
        let other_string;
        if let Ok(s) = other.extract::<String>() {
            other_string = s;
        } else if other.get_type().name()? == "PinActions" {
            let other_actions = other.extract::<PyRef<Self>>()?;
            other_string = OrigenPinAction::to_action_string(&other_actions.actions)?;
        } else {
            return Ok(false.to_object(py));
        }

        match op {
            CompareOp::Eq => {
                if OrigenPinAction::to_action_string(&self.actions)? == other_string {
                    Ok(true.to_object(py))
                } else {
                    Ok(false.to_object(py))
                }
            }
            CompareOp::Ne => {
                if OrigenPinAction::to_action_string(&self.actions)? == other_string {
                    Ok(false.to_object(py))
                } else {
                    Ok(true.to_object(py))
                }
            }
            _ => Ok(py.NotImplemented()),
        }
    }

    fn __iter__(slf: PyRefMut<Self>) -> PyResult<PinActionsIter> {
        Ok(PinActionsIter {
            parent: Box::new((slf).clone()),
            i: 0,
        })
    }

    fn __getitem__(&self, idx: &PyAny) -> PyResult<PyObject> {
        GeneralizedListLikeAPI::__getitem__(self, idx)
    }

    fn __len__(&self) -> PyResult<usize> {
        GeneralizedListLikeAPI::__len__(self)
    }
}

impl GeneralizedListLikeAPI for PinActions {
    type Contained = OrigenPinAction;

    fn items(&self) -> &Vec<Self::Contained> {
        &self.actions
    }

    fn new_pyitem(&self, py: Python, item: &Self::Contained, _idx: usize) -> PyResult<PyObject> {
        Ok(PinActions {
            actions: vec![item.clone()],
        }
        .into_py(py))
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
            self.new_pyitem(py, &self.items()[_idx], _idx)
        })
    }

    fn ___getslice__(&self, slice: &PySlice) -> PyResult<PyObject> {
        let mut actions: Vec<OrigenPinAction> = vec![];
        {
            let indices = slice.indices((self.items().len() as i32).into())?;
            let mut i = indices.start;
            if indices.step > 0 {
                while i < indices.stop {
                    actions.push(self.actions[i as usize].clone());
                    i += indices.step;
                }
            } else {
                while i > indices.stop {
                    actions.push(self.actions[i as usize].clone());
                    i += indices.step;
                }
            }
        }
        Ok(Python::with_gil(|py| {
            PinActions { actions: actions }.into_py(py)
        }))
    }
}

#[pyclass]
pub struct PinActionsIter {
    pub parent: Box<PinActions>,
    pub i: usize,
}

#[pymethods]
impl PinActionsIter {
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
