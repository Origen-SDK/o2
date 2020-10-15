use super::super::super::dut::Dut;
use super::super::pins::Endianness;
use super::pin::PinAction;
use super::pin_store::PinStore;
use crate::error::Error;
use crate::Result;

// We'll maintain both the pin_names which the group was built with, but we'll also maintain the list
// of physical names. Even though we can resolve this later, most operations wil
#[derive(Debug, Clone)]
pub struct PinGroup {
    pub model_id: usize,
    pub id: usize,
    pub name: String,
    pub endianness: Endianness,
    pub pin_ids: Vec<usize>,
}

impl PinGroup {
    pub fn new(
        model_id: usize,
        id: usize,
        name: &str,
        pin_ids: Vec<usize>,
        endianness: Option<Endianness>,
    ) -> PinGroup {
        return PinGroup {
            model_id: model_id,
            id: id,
            name: name.to_string(),
            pin_ids: match endianness {
                Some(e) => match e {
                    Endianness::LittleEndian => pin_ids,
                    Endianness::BigEndian => {
                        let mut _pins = pin_ids.clone();
                        _pins.reverse();
                        _pins
                    }
                },
                None => pin_ids,
            },
            endianness: match endianness {
                Some(e) => e,
                None => Endianness::LittleEndian,
            },
        };
    }

    pub fn len(&self) -> usize {
        return self.pin_ids.len();
    }

    pub fn is_little_endian(&self) -> bool {
        match self.endianness {
            Endianness::LittleEndian => true,
            Endianness::BigEndian => false,
        }
    }

    pub fn is_big_endian(&self) -> bool {
        return !self.is_little_endian();
    }

    pub fn to_identifier(&self) -> (String, usize) {
        (self.name.clone(), self.model_id)
    }

    pub fn pin_names(&self, dut: &Dut) -> Result<Vec<String>> {
        let pc = super::super::pins::PinCollection::from_pin_group(&dut, self)?;
        Ok(pc.pin_names())
    }

    pub fn update(&self, dut: &Dut, trans: &crate::Transaction) -> Result<()> {
        let pc = super::super::pins::PinCollection::from_pin_group(&dut, self)?;
        pc.set_from_transaction(trans)?;
        Ok(())
    }

    pub fn get_actions(&self, dut: &Dut) -> Result<Vec<PinAction>> {
        let pc = super::super::pins::PinCollection::from_pin_group(&dut, self)?;
        Ok(pc.get_actions())
    }

    pub fn get_reset_actions(&self, dut: &Dut) -> Result<Vec<PinAction>> {
        let pc = super::super::pins::PinCollection::from_pin_group(&dut, self)?;
        Ok(pc.get_reset_actions())
    }

    pub fn reset(&self, dut: &Dut) -> Result<()> {
        let pc = super::super::pins::PinCollection::from_pin_group(&dut, self)?;
        pc.reset();
        Ok(())
    }

    pub fn set_action(&self, dut: &Dut, action: &PinAction) -> Result<()> {
        let pc = super::super::pins::PinCollection::from_pin_group(&dut, &self)?;
        pc.set_action(&action.to_string());
        Ok(())
    }

    pub fn set_actions(&self, dut: &Dut, actions: &Vec<PinAction>) -> Result<()> {
        let pc = super::super::pins::PinCollection::from_pin_group(&dut, &self)?;
        let mut v = vec!();
        actions.iter().for_each( |a| v.push(a.to_string()));
        pc.set_actions(&v)?;
        Ok(())
    }

    pub fn slice(
        &self,
        start_idx: usize,
        stop_idx: usize,
        step_size: usize,
    ) -> Result<PinStore> {
        let mut sliced_names: Vec<usize> = vec![];
        for i in (start_idx..=stop_idx).step_by(step_size) {
            if i >= self.pin_ids.len() {
                return Err(Error::new(&format!(
                    "Index {} exceeds available pins in collection! (length: {})",
                    i,
                    self.pin_ids.len()
                )));
            }
            let p = self.pin_ids[i].clone();
            sliced_names.push(p);
        }
        Ok(PinStore::new(sliced_names, Some(self.endianness)))
    }
}
