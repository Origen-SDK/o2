use super::super::super::dut::Dut;
use super::super::pins::Endianness;
use super::pin::PinAction;
use crate::error::Error;
use crate::Result;

/// Model for a collection (or group) of pins
#[derive(Debug, Clone)]
pub struct PinStore {
    // pub pin_names: Vec<String>,
    pub endianness: Endianness,
    pub mask: Option<usize>,
    // pub model_id: usize,
    pub pin_ids: Vec<usize>,
}

impl PinStore {
    // pub fn new(
    //     model_id: usize,
    //     pin_names: &Vec<String>,
    //     endianness: Option<Endianness>,
    // ) -> PinStore {
    //     PinStore {
    //         pin_names: match endianness {
    //             Some(e) => match e {
    //                 Endianness::LittleEndian => pin_names.iter().map(|p| String::from(p)).collect(),
    //                 Endianness::BigEndian => {
    //                     pin_names.iter().rev().map(|p| String::from(p)).collect()
    //                 }
    //             },
    //             None => pin_names.iter().map(|p| String::from(p)).collect(),
    //         },
    //         endianness: endianness.unwrap_or(Endianness::LittleEndian),
    //         mask: Option::None,
    //         model_id: model_id,
    //         pin_ids: vec!(),
    //     }
    // }

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
            mask: None,
            endianness: e
        }
    }

    pub fn from_grp_identifiers(dut: &Dut, grps: &Vec<(String, usize)>) -> Result<Self> {
        Ok(Self {
            // model_id: 0,
            // pin_names: vec!(),
            endianness: Endianness::LittleEndian,
            mask: None,
            pin_ids: dut.collect_grp_ids_as_pin_ids(grps)?
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
        // Ok(PinStore::new(
        //     self.model_id,
        //     &sliced_names,
        //     Option::Some(self.endianness),
        // ))
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
        let mut v = vec!();
        actions.iter().for_each( |a| v.push(a.to_string()));
        pc.set_actions(&v)?;
        Ok(())
    }

}

impl Dut {
    // pub fn drive_pin_store(
    //     &mut self,
    //     pin_store: &mut PinStore,
    //     data: Option<u32>,
    // ) -> Result<(), Error> {
    //     self.set_pin_store_actions(pin_store, PinActions::Drive, data)
    // }

    // pub fn verify_pin_store(
    //     &mut self,
    //     pin_store: &mut PinStore,
    //     data: Option<u32>,
    // ) -> Result<(), Error> {
    //     self.set_pin_store_actions(pin_store, PinActions::Verify, data)
    // }

    // pub fn capture_pin_store(
    //     &mut self,
    //     pin_store: &mut PinStore,
    // ) -> Result<(), Error> {
    //     self.set_pin_store_actions(pin_store, PinActions::Capture, Option::None)
    // }

    // pub fn highz_pin_store(
    //     &mut self,
    //     pin_store: &mut PinStore,
    // ) -> Result<(), Error> {
    //     self.set_pin_store_actions(pin_store, PinActions::HighZ, Option::None)
    // }

    // pub fn set_pin_store_actions(
    //     &mut self,
    //     collection: &mut PinStore,
    //     action: PinAction,
    //     data: Option<u32>,
    // ) -> Result<()> {
    //     let pin_names = &collection.pin_names;
    //     let mask = collection.mask;
    //     collection.mask = Option::None;
    //     self.set_pin_actions(collection.model_id, pin_names, action, data, mask, None)
    // }

    // pub fn set_per_pin_store_actions(
    //     &mut self,
    //     collection: &mut PinStore,
    //     actions: &Vec<PinAction>,
    // ) -> Result<()> {
    //     let pin_names = &collection.pin_names;
    //     let mask = collection.mask;
    //     collection.mask = Option::None;
    //     self.set_per_pin_actions(collection.model_id, pin_names, &actions, mask)
    // }

    // pub fn get_pin_store_data(&mut self, collection: &PinStore) -> Result<u32, Error> {
    //     let pin_names = &collection.pin_names;
    //     Ok(self.get_pin_data(&pin_names))
    // }

    // pub fn get_pin_store_reset_data(&self, collection: &PinStore) -> Result<u32, Error> {
    //     let pin_names = &collection.pin_names;
    //     self.get_pin_reset_data(collection.model_id, &pin_names)
    // }

    // pub fn get_pin_store_reset_actions(
    //     &self,
    //     collection: &PinStore,
    // ) -> Result<Vec<PinAction>, Error> {
    //     let pin_names = &collection.pin_names;
    //     self.get_pin_reset_actions(collection.model_id, &pin_names)
    // }

    // pub fn reset_pin_store(&mut self, collection: &PinStore) -> Result<(), Error> {
    //     let pin_names = &collection.pin_names;
    //     self.reset_pin_names(collection.model_id, &pin_names)
    // }

    // pub fn set_pin_store_data(
    //     &mut self,
    //     collection: &PinStore,
    //     data: u32,
    // ) -> Result<(), Error> {
    //     let pin_names = &collection.pin_names;
    //     self.set_pin_data(collection.model_id, &pin_names, data, collection.mask)
    // }

    pub fn set_pin_store_nonsticky_mask(
        &mut self,
        pin_store: &mut PinStore,
        mask: usize,
    ) -> Result<()> {
        pin_store.mask = Some(mask);
        Ok(())
    }
}
