use origen::DUT;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyAny};
use origen::error::Error;
use super::timeset_container::{WavetableContainer, WaveContainer, EventContainer};

#[macro_export]
macro_rules! pytimeset {
  ($py:expr, $model:expr, $model_id:expr, $name:expr) => {
    if $model.contains_timeset($name) {
      Ok(Py::new($py, crate::timesets::timeset::Timeset {
        name: String::from($name),
        model_id: $model_id,
      }).unwrap().to_object($py))
    } else {
      // Note: Errors here shouldn't happen. Any errors that arise are either
      // bugs or from the user meta-programming their way into the backend DB.
      Err(PyErr::from(origen::error::Error::new(&format!("No timeset {} has been added on block {}", $name, $model.name))))
    }
  };
}

// Returns a (Python) Timeset or NoneType instance.
// Note: this does NOT return a Rust Option::None, but 
#[macro_export]
macro_rules! pytimeset_or_pynone {
  ($py:expr, $model:expr, $model_id:expr, $name:expr) => {
    if $model.contains_timeset($name) {
      Py::new($py, crate::timesets::timeset::Timeset {
        name: String::from($name),
        model_id: $model_id,
      }).unwrap().to_object($py)
    } else {
      $py.None()
    }
  };
}

#[macro_export]
macro_rules! pywavetable {
  ($py:expr, $timeset:expr, $t_id:expr, $name:expr) => {
    if $timeset.contains_wavetable($name) {
      Ok(Py::new($py, crate::timesets::timeset::Wavetable {
        name: String::from($name),
        model_id: $timeset.model_id,
        timeset_id: $timeset.id,
      }).unwrap().to_object($py))
    } else {
      // Note: Errors here shouldn't happen. Any errors that arise are either
      // bugs or from the user meta-programming their way into the backend DB.
      Err(PyErr::from(origen::error::Error::new(&format!("No wavetable {} has been added on block {}", $name, $timeset.name))))
    }
  };
}

#[macro_export]
macro_rules! pywave {
  ($py:expr, $wavetable:expr, $name:expr) => {
    if $wavetable.contains_wave($name) {
      Ok(Py::new($py, crate::timesets::timeset::Wave {
        name: String::from($name),
        model_id: $wavetable.model_id,
        timeset_id: $wavetable.timeset_id,
        wavetable_id: $wavetable.id,
      }).unwrap().to_object($py))
    } else {
      // Note: Errors here shouldn't happen. Any errors that arise are either
      // bugs or from the user meta-programming their way into the backend DB.
      Err(PyErr::from(origen::error::Error::new(&format!("No wave {} has been added on block {}", $name, $wavetable.name))))
    }
  };
}

