use crate::error::Error;
use super::super::super::dut::Dut;
use super::super::super::model::Model;

#[derive(Debug, Clone)]
pub struct PinHeader {
    pub model_id: usize,
    pub id: usize,
    pub name: String,
    pub pin_names: Vec<String>,
}

impl PinHeader {
  pub fn new(
    model_id: usize,
    id: usize,
    name: &str,
    pins: Vec<String>,
  ) -> Self {
    return Self {
        model_id: model_id,
        id: id,
        name: String::from(name),
        pin_names: pins,
    };
  }
}

impl Dut {
  pub fn create_pin_header(&mut self, model_id: usize, name: &str, pins: Vec<String>) -> Result<&PinHeader, Error> {
    let id;
    {
      id = self.pin_headers.len();
    }
    let _pnames = self.verify_names(model_id, &pins)?;
    let model = &mut self.models[model_id];
    self.pin_headers.push(model.register_pin_header(id, name, pins)?);
    Ok(&self.pin_headers[id])
  }
}

impl Model {
  pub fn register_pin_header(&mut self, pid: usize, name: &str, pin_names: Vec<String>) -> Result<PinHeader, Error> {
    let header = PinHeader::new(self.id, pid, name, pin_names);
    self.pin_headers.insert(name.to_string(), pid);
    Ok(header)
  }

  pub fn get_pin_header_id(&self, name: &str) -> Option<usize> {
    match self.pin_headers.get(name) {
      Some(t) => Some(*t),
      None => None,
    }
  }
}
