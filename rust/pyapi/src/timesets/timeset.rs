use super::timeset_container::{
    EventContainer, WaveContainer, WaveGroupContainer, WavetableContainer,
};
use origen::error::Error;
use origen::DUT;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict, PyTuple};
use pyo3::class::mapping::PyMappingProtocol;
use super::super::pins::pins_to_backend_lookup_fields;

#[macro_export]
macro_rules! pytimeset {
    ($py:expr, $model:expr, $model_id:expr, $name:expr) => {
        if $model.contains_timeset($name) {
            Ok(Py::new(
                $py,
                crate::timesets::timeset::Timeset {
                    name: String::from($name),
                    model_id: $model_id,
                },
            )
            .unwrap()
            .to_object($py))
        } else {
            // Note: Errors here shouldn't happen. Any errors that arise are either
            // bugs or from the user meta-programming their way into the backend DB.
            Err(PyErr::from(origen::error::Error::new(&format!(
                "No timeset {} has been added on block {}",
                $name, $model.name
            ))))
        }
    };
}

// Returns a (Python) Timeset or NoneType instance.
// Note: this does NOT return a Rust Option::None, but
#[macro_export]
macro_rules! pytimeset_or_pynone {
    ($py:expr, $model:expr, $model_id:expr, $name:expr) => {
        if $model.contains_timeset($name) {
            Py::new(
                $py,
                crate::timesets::timeset::Timeset {
                    name: String::from($name),
                    model_id: $model_id,
                },
            )
            .unwrap()
            .to_object($py)
        } else {
            $py.None()
        }
    };
}

#[macro_export]
macro_rules! pywavetable {
    ($py:expr, $timeset:expr, $t_id:expr, $name:expr) => {
        if $timeset.contains_wavetable($name) {
            Ok(Py::new(
                $py,
                crate::timesets::timeset::Wavetable {
                    name: String::from($name),
                    model_id: $timeset.model_id,
                    timeset_id: $timeset.id,
                },
            )
            .unwrap()
            .to_object($py))
        } else {
            // Note: Errors here shouldn't happen. Any errors that arise are either
            // bugs or from the user meta-programming their way into the backend DB.
            Err(PyErr::from(origen::error::Error::new(&format!(
                "No wavetable {} has been added on block {}",
                $name, $timeset.name
            ))))
        }
    };
}

#[macro_export]
macro_rules! pywave_group {
    ($py:expr, $wavetable:expr, $name:expr) => {
        if $wavetable.contains_wave_group($name) {
            Ok(Py::new(
                $py,
                crate::timesets::timeset::WaveGroup {
                    name: String::from($name),
                    model_id: $wavetable.model_id,
                    timeset_id: $wavetable.timeset_id,
                    wavetable_id: $wavetable.id,
                },
            )
            .unwrap()
            .to_object($py))
        } else {
            // Note: Errors here shouldn't happen. Any errors that arise are either
            // bugs or from the user meta-programming their way into the backend DB.
            Err(PyErr::from(origen::error::Error::new(&format!(
                "No wave group {} has been added on block {}",
                $name, $wavetable.name
            ))))
        }
    };
}

#[macro_export]
macro_rules! pywave {
    ($py:expr, $wave_group:expr, $name:expr) => {
        if $wave_group.contains_wave($name) {
            Ok(Py::new(
                $py,
                crate::timesets::timeset::Wave {
                    name: String::from($name),
                    model_id: $wave_group.model_id,
                    timeset_id: $wave_group.timeset_id,
                    wavetable_id: $wave_group.wavetable_id,
                    wave_group_id: $wave_group.id,
                },
            )
            .unwrap()
            .to_object($py))
        } else {
            // Note: Errors here shouldn't happen. Any errors that arise are either
            // bugs or from the user meta-programming their way into the backend DB.
            Err(PyErr::from(origen::error::Error::new(&format!(
                "No wave {} has been added on block {}",
                $name, $wave_group.name
            ))))
        }
    };
}

