use super::super::super::dut::Dut;
use super::super::pins::Endianness;
use super::pin::PinAction;
use crate::error::Error;
use crate::Result;

/// Model for an anonymous pin group
#[derive(Debug, Clone)]
pub struct PinStore {
    pub endianness: Endianness,
    pub pin_ids: Vec<usize>,
}

impl PinStore {
    pub fn new(pin_ids: Vec<usize>, endianness: Option<Endianness>) -> Self {
        let e = endianness.unwrap_or(Endianness::LittleEndian);
        Self {
            pin_ids: {
                match e {
                    Endianness::LittleEndian => pin_ids,
                    Endianness::BigEndian => {
                        let mut _pins = pin_ids.clone();
                        _pins.reverse();
                        _pins
                    }
                }
            },
            endianness: e,
        }
    }

    pub fn from_grp_identifiers(dut: &Dut, grps: &Vec<(String, usize)>) -> Result<Self> {
        Ok(Self {
            endianness: Endianness::LittleEndian,
            pin_ids: dut.collect_grp_ids_as_pin_ids(grps)?,
        })
    }

    pub fn len(&self) -> usize {
        self.pin_ids.len()
    }

    pub fn slice_names(
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
        Ok(Self::new(sliced_names, Some(self.endianness)))
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

    pub fn pin_names(&self, dut: &Dut) -> Result<Vec<String>> {
        let pc = super::super::pins::PinCollection::from_pin_store(&dut, &self)?;
        Ok(pc.pin_names())
    }

    pub fn contains_identifier(&self, dut: &Dut, identifier: (usize, String)) -> Result<bool> {
        let pc = super::super::pins::PinCollection::from_pin_store(&dut, &self)?;
        pc.contains_group_identifier(&dut, identifier)
    }

    pub fn update(&self, dut: &Dut, trans: &crate::Transaction) -> Result<()> {
        let pc = super::super::pins::PinCollection::from_pin_store(&dut, &self)?;
        pc.set_from_transaction(trans)?;
        Ok(())
    }

    pub fn get_actions(&self, dut: &Dut) -> Result<Vec<PinAction>> {
        let pc = super::super::pins::PinCollection::from_pin_store(&dut, &self)?;
        Ok(pc.get_actions())
    }

    pub fn get_reset_actions(&self, dut: &Dut) -> Result<Vec<PinAction>> {
        let pc = super::super::pins::PinCollection::from_pin_store(&dut, &self)?;
        Ok(pc.get_reset_actions())
    }

    pub fn reset(&self, dut: &Dut) -> Result<()> {
        let pc = super::super::pins::PinCollection::from_pin_store(&dut, &self)?;
        pc.reset();
        Ok(())
    }

    pub fn set_action(&self, dut: &Dut, action: &PinAction) -> Result<()> {
        let pc = super::super::pins::PinCollection::from_pin_store(&dut, &self)?;
        pc.set_action(&action.to_string());
        Ok(())
    }

    pub fn set_actions(&self, dut: &Dut, actions: &Vec<PinAction>) -> Result<()> {
        let pc = super::super::pins::PinCollection::from_pin_store(&dut, &self)?;
        let mut v = vec![];
        actions.iter().for_each(|a| v.push(a.to_string()));
        pc.set_actions(&v)?;
        Ok(())
    }
}