#[macro_export]
macro_rules! pyevent {
  ($py:expr, $wave:expr, $event_index:expr) => {
    if $wave.events.len() > $event_index {
      Ok(Py::new($py, crate::timesets::timeset::Event {
        model_id: $wave.model_id,
        timeset_id: $wave.timeset_id,
        wavetable_id: $wave.wavetable_id,
        wave_id: $wave.wave_id,
        wave_name: $wave.name.clone(),
        index: $event_index,
      }).unwrap().to_object($py))
    } else {
      // Note: Errors here shouldn't happen. Any errors that arise are either
      // bugs or from the user meta-programming their way into the backend DB.
      Err(PyErr::from(origen::error::Error::new(&format!("No event at {} has been added on wave {}", $event_index, $wave.name))))
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
}

impl Timeset {
  pub fn new(name: &str, model_id: usize) -> Self {
    Self {
      name: String::from(name),
      model_id: model_id
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
  fn add_wave(&self, name: &str, _kwargs: Option<&PyDict>) -> PyResult<PyObject>  {
    let mut dut = DUT.lock().unwrap();
    let w_id;
    {
      w_id = dut.get_wavetable(self.timeset_id, &self.name).unwrap().id;
    }
    dut.create_wave(w_id, name)?;

    let gil = Python::acquire_gil();
    let py = gil.python();
    let wt = dut.get_wavetable(self.timeset_id, &self.name).unwrap();
    Ok(pywave!(py, wt, name)?)
  }

  #[getter]
  fn get_waves(&self) -> PyResult<Py<WaveContainer>> {
    let w_id;
    {
      w_id = self.get_origen_id()?;
    }
    let gil = Python::acquire_gil();
    let py = gil.python();
    Ok(pywave_container!(py, self.model_id, self.timeset_id, w_id, &self.name))
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
      model_id: model_id
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
pub struct Wave {
  pub model_id: usize,
  pub timeset_id: usize,
  pub wavetable_id: usize,
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
    Ok(pyevent_container!(py, self.model_id, self.timeset_id, self.wavetable_id, wave_id, &self.name))
  }

  #[args(event="**")]
  fn push_event(&self, event: Option<&PyDict>) -> PyResult<PyObject> {
    let mut dut = DUT.lock().unwrap();
    let (w_id, e_index);
    {
      w_id = dut.get_wave(self.wavetable_id, &self.name).unwrap().wave_id;
    }

    if event.is_none() {
      return type_error!("Keywords 'at' and 'action' are required to push a new event!");
    }

    let (at, unit, action) = (
      event.unwrap().get_item("at"), 
      event.unwrap().get_item("unit"),
      event.unwrap().get_item("action")
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
            return type_error!("'action' keyword is required (found None)!")
          } else {
            return type_error!("Could not interpret 'action' argument as String!")
          }
        },
        None => return type_error!("'action' keyword is required!")
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
              return type_error!("'at' keyword is required (found None)!")
            } else {
              return type_error!("Could not interpret 'at' argument as String or Numeric!");
            }
          },
          None => return type_error!("'at' keyword is required!")
        },

        // Resolve the 'unit' keyword. This is optional and can only be a string.
        match unit {
          Some(_unit) => {
            if let Ok(val) = _unit.extract::<String>() {
              Some(val)
            } else if _unit.is_none() {
              Option::None
            } else {
              return type_error!("Could not interpret 'unit' argument as String or NoneType!")
            }
          },
          None => Option::None
        },
        &temp,
      )?;
      e_index = e.event_index;
    }
    
    // Return the newly created event
    let gil = Python::acquire_gil();
    let py = gil.python();

    let w = dut.get_wave(self.wavetable_id, &self.name).unwrap();
    Ok(pyevent!(py, w, e_index)?)
  }

  #[getter]
  fn get_indicator(&self) -> PyResult<String> {
    let dut = DUT.lock().unwrap();
    let w = dut.get_wave(self.wavetable_id, &self.name).unwrap();
    Ok(w.indicator.clone())
  }

  #[setter]
  fn set_indicator(&self, indicator: &str) -> PyResult<()> {
    let mut dut = DUT.lock().unwrap();
    let w = dut.get_mut_wave(self.wavetable_id, &self.name).unwrap();
    w.set_indicator(&indicator)?;
    Ok(())
  }

  #[getter]
  fn get_applied_to(&self) -> PyResult<Vec<String>> {
    let dut = DUT.lock().unwrap();
    let w = dut.get_wave(self.wavetable_id, &self.name).unwrap();
    Ok(w.pins.clone())
  }

  #[args(pins="*")]
  fn apply_to(&self, pins: Vec<String>) -> PyResult<PyObject> {
    let mut dut = DUT.lock().unwrap();
    let w = dut.get_mut_wave(self.wavetable_id, &self.name).unwrap();
    w.apply_to(pins)?;

    let gil = Python::acquire_gil();
    let py = gil.python();
    Ok(Py::new(py, crate::timesets::timeset::Wave {
      name: self.name.clone(),
      model_id: self.model_id,
      timeset_id: self.timeset_id,
      wavetable_id: self.wavetable_id,
    }).unwrap().to_object(py))
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
    let w_id = wavetable.get_wave_id(&self.name).unwrap();
    Ok(w_id)
  }
}

#[pyclass]
pub struct EventList {
  pub model_id: usize,
  pub timeset_id: String,
  pub wavetable_id: String,
  pub wave_name: String,
}

#[pyclass]
pub struct Event {
  pub model_id: usize,
  pub timeset_id: usize,
  pub wavetable_id: usize,
  pub wave_id: usize,
  pub wave_name: String,
  pub index: usize,
}

#[pymethods]
impl Event {
  #[getter]
  pub fn action(&self) -> PyResult<String> {
    let dut = DUT.lock().unwrap();
    let e = dut.get_event(self.wave_id, self.index).unwrap();
    Ok(e.action.clone())
  }

  #[getter]
  pub fn unit(&self) -> PyResult<PyObject> {
    let dut = DUT.lock().unwrap();
    let e = dut.get_event(self.wave_id, self.index).unwrap();

    let gil = Python::acquire_gil();
    let py = gil.python();
    match &e.unit {
      Some(unit) => Ok(unit.clone().to_object(py)),
      None => Ok(py.None())
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
  pub fn new(model_id: usize, timeset_id:usize, wavetable_id: usize, wave_id: usize, wave_name: &str, id: usize) -> Self {
    Self {
      model_id: model_id,
      timeset_id: timeset_id,
      wavetable_id: wavetable_id,
      wave_id: wave_id,
      wave_name: String::from(wave_name),
      index: id,
    }
  }
}