#[macro_export]
macro_rules! pyevent {
    ($py:expr, $wave:expr, $event_index:expr) => {
        if $wave.events.len() > $event_index {
            Ok(Py::new(
                $py,
                crate::timesets::timeset::Event {
                    model_id: $wave.model_id,
                    timeset_id: $wave.timeset_id,
                    wavetable_id: $wave.wavetable_id,
                    wave_group_id: $wave.wave_group_id,
                    wave_id: $wave.wave_id,
                    wave_indicator: $wave.indicator.clone(),
                    index: $event_index,
                },
            )
            .unwrap()
            .to_object($py))
        } else {
            // Note: Errors here shouldn't happen. Any errors that arise are either
            // bugs or from the user meta-programming their way into the backend DB.
            Err(PyErr::from(origen::error::Error::new(&format!(
                "No event at {} has been added on wave {}",
                $event_index, $wave.indicator
            ))))
        }
    };
}

#[pyclass]
pub struct Timeset {
    pub name: String,
    pub model_id: usize,
}

#[pymethods]
impl Timeset {
    #[getter]
    fn get_name(&self) -> PyResult<String> {
        let dut = DUT.lock().unwrap();
        let timeset = dut.get_timeset(self.model_id, &self.name);
        Ok(timeset.unwrap().name.clone())
    }

    #[allow(non_snake_case)]
    #[getter]
    fn get___origen__model_id__(&self) -> PyResult<usize> {
        Ok(self.model_id)
    }

    #[getter]
    fn get_period(&self) -> PyResult<f64> {
        let dut = DUT.lock().unwrap();
        let timeset = dut._get_timeset(self.model_id, &self.name)?;
        Ok(timeset.eval(Option::None)?)
    }

    #[getter]
    fn get_default_period(&self) -> PyResult<PyObject> {
        let dut = DUT.lock().unwrap();
        let timeset = dut._get_timeset(self.model_id, &self.name)?;

        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(match timeset.default_period {
            Some(p) => p.to_object(py),
            None => py.None(),
        })
    }

    #[allow(non_snake_case)]
    #[getter]
    fn get___eval_str__(&self) -> PyResult<String> {
        let dut = DUT.lock().unwrap();
        let timeset = dut._get_timeset(self.model_id, &self.name)?;
        Ok(timeset.eval_str().clone())
    }

    #[allow(non_snake_case)]
    #[getter]
    fn get___period__(&self) -> PyResult<PyObject> {
        let dut = DUT.lock().unwrap();
        let timeset = dut._get_timeset(self.model_id, &self.name)?;
        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(match &timeset.period_as_string {
            Some(p) => p.clone().to_object(py),
            None => py.None(),
        })
    }

    #[getter]
    fn wavetables(&self) -> PyResult<Py<WavetableContainer>> {
        let t_id;
        {
            t_id = self.get_origen_id()?;
        }
        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(pywavetable_container!(py, self.model_id, t_id, &self.name))
    }

    #[args(_kwargs = "**")]
    fn add_wavetable(&self, name: &str, _kwargs: Option<&PyDict>) -> PyResult<PyObject> {
        let mut dut = DUT.lock().unwrap();
        let t_id;
        {
            t_id = dut._get_timeset(self.model_id, &self.name).unwrap().id;
        }
        dut.create_wavetable(t_id, name)?;

        let gil = Python::acquire_gil();
        let py = gil.python();
        let tset = dut._get_timeset(self.model_id, &self.name).unwrap();
        Ok(pywavetable!(py, tset, t_id, name)?)
    }

    #[getter]
    fn symbol_map(&self) -> PyResult<PyObject> {
        let dut = DUT.lock().unwrap();
        let t_id;
        {
            t_id = dut._get_timeset(self.model_id, &self.name).unwrap().id;
        }
        let tester = origen::tester();

        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(Py::new(
            py,
            crate::timesets::timeset::SymbolMap {
                timeset_id: t_id,
                target_name: {
                    match tester.focused_tester_name() {
                        Some(n) => n,
                        None => return Ok(py.None())
                    }
                }
            },
        )
        .unwrap()
        .to_object(py))
    }

