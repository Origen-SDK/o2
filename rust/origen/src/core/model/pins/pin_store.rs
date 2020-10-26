use super::super::super::dut::Dut;
use super::super::pins::Endianness;
use super::pin::PinActions;
use crate::error::Error;

/// Model for a collection (or group) of pins
#[derive(Debug, Clone)]
pub struct PinStore {
    pub pin_names: Vec<String>,
    pub endianness: Endianness,
    pub mask: Option<usize>,
    pub model_id: usize,
}

impl PinStore {
    pub fn new(
        model_id: usize,
        pin_names: &Vec<String>,
        endianness: Option<Endianness>,
    ) -> PinStore {
        PinStore {
            pin_names: match endianness {
                Some(e) => match e {
                    Endianness::LittleEndian => pin_names.iter().map(|p| String::from(p)).collect(),
                    Endianness::BigEndian => {
                        pin_names.iter().rev().map(|p| String::from(p)).collect()
                    }
                },
                None => pin_names.iter().map(|p| String::from(p)).collect(),
            },
            endianness: endianness.unwrap_or(Endianness::LittleEndian),
            mask: Option::None,
            model_id: model_id,
        }
    }

    pub fn len(&self) -> usize {
        self.pin_names.len()
    }

    pub fn slice_names(
        &self,
        start_idx: usize,
        stop_idx: usize,
        step_size: usize,
    ) -> Result<PinStore, Error> {
        let mut sliced_names: Vec<String> = vec![];
        for i in (start_idx..=stop_idx).step_by(step_size) {
            if i >= self.pin_names.len() {
                return Err(Error::new(&format!(
                    "Index {} exceeds available pins in collection! (length: {})",
                    i,
                    self.pin_names.len()
                )));
            }
            let p = self.pin_names[i].clone();
            sliced_names.push(p);
        }
        Ok(PinStore::new(
            self.model_id,
            &sliced_names,
            Option::Some(self.endianness),
        ))
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
}

impl Dut {
    pub fn drive_pin_store(
        &mut self,
        pin_store: &mut PinStore,
        data: Option<u32>,
    ) -> Result<(), Error> {
        self.set_pin_store_actions(pin_store, PinActions::Drive, data)
    }

    pub fn verify_pin_store(
        &mut self,
        pin_store: &mut PinStore,
        data: Option<u32>,
    ) -> Result<(), Error> {
        self.set_pin_store_actions(pin_store, PinActions::Verify, data)
    }

    pub fn capture_pin_store(&mut self, pin_store: &mut PinStore) -> Result<(), Error> {
        self.set_pin_store_actions(pin_store, PinActions::Capture, Option::None)
    }

    pub fn highz_pin_store(&mut self, pin_store: &mut PinStore) -> Result<(), Error> {
        self.set_pin_store_actions(pin_store, PinActions::HighZ, Option::None)
    }

    pub fn set_pin_store_actions(
        &mut self,
        collection: &mut PinStore,
        action: PinActions,
        data: Option<u32>,
    ) -> Result<(), Error> {
        let pin_names = &collection.pin_names;
        let mask = collection.mask;
        collection.mask = Option::None;
        self.set_pin_actions(collection.model_id, pin_names, action, data, mask, None)
    }

    pub fn set_per_pin_store_actions(
        &mut self,
        collection: &mut PinStore,
        actions: &Vec<PinActions>,
    ) -> Result<(), Error> {
        let pin_names = &collection.pin_names;
        let mask = collection.mask;
        collection.mask = Option::None;
        self.set_per_pin_actions(collection.model_id, pin_names, &actions, mask)
    }

    // pub fn get_pin_store_data(&mut self, collection: &PinStore) -> Result<u32, Error> {
    //     let pin_names = &collection.pin_names;
    //     Ok(self.get_pin_data(&pin_names))
    // }

    pub fn get_pin_store_reset_data(&self, collection: &PinStore) -> Result<u32, Error> {
        let pin_names = &collection.pin_names;
        self.get_pin_reset_data(collection.model_id, &pin_names)
    }

    pub fn get_pin_store_reset_actions(
        &self,
        collection: &PinStore,
    ) -> Result<Vec<PinActions>, Error> {
        let pin_names = &collection.pin_names;
        self.get_pin_reset_actions(collection.model_id, &pin_names)
    }

    pub fn reset_pin_store(&mut self, collection: &PinStore) -> Result<(), Error> {
        let pin_names = &collection.pin_names;
        self.reset_pin_names(collection.model_id, &pin_names)
    }

    pub fn set_pin_store_data(&mut self, collection: &PinStore, data: u32) -> Result<(), Error> {
        let pin_names = &collection.pin_names;
        self.set_pin_data(collection.model_id, &pin_names, data, collection.mask)
    }

    pub fn set_pin_store_nonsticky_mask(
        &mut self,
        pin_store: &mut PinStore,
        mask: usize,
    ) -> Result<(), Error> {
        pin_store.mask = Some(mask);
        Ok(())
    }
}
