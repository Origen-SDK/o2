use super::dut::PyDUT;
use origen::core::tester::TesterSource;
use origen::{DUT, TESTER};
use pyo3::prelude::*;

#[macro_export]
macro_rules! type_error {
    ($message:expr) => {
        Err(pyo3::exceptions::PyTypeError::new_err(format!(
            "{}",
            $message
        )))
    };
}

#[macro_use]
pub mod timeset_container;
#[macro_use]
pub mod timeset;

use pyo3::types::{PyAny, PyDict};
use timeset::{Event, SymbolMap, Timeset, Wave, WaveGroup, Wavetable};
use timeset_container::{
    EventContainer, TimesetContainer, WaveContainer, WaveGroupContainer, WavetableContainer,
};

pub fn define(py: Python, m: &PyModule) -> PyResult<()> {
    let subm = PyModule::new(py, "timesets")?;
    subm.add_class::<TimesetContainer>()?;
    subm.add_class::<WavetableContainer>()?;
    subm.add_class::<WaveGroupContainer>()?;
    subm.add_class::<WaveContainer>()?;
    subm.add_class::<EventContainer>()?;
    subm.add_class::<Timeset>()?;
    subm.add_class::<Wavetable>()?;
    subm.add_class::<WaveGroup>()?;
    subm.add_class::<Wave>()?;
    subm.add_class::<Event>()?;
    subm.add_class::<SymbolMap>()?;
    m.add_submodule(subm)?;
    Ok(())
}

#[pymethods]
impl PyDUT {
    #[args(kwargs = "**")]
    fn add_timeset(
        &self,
        py: Python,
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
            } else if period.get_type().name()? == "NoneType" {
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
                tester
                    .target_testers
                    .iter()
                    .map(|t| t)
                    .collect::<Vec<&TesterSource>>()
            },
        )?;

        let model = dut.get_mut_model(model_id)?;
        Ok(pytimeset!(py, model, model_id, name)?)
    }

    fn timeset(&self, py: Python, model_id: usize, name: &str) -> PyResult<PyObject> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(model_id)?;
        Ok(pytimeset_or_pynone!(py, model, model_id, name))
    }

    fn timesets(&self, py: Python, model_id: usize) -> PyResult<Py<TimesetContainer>> {
        Ok(pytimeset_container!(py, model_id))
    }
}