    #[getter]
    fn symbol_maps(&self) -> PyResult<Vec<PyObject>> {
        let dut = DUT.lock().unwrap();
        let t = dut._get_timeset(self.model_id, &self.name).unwrap();
        let t_id;
        {
            t_id = t.id;
        }

        let gil = Python::acquire_gil();
        let py = gil.python();
        let retn = t.pin_action_resolvers.keys().map({ |target| 
            Py::new(
                py,
                crate::timesets::timeset::SymbolMap {
                    timeset_id: t_id,
                    target_name: target.to_string()
                },
            )
            .unwrap()
            .to_object(py)
        }).collect::<Vec<PyObject>>();
        Ok(retn)
    }
}

impl Timeset {
    pub fn new(name: &str, model_id: usize) -> Self {
        Self {
            name: String::from(name),
            model_id: model_id,
        }
    }

    pub fn get_origen_id(&self) -> Result<usize, Error> {
        let dut = DUT.lock().unwrap();
        let timeset = dut._get_timeset(self.model_id, &self.name)?;
        Ok(timeset.id)
    }
}

#[pyclass]
pub struct Wavetable {
    pub timeset_id: usize,
    pub name: String,
    pub model_id: usize,
}

#[pymethods]
impl Wavetable {
    #[args(_kwargs = "**")]
    fn add_waves(&self, name: &str, _kwargs: Option<&PyDict>) -> PyResult<PyObject> {
        let mut dut = DUT.lock().unwrap();
        let w_id;
        {
            w_id = dut.get_wavetable(self.timeset_id, &self.name).unwrap().id;
        }
        dut.create_wave_group(w_id, name, Option::None)?;

        let gil = Python::acquire_gil();
        let py = gil.python();
        let wt = dut.get_wavetable(self.timeset_id, &self.name).unwrap();
        Ok(pywave_group!(py, wt, name)?)
    }

    #[args(_kwargs = "**")]
    fn add_wave(&self, name: &str, _kwargs: Option<&PyDict>) -> PyResult<PyObject> {
        self.add_waves(name, _kwargs)
    }

    #[getter]
    fn get_waves(&self) -> PyResult<Py<WaveGroupContainer>> {
        let w_id;
        {
            w_id = self.get_origen_id()?;
        }
        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(pywave_group_container!(
            py,
            self.model_id,
            self.timeset_id,
            w_id,
            &self.name
        ))
    }

    /// Retrieves all applied waves as a dictionary whose keys are the physical pins which has a corresponding
    /// wave. The values are another dictionary whose key-value pair is the indicator finally pointing to the wave
    /// which defines the wave.
    ///
    /// .. code: python
    ///     {
    ///         porta1: {
    ///             "0": <Wave>,
    ///             "1": <Wave>,
    ///             "h": <Wave>,
    ///             "l": <Wave>,
    ///         },
    ///         porta0: {
    ///             "0": <Wave>,
    ///             "1": <Wave>,
    ///             "h": <Wave>,
    ///             "l": <Wave>,
    ///         },
    ///         clk: {
    ///             "0": <Wave>,
    ///             "1": <Wave>,
    ///         }
    ///     }
    ///
    fn applied_waves(&self) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let empty: [PyObject; 0] = [];
        let t = PyTuple::new(py, &empty);
        self.applied_waves_for(t, None)
    }

    /// Same as :meth:`applied_waves` but supports internal filtering of the return values.
    #[args(pins = "*")]
    fn applied_waves_for(&self, pins: &PyTuple, indicators: Option<Vec<String>>) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let dut = DUT.lock().unwrap();
        let wt = dut._get_wavetable(self.timeset_id, &self.name)?;
        let waves = wt.applied_waves(&dut, &pins_to_backend_lookup_fields(py, &pins)?, &indicators.unwrap_or(vec!()))?;
        Ok(waves.to_object(py))
    }

    #[getter]
    fn get_symbol_map(&self) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let tester = origen::tester();
        match tester.focused_tester_name() {
            Some(name) => {
                Ok(Py::new(py, SymbolMap::new(self.timeset_id, name))
                    .unwrap()
                    .to_object(py))
            },
            None => Ok(py.None())
        }
    }

    #[getter]
    fn get_name(&self) -> PyResult<String> {
        let dut = DUT.lock().unwrap();
        let wt = dut.get_wavetable(self.timeset_id, &self.name);
        Ok(wt.unwrap().name.clone())
    }

    // Evaluates and returns the period.
    // Returns None if no period was specified or an error if it could not be evaluated.
    #[getter]
    pub fn get_period(&self) -> PyResult<PyObject> {
        let dut = DUT.lock().unwrap();
        let wt = dut.get_wavetable(self.timeset_id, &self.name);
        let p = wt.unwrap().eval(Option::None)?;

        let gil = Python::acquire_gil();
        let py = gil.python();
        match p {
            Some(_p) => Ok(_p.to_object(py)),
            None => Ok(py.None()),
        }
    }

    // From the Python side, want to support receiving input as either an expression (String)
    // or as hard coded integer/float values..
    #[setter]
    pub fn set_period(&self, period: &PyAny) -> PyResult<()> {
        let mut dut = DUT.lock().unwrap();
        let wt = dut.get_mut_wavetable(self.timeset_id, &self.name).unwrap();
        if let Ok(p) = period.extract::<String>() {
            wt.set_period(Some(Box::new(p)))?;
        } else if let Ok(p) = period.extract::<f64>() {
            wt.set_period(Some(Box::new(p)))?;
        } else if period.get_type().name() == "NoneType" {
            wt.set_period(Option::None)?;
        } else {
            return super::super::type_error!(format!("Could not interpret 'period' argument as Numeric, String, or NoneType! (class '{}')", period.get_type().name()));
        };
        Ok(())
    }

    // Returns the period as a string before evaluation.
    #[allow(non_snake_case)]
    #[getter]
    pub fn get___period__(&self) -> PyResult<PyObject> {
        let dut = DUT.lock().unwrap();
        let wt = dut.get_wavetable(self.timeset_id, &self.name);
        let p = &wt.unwrap().period;

        let gil = Python::acquire_gil();
        let py = gil.python();
        match p {
            Some(_p) => Ok(_p.to_object(py)),
            None => Ok(py.None()),
        }
    }
}

