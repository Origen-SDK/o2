use super::super::pins::Endianness;
use crate::error::Error;

/// Model for a collection (or group) of pins
#[derive(Debug, Clone)]
pub struct PinCollection {
    pub ids: Vec<String>,
    pub endianness: Endianness,
    pub path: String,
    pub mask: Option<usize>,
    pub model_id: usize,
}

impl PinCollection {
    pub fn new(
        model_id: usize,
        path: &str,
        pin_ids: &Vec<String>,
        endianness: Option<Endianness>,
    ) -> PinCollection {
        PinCollection {
            path: path.to_string(),
            ids: match endianness {
                Some(e) => match e {
                    Endianness::LittleEndian => pin_ids.iter().map(|p| String::from(p)).collect(),
                    Endianness::BigEndian => {
                        pin_ids.iter().rev().map(|p| String::from(p)).collect()
                    }
                },
                None => pin_ids.iter().map(|p| String::from(p)).collect(),
            },
            endianness: endianness.unwrap_or(Endianness::LittleEndian),
            mask: Option::None,
            model_id: model_id,
        }
    }

    pub fn len(&self) -> usize {
        self.ids.len()
    }

    pub fn slice_ids(
        &self,
        start_idx: usize,
        stop_idx: usize,
        step_size: usize,
    ) -> Result<PinCollection, Error> {
        let mut sliced_ids: Vec<String> = vec![];
        for i in (start_idx..=stop_idx).step_by(step_size) {
            if i >= self.ids.len() {
                return Err(Error::new(&format!(
                    "Index {} exceeds available pins in collection! (length: {})",
                    i,
                    self.ids.len()
                )));
            }
            let p = self.ids[i].clone();
            sliced_ids.push(p);
        }
        Ok(PinCollection::new(
            self.model_id,
            &self.path,
            &sliced_ids,
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
