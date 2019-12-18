use origen::DUT;
use origen::error::Error;
use pyo3::prelude::*;
#[allow(unused_imports)]
use pyo3::types::{PyDict, PyList, PyTuple, PyIterator, PyAny, PyBytes};
use origen::core::model::pins::pin_collection::PinCollection as OrigenPinCollection;

#[pyclass]
pub struct PinCollection {
    path: String,
    pin_collection: OrigenPinCollection
}

impl PinCollection{
  pub fn new(path: &str, ids: Vec<String>) -> Result<PinCollection, Error> {
    Ok(PinCollection {
        path: String::from(path),
        pin_collection: OrigenPinCollection::new(
          path,
          &ids,
          Option::None
        ),
    })
  }
}

#[pymethods]
impl PinCollection {
    #[getter]
    fn get_data(&self) -> PyResult<u32> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(&self.pin_collection.path)?;
        Ok(model.get_pin_data(&self.pin_collection.ids))
    }

    #[setter]
    fn set_data(&self, data: u32) -> PyResult<()> {
        let mut dut = DUT.lock().unwrap();
        let model = dut.get_mut_model(&self.path)?;
        model.set_pin_data(&self.pin_collection.ids, data)?;
        Ok(())
    }

    fn set(&self, data: u32) -> PyResult<()> {
        return self.set_data(data);
    }

    #[getter]
    fn get_pin_actions(&self) -> PyResult<String> {
      let mut dut = DUT.lock().unwrap();
      let model = dut.get_mut_model(&self.path)?;
      Ok(model.get_pin_actions(&self.pin_collection.ids)?)
    }

    fn drive(&self, data: Option<u32>) -> PyResult<()> {
      let mut dut = DUT.lock().unwrap();
      let model = dut.get_mut_model(&self.path)?;
      model.drive_pins(&self.pin_collection.ids, data)?;
      Ok(())
    }

    fn verify(&self, data: Option<u32>) -> PyResult<()> {
      let mut dut = DUT.lock().unwrap();
      let model = dut.get_mut_model(&self.path)?;
      model.verify_pins(&self.pin_collection.ids, data)?;
      Ok(())
    }

    fn capture(&self) -> PyResult<()> {
      let mut dut = DUT.lock().unwrap();
      let model = dut.get_mut_model(&self.path)?;
      model.capture_pins(&self.pin_collection.ids)?;
      Ok(())
    }

    fn highz(&self) -> PyResult<()> {
      let mut dut = DUT.lock().unwrap();
      let model = dut.get_mut_model(&self.path)?;
      model.highz_pins(&self.pin_collection.ids)?;
      Ok(())
    }

    #[getter]
    fn get_ids(&self) -> PyResult<Vec<String>> {
        Ok(self.pin_collection.ids.clone())
    }

    #[allow(non_snake_case)]
    #[getter]
    fn get__path(&self) -> PyResult<String> {
        Ok(self.pin_collection.path.clone())
    }
}

#[pyproto]
impl pyo3::class::sequence::PySequenceProtocol for PinCollection {
    fn __len__(&self) -> PyResult<usize> {
        Ok(self.pin_collection.len())
    }

    // fn __contains__(&self, item: &str) -> PyResult<bool> {
    //     let mut dut = DUT.lock().unwrap();
    //     let model = dut.get_mut_model(&self.path)?;
    //     Ok(model.pin_group_contains_pin(&self.id, item))
    //     // let grp = model.pin_group(&self.id);
    //     // match grp {
    //     //     Some(_grp) => {
    //     //         Ok(_grp.contains_pin(model, item))
    //     //         //Ok(true)
    //     //     },
    //     //     None => Err(exceptions::KeyError::py_err(format!("No pin group or pin group alias found for {}", self.id)))
    //     // }
    //     //Ok(true)
    // }
}