impl Wavetable {
    pub fn new(model_id: usize, timeset_id: usize, name: &str) -> Self {
        Self {
            timeset_id: timeset_id,
            name: String::from(name),
            model_id: model_id,
        }
    }

    pub fn get_origen_id(&self) -> Result<usize, Error> {
        let dut = DUT.lock().unwrap();
        let timeset = &dut.timesets[self.timeset_id];
        let w_id = timeset.get_wavetable_id(&self.name).unwrap();
        Ok(w_id)
    }
}

#[pyclass]
pub struct WaveGroup {
    pub model_id: usize,
    pub timeset_id: usize,
    pub wavetable_id: usize,
    pub name: String,
}

#[pymethods]
impl WaveGroup {
    #[args(_kwargs = "**")]
    fn add_wave(&self, name: &str, _kwargs: Option<&PyDict>) -> PyResult<PyObject> {
        let mut dut = DUT.lock().unwrap();
        let wgrp_id;
        {
            wgrp_id = dut
                .get_wave_group(self.wavetable_id, &self.name)
                .unwrap()
                .id;
        }
        let mut derived_from = Option::None;
        if let Some(args) = _kwargs {
            if let Some(_derived_from) = args.get_item("derived_from") {
                if let Ok(_waves) = _derived_from.extract::<String>() {
                    derived_from = Some(vec![_waves]);
                } else if let Ok(_waves) = _derived_from.extract::<Vec<String>>() {
                    derived_from = Some(_waves);
                } else {
                    return type_error!("Could not interpret 'derived_From' argument as a string or as a list of strings!");
                }
            }
        }
        dut.create_wave(wgrp_id, name, derived_from)?;

        let gil = Python::acquire_gil();
        let py = gil.python();
        let wgrp = dut.get_wave_group(self.wavetable_id, &self.name).unwrap();
        Ok(pywave!(py, wgrp, name)?)
    }

    #[getter]
    fn get_waves(&self) -> PyResult<Py<WaveContainer>> {
        let wgrp_id;
        {
            wgrp_id = self.get_origen_id()?;
        }
        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(pywave_container!(
            py,
            self.model_id,
            self.timeset_id,
            self.wavetable_id,
            wgrp_id,
            &self.name
        ))
    }

