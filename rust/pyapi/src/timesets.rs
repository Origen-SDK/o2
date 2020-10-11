use super::dut::PyDUT;
use origen::{DUT, TESTER};
use pyo3::prelude::*;
use origen::core::tester::TesterSource;

#[macro_export]
macro_rules! type_error {
    ($message:expr) => {
        Err(pyo3::exceptions::TypeError::py_err(format!("{}", $message)))
    };
}

#[macro_use]
pub mod timeset_container;
#[macro_use]
pub mod timeset;

use pyo3::types::{PyAny, PyDict};
use timeset::{Event, Timeset, Wave, WaveGroup, Wavetable, SymbolMap};
use timeset_container::{
    EventContainer, TimesetContainer, WaveContainer, WaveGroupContainer, WavetableContainer,
};

#[pymodule]
pub fn timesets(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<TimesetContainer>()?;
    m.add_class::<WavetableContainer>()?;
    m.add_class::<WaveGroupContainer>()?;
    m.add_class::<WaveContainer>()?;
    m.add_class::<EventContainer>()?;
    m.add_class::<Timeset>()?;
    m.add_class::<Wavetable>()?;
    m.add_class::<WaveGroup>()?;
    m.add_class::<Wave>()?;
    m.add_class::<Event>()?;
    m.add_class::<SymbolMap>()?;
    Ok(())
}

#[pymethods]
impl PyDUT {
    #[args(kwargs = "**")]
    fn add_timeset(
        &self,
        model_id: usize,
        name: &str,
        period: &PyAny,
        kwargs: Option<&PyDict>,
    ) -> PyResult<PyObject> {
        let mut dut = DUT.lock().unwrap();
        let tester = TESTER.lock().unwrap();

        dut.create_timeset(
            model_id,
            name,
            if let Ok(p) = period.extract::<String>() {
                Some(Box::new(p))
            } else if let Ok(p) = period.extract::<f64>() {
                Some(Box::new(p))
            } else if period.get_type().name() == "NoneType" {
                Option::None
            } else {
                return type_error!("Could not convert 'period' argument to String or NoneType!");
            },
            match kwargs {
                Some(args) => match args.get_item("default_period") {
                    Some(arg) => Some(arg.extract::<f64>()?),
                    None => Option::None,
                },
                None => Option::None,
            },
            {
                // if let Some(t) = tester.target_testers.first() {
                //     match t {
                //         TesterSource::Internal(tester_struct) => {
                //             tester_struct.pin_action_resolver()
                //         },
                //         _ => None
                //     }
                tester.target_testers.iter().map( |t| t).collect::<Vec<&TesterSource>>()
            },
        )?;

        let gil = Python::acquire_gil();
        let py = gil.python();
        let model = dut.get_mut_model(model_id)?;
        Ok(pytimeset!(py, model, model_id, name)?)
    }

    fn timeset(&self, model_id: usize, name: &str) -> PyResult<PyObject> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(model_id)?;
        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(pytimeset_or_pynone!(py, model, model_id, name))
    }

    fn timesets(&self, model_id: usize) -> PyResult<Py<TimesetContainer>> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(pytimeset_container!(py, model_id))
    }
}