    #[getter]
    fn get_name(&self) -> PyResult<String> {
        let dut = DUT.lock().unwrap();
        let wt = dut.get_wavetable(self.timeset_id, &self.name);
        Ok(wt.unwrap().name.clone())
    }
}

impl WaveGroup {
    pub fn new(model_id: usize, timeset_id: usize, wavetable_id: usize, name: &str) -> Self {
        Self {
            model_id: model_id,
            timeset_id: timeset_id,
            wavetable_id: wavetable_id,
            name: String::from(name),
        }
    }

    pub fn get_origen_id(&self) -> Result<usize, Error> {
        let dut = DUT.lock().unwrap();
        let wavetable = &dut.wavetables[self.wavetable_id];
        let wgrp_id = wavetable.get_wave_group_id(&self.name).unwrap();
        Ok(wgrp_id)
    }
}

#[pyclass]
pub struct Wave {
    pub model_id: usize,
    pub timeset_id: usize,
    pub wavetable_id: usize,
    pub wave_group_id: usize,
    pub name: String,
}

#[pymethods]
impl Wave {
    #[getter]
    fn get_events(&self) -> PyResult<Py<EventContainer>> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let wave_id;
        {
            wave_id = self.get_origen_id()?;
        }
        Ok(pyevent_container!(
            py,
            self.model_id,
            self.timeset_id,
            self.wavetable_id,
            self.wave_group_id,
            wave_id,
            &self.name
        ))
    }

    #[args(event = "**")]
    fn push_event(&self, event: Option<&PyDict>) -> PyResult<PyObject> {
        let mut dut = DUT.lock().unwrap();
        let (w_id, e_index);
        {
            w_id = dut
                .get_wave(self.wave_group_id, &self.name)
                .unwrap()
                .wave_id;
        }

        if event.is_none() {
            return type_error!("Keywords 'at' and 'action' are required to push a new event!");
        }

        let (at, unit, action) = (
            event.unwrap().get_item("at"),
            event.unwrap().get_item("unit"),
            event.unwrap().get_item("action"),
        );
        {
            // Resolve the 'action' keyword first because rust is a pain in the butt.
            // This is required and can only be a String.
            let temp: String;
            match action {
                Some(_action) => {
                    if let Ok(val) = _action.extract::<String>() {
                        temp = val;
                    } else if _action.is_none() {
                        return type_error!("'action' keyword is required (found None)!");
                    } else {
                        return type_error!("Could not interpret 'action' argument as String!");
                    }
                }
                None => return type_error!("'action' keyword is required!"),
            }
            let e = dut.create_event(
                w_id,
                // Resolve the 'at' keyword. This is required and can be either a String or a numeric.
                match at {
                    Some(_at) => {
                        if let Ok(val) = _at.extract::<String>() {
                            Box::new(val)
                        } else if let Ok(val) = _at.extract::<f64>() {
                            Box::new(val)
                        } else if _at.is_none() {
                            return type_error!("'at' keyword is required (found None)!");
                        } else {
                            return type_error!(
                                "Could not interpret 'at' argument as String or Numeric!"
                            );
                        }
                    }
                    None => return type_error!("'at' keyword is required!"),
                },
                // Resolve the 'unit' keyword. This is optional and can only be a string.
                match unit {
                    Some(_unit) => {
                        if let Ok(val) = _unit.extract::<String>() {
                            Some(val)
                        } else if _unit.is_none() {
                            Option::None
                        } else {
                            return type_error!(
                                "Could not interpret 'unit' argument as String or NoneType!"
                            );
                        }
                    }
                    None => Option::None,
                },
                &temp,
            )?;
            e_index = e.event_index;
        }

        // Return the newly created event
        let gil = Python::acquire_gil();
        let py = gil.python();

        let w = dut.get_wave(self.wave_group_id, &self.name).unwrap();
        Ok(pyevent!(py, w, e_index)?)
    }

    #[getter]
    fn get_indicator(&self) -> PyResult<String> {
        let dut = DUT.lock().unwrap();
        let w = dut.get_wave(self.wave_group_id, &self.name).unwrap();
        Ok(w.indicator.clone())
    }

    #[setter]
    fn set_indicator(&self, indicator: &str) -> PyResult<()> {
        let mut dut = DUT.lock().unwrap();
        let w = dut.get_mut_wave(self.wave_group_id, &self.name).unwrap();
        w.set_indicator(&indicator)?;
        Ok(())
    }

    #[getter]
    fn get_applied_to(&self) -> PyResult<Vec<PyObject>> {
        let dut = DUT.lock().unwrap();
        let w = dut.get_wave(self.wave_group_id, &self.name).unwrap();
        let gil = Python::acquire_gil();
        let py = gil.python();

        let mut pins: Vec<PyObject> = vec![];
        for p in w.applied_pin_ids.iter() {
            let ppin = &dut.pins[*p];
            pins.push(super::super::pins::pin::Pin {
                name: ppin.name.clone(),
                model_id: ppin.model_id
            }.into_py(py));
        }
        Ok(pins)
    }

    #[args(pins = "*")]
    fn apply_to(&self, pins: Vec<String>) -> PyResult<PyObject> {
        let mut dut = DUT.lock().unwrap();
        let wid;
        {
            wid = dut.get_wave(self.wave_group_id, &self.name).unwrap().wave_id;
        }
        let pins_with_model_id: Vec<(usize, String)> = pins.iter().map(|pin| {
            (0, pin.clone())
        }).collect();
        dut.apply_wave_id_to_pins(wid, &pins_with_model_id)?;

        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(Py::new(
            py,
            crate::timesets::timeset::Wave {
                name: self.name.clone(),
                model_id: self.model_id,
                timeset_id: self.timeset_id,
                wavetable_id: self.wavetable_id,
                wave_group_id: self.wave_group_id,
            },
        )
        .unwrap()
        .to_object(py))
    }

    #[allow(non_snake_case)]
    #[getter]
    pub fn get_name(&self) -> PyResult<String> {
        Ok(self.name.clone())
    }

    #[allow(non_snake_case)]
    #[getter]
    pub fn get_DriveHigh(&self) -> PyResult<String> {
        Ok(String::from("DriveHigh"))
    }

    #[allow(non_snake_case)]
    #[getter]
    pub fn get_DriveLow(&self) -> PyResult<String> {
        Ok(String::from("DriveLow"))
    }

    #[allow(non_snake_case)]
    #[getter]
    pub fn get_HighZ(&self) -> PyResult<String> {
        Ok(String::from("HighZ"))
    }

    #[allow(non_snake_case)]
    #[getter]
    pub fn get_VerifyHigh(&self) -> PyResult<String> {
        Ok(String::from("VerifyHigh"))
    }

    #[allow(non_snake_case)]
    #[getter]
    pub fn get_VerifyLow(&self) -> PyResult<String> {
        Ok(String::from("VerifyLow"))
    }

    #[allow(non_snake_case)]
    #[getter]
    pub fn get_VerifyZ(&self) -> PyResult<String> {
        Ok(String::from("VerifyZ"))
    }

    #[allow(non_snake_case)]
    #[getter]
    pub fn get_Capture(&self) -> PyResult<String> {
        Ok(String::from("Capture"))
    }
}

impl Wave {
    pub fn new(
        model_id: usize,
        timeset_id: usize,
        wavetable_id: usize,
        wave_group_id: usize,
        name: &str,
    ) -> Self {
        Self {
            model_id: model_id,
            timeset_id: timeset_id,
            wavetable_id: wavetable_id,
            wave_group_id: wave_group_id,
            name: String::from(name),
        }
    }

    pub fn get_origen_id(&self) -> Result<usize, Error> {
        let dut = DUT.lock().unwrap();
        let wgrp = &dut.wave_groups[self.wave_group_id];
        let w_id = wgrp.get_wave_id(&self.name).unwrap();
        Ok(w_id)
    }
}

#[pyclass]
pub struct Event {
    pub model_id: usize,
    pub timeset_id: usize,
    pub wavetable_id: usize,
    pub wave_group_id: usize,
    pub wave_id: usize,
    pub wave_indicator: String,
    pub index: usize,
}

#[pymethods]
impl Event {
    #[getter]
    pub fn get_action(&self) -> PyResult<String> {
        let dut = DUT.lock().unwrap();
        let e = dut.get_event(self.wave_id, self.index).unwrap();
        Ok(e.action.clone())
    }

    #[setter]
    pub fn action(&self, action: &str) -> PyResult<()> {
        let mut dut = DUT.lock().unwrap();
        let e = dut.get_mut_event(self.wave_id, self.index).unwrap();
        e.set_action(action)?;
        Ok(())
    }

    #[getter]
    pub fn unit(&self) -> PyResult<PyObject> {
        let dut = DUT.lock().unwrap();
        let e = dut.get_event(self.wave_id, self.index).unwrap();

        let gil = Python::acquire_gil();
        let py = gil.python();
        match &e.unit {
            Some(unit) => Ok(unit.clone().to_object(py)),
            None => Ok(py.None()),
        }
    }

    #[getter]
    pub fn at(&self) -> PyResult<PyObject> {
        let dut = DUT.lock().unwrap();
        let e = dut.get_event(self.wave_id, self.index).unwrap();

        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(e.eval(&dut, Option::None)?.to_object(py))
    }

    #[getter]
    pub fn __at__(&self) -> PyResult<String> {
        let dut = DUT.lock().unwrap();
        let e = dut.get_event(self.wave_id, self.index).unwrap();
        Ok(e.at.clone())
    }
}

impl Event {
    pub fn new(
        model_id: usize,
        timeset_id: usize,
        wavetable_id: usize,
        wave_group_id: usize,
        wave_id: usize,
        wave_indicator: &str,
        id: usize,
    ) -> Self {
        Self {
            model_id: model_id,
            timeset_id: timeset_id,
            wavetable_id: wavetable_id,
            wave_group_id: wave_group_id,
            wave_id: wave_id,
            wave_indicator: String::from(wave_indicator),
            index: id,
        }
    }
}

macro_rules! action_from_pyany {
    ($action:ident) => {
        origen::core::model::pins::pin::PinAction::checked_new(
            {
                let t;
                if let Ok(a) = $action.extract::<String>() {
                    t = a.clone();
                } else if $action.get_type().name() == "PinActions" {
                    let pin_actions = $action.extract::<PyRef<super::super::pins::pin_actions::PinActions>>().unwrap();
                    if pin_actions.actions.len() == 1 {
                        t = pin_actions.actions.first().unwrap().to_string();
                    } else {
                        return Err(pyo3::exceptions::ValueError::py_err(
                            "SymbolMap lookups can only retrieve single symbols at a time"
                        ))
                    }
                } else {
                    return super::super::type_error!(&format!(
                        "Cannot cast type {} to a valid PinAction",
                        $action.get_type().name()
                    ));
                }
                t
            }.as_str()
        )
    };
}

#[pyclass]
pub struct SymbolMap {
    timeset_id: usize,
    target_name: String,
}

impl SymbolMap {
    pub fn new(timeset_id: usize, target_name: String) -> Self {
        Self {
            timeset_id: timeset_id,
            target_name: target_name
        }
    }
}

#[pymethods]
impl SymbolMap {
    fn keys(&self) -> PyResult<Vec<String>> {
        let dut = DUT.lock().unwrap();
        let resolver = &dut.timesets[self.timeset_id].pin_action_resolvers[&self.target_name];
        Ok(resolver.mapping().iter().map(|(k, _)| k.to_string()).collect())
    }

    fn values(&self) -> PyResult<Vec<String>> {
        let dut = DUT.lock().unwrap();
        let resolver = &dut.timesets[self.timeset_id].pin_action_resolvers[&self.target_name];
        Ok(resolver.mapping().iter().map(|(_, v)| v.to_string()).collect::<Vec<String>>())
    }

    fn items(&self) -> PyResult<Vec<(String, String)>> {
        let dut = DUT.lock().unwrap();
        let resolver = &dut.timesets[self.timeset_id].pin_action_resolvers[&self.target_name];

        Ok(resolver.mapping().iter().map(
            |(k, v)| (k.to_string(), v.to_string())
        ).collect::<Vec<(String, String)>>())
    }

    fn get(&self, action: &PyAny) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        match self.__getitem__(action) {
            Ok(a) => Ok(a.into_py(py)),
            Err(_) => Ok(py.None())
        }
    }

    fn set_symbol(&mut self, action: &PyAny, new_resolution: String, target: Option<String>) -> PyResult<()> {
        if let Some(t) = target {
            let mut dut = DUT.lock().unwrap();
            let tset = &mut dut.timesets[self.timeset_id];
            if let Some(resolver) = tset.pin_action_resolvers.get_mut(&t) {
                resolver.update_mapping(
                    action_from_pyany!(action)?,
                    new_resolution.clone()
                );
                Ok(())
            } else {
                Err(pyo3::exceptions::KeyError::py_err(format!(
                    "Timeset '{}' does not have a symbol map targeting '{}' (The target must be set prior to timeset creation)",
                    tset.name,
                    t
                )))
            }
        } else {
            self.__setitem__(action, new_resolution)
        }
    }

    fn for_target(&self, target: String) -> PyResult<PyObject> {
        let dut = DUT.lock().unwrap();
        {
            let t = &dut.timesets[self.timeset_id];
            if !t.pin_action_resolvers.contains_key(&target) {
                return Err(pyo3::exceptions::KeyError::py_err(format!(
                    "Timeset '{}' does not have a symbol map targeting '{}' (The target must be set prior to timeset creation)",
                    t.name,
                    target
                )))
            }
        }

        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(Py::new(
            py,
            crate::timesets::timeset::SymbolMap {
                timeset_id: self.timeset_id,
                target_name: target
            },
        )
        .unwrap()
        .to_object(py))
    }
}

#[pyproto]
impl PyMappingProtocol for SymbolMap {

    fn __getitem__(&self, action: &PyAny) -> PyResult<String> {
        let dut = DUT.lock().unwrap();
        let resolver = &dut.timesets[self.timeset_id].pin_action_resolvers[&self.target_name];

        if let Some(r) = resolver.resolve(&action_from_pyany!(action)?) {
            Ok(r)
        } else {
            Err(pyo3::exceptions::KeyError::py_err(format!(
                "No symbol found for {}",
                action
            )))
        }
    }

    fn __setitem__(&mut self, action: &PyAny, new_resolution: String) -> PyResult<()> {
        let mut dut = DUT.lock().unwrap();
        let tester = origen::tester();
        for target in tester.targets_as_strs().iter() {
            // let resolver = &mut dut.timesets[self.timeset_id].pin_action_resolvers[&self.target_name];
            let resolver = &mut dut.timesets[self.timeset_id].pin_action_resolvers[target];
            resolver.update_mapping(
                action_from_pyany!(action)?,
                new_resolution.clone()
            )
        }
        Ok(())
    }

    fn __len__(&self) -> PyResult<usize> {
        let dut = DUT.lock().unwrap();
        let resolver = &dut.timesets[self.timeset_id].pin_action_resolvers[&self.target_name];
        Ok(resolver.mapping().len())
    }
}

#[pyproto]
impl pyo3::class::sequence::PySequenceProtocol for SymbolMap {
    fn __contains__(&self, item: &PyAny) -> PyResult<bool> {
        match pyo3::PyMappingProtocol::__getitem__(self, &item) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false)
        }
    }
}

#[pyclass]
pub struct SymbolMapIter {
    pub keys: Vec<String>,
    pub i: usize,
}

#[pyproto]
impl pyo3::class::iter::PyIterProtocol for SymbolMapIter {
    fn __iter__(slf: PyRefMut<Self>) -> PyResult<Py<Self>> {
        Ok(slf.into())
    }

    fn __next__(mut slf: PyRefMut<Self>) -> PyResult<Option<String>> {
        if slf.i >= slf.keys.len() {
            return Ok(None);
        }
        let name = slf.keys[slf.i].clone();
        slf.i += 1;
        Ok(Some(name))
    }
}

#[pyproto]
impl pyo3::class::iter::PyIterProtocol for SymbolMap {
    fn __iter__(slf: PyRefMut<Self>) -> PyResult<SymbolMapIter> {
      Ok(SymbolMapIter {
        keys: slf.keys().unwrap(),
        i: 0,
      })
    }
}
