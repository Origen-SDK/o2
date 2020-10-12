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
    // pub pin_names: Vec<String>,
    pub endianness: Endianness,
    pub mask: Option<usize>,
    pub pin_ids: Vec<usize>,
}

impl PinGroup {
    pub fn new(
        model_id: usize,
        id: usize,
        name: &str,
        // pins: Vec<String>,
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
                        // pin_ids.reverse();
                        // pin_ids
                    }
                },
                None => pin_ids,
            },
            endianness: match endianness {
                Some(e) => e,
                None => Endianness::LittleEndian,
            },
            mask: Option::None,
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
        // if let Some(p) = self.get_pin_group(model_id, name) {
        //     let names = &p.pin_names;
        //     let mut sliced_names: Vec<String> = vec![];

        //     for i in (start_idx..stop_idx).step_by(step_size) {
        //         if i >= names.len() {
        //             return Err(Error::new(&format!(
        //                 "Index {} exceeds available pins in group {} (length: {})",
        //                 i,
        //                 name,
        //                 names.len()
        //             )));
        //         }
        //         let p = names[i].clone();
        //         sliced_names.push(p);
        //     }
        //     Ok(PinStore::new(model_id, &sliced_names, Option::None))
        // } else {
        //     Err(Error::new(&format!(
        //         "Could not slice pin group {} because it doesn't exists!",
        //         name
        //     )))
        // }
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
        Ok(PinStore::new(sliced_names, Some(self.endianness)))
    }
}

impl Dut {
    // pub fn get_pin_group_data(&self, model_id: usize, name: &str) -> Result<u32, Error> {
    //     let pin_names = &self._get_pin_group(model_id, name)?.pin_names;
    //     self.get_pin_data(model_id, pin_names)
    // }

    // pub fn get_pin_group_reset_data(&self, model_id: usize, name: &str) -> Result<u32, Error> {
    //     let pin_names = &self._get_pin_group(model_id, name)?.pin_names;
    //     self.get_pin_reset_data(model_id, &pin_names)
    // }

    // pub fn reset_pin_group(&mut self, model_id: usize, name: &str) -> Result<(), Error> {
    //     let pin_names = self._get_pin_group(model_id, name)?.pin_names.clone();
    //     self.reset_pin_names(model_id, &pin_names)
    // }

    // pub fn set_pin_group_data(
    //     &mut self,
    //     model_id: usize,
    //     name: &str,
    //     data: u32,
    // ) -> Result<(), Error> {
    //     let grp = self._get_mut_pin_group(model_id, name)?;
    //     let m = grp.mask;
    //     let pin_names = grp.pin_names.clone();
    //     grp.mask = None;
    //     self.set_pin_data(model_id, &pin_names, data, m)
    // }

    // pub fn resolve_pin_group_names(
    //     &self,
    //     model_id: usize,
    //     name: &str,
    // ) -> Result<Vec<String>, Error> {
    //     let pin_names = self._get_pin_group(model_id, name)?.pin_names.clone();
    //     self.resolve_pin_names(model_id, &pin_names)
    // }

    // /// Returns the pin actions as a string.
    // /// E.g.: for an 8-pin bus where the two MSBits are driving, the next two are capturing, then next wo are verifying, and the
    // ///   two LSBits are HighZ, the return value will be "DDCCVVZZ"
    // pub fn get_pin_group_actions(&self, model_id: usize, name: &str) -> Result<Vec<PinAction>, Error> {
    //     let pin_names = self._get_pin_group(model_id, name)?.pin_names.clone();
    //     self.get_pin_actions(model_id, &pin_names)
    // }

    // pub fn get_pin_group_reset_actions(
    //     &self,
    //     model_id: usize,
    //     name: &str,
    // ) -> Result<Vec<PinAction>, Error> {
    //     let pin_names = self._get_pin_group(model_id, name)?.pin_names.clone();
    //     self.get_pin_reset_actions(model_id, &pin_names)
    // }

    // pub fn set_pin_group_actions(
    //     &mut self,
    //     model_id: usize,
    //     name: &str,
    //     action: PinAction,
    //     data: Option<u32>,
    //     mask: Option<usize>,
    // ) -> Result<(), Error> {
    //     let grp = self._get_mut_pin_group(model_id, name)?;
    //     let grp_id;
    //     {
    //         grp_id = grp.id;
    //     }
    //     let pin_names = grp.pin_names.clone();
    //     let m;
    //     if let Some(_m) = mask {
    //         m = Some(_m);
    //     } else {
    //         if let Some(_m) = grp.mask {
    //             m = Some(_m);
    //         } else {
    //             m = None;
    //         }
    //     }
    //     grp.mask = None;
    //     self.set_pin_actions(model_id, &pin_names, action, data, m, Some(grp_id))
    // }

    // pub fn set_pin_group_symbols(
    //     &mut self,
    //     model_id: usize,
    //     name: &str,
    //     symbols: &Vec<PinAction>,
    //     mask: Option<usize>
    // ) -> Result<(), Error> {
    //     let grp = self._get_mut_pin_group(model_id, name)?;
    //     let pin_names = grp.pin_names.clone();
    //     let m;
    //     if let Some(_m) = mask {
    //         m = Some(_m);
    //     } else {
    //         if let Some(_m) = grp.mask {
    //             m = Some(_m);
    //         } else {
    //             m = None;
    //         }
    //     }
    //     grp.mask = None;
    //     self.set_per_pin_actions(
    //         model_id,
    //         &pin_names,
    //         symbols,
    //         m,
    //     )
    // }
    // pub fn drive_pin_group(
    //     &mut self,
    //     model_id: usize,
    //     group_name: &str,
    //     data: Option<u32>,
    //     mask: Option<usize>,
    // ) -> Result<(), Error> {
    //     return self.set_pin_group_actions(model_id, group_name, PinActions::Drive, data, mask);
    // }

    // pub fn verify_pin_group(
    //     &mut self,
    //     model_id: usize,
    //     group_name: &str,
    //     data: Option<u32>,
    //     mask: Option<usize>,
    // ) -> Result<(), Error> {
    //     return self.set_pin_group_actions(model_id, group_name, PinActions::Verify, data, mask);
    // }

    // pub fn capture_pin_group(
    //     &mut self,
    //     model_id: usize,
    //     group_name: &str,
    //     mask: Option<usize>,
    // ) -> Result<(), Error> {
    //     return self.set_pin_group_actions(
    //         model_id,
    //         group_name,
    //         PinActions::Capture,
    //         Option::None,
    //         mask,
    //     );
    // }

    // pub fn highz_pin_group(
    //     &mut self,
    //     model_id: usize,
    //     group_name: &str,
    //     mask: Option<usize>,
    // ) -> Result<(), Error> {
    //     return self.set_pin_group_actions(
    //         model_id,
    //         group_name,
    //         PinActions::HighZ,
    //         Option::None,
    //         mask,
    //     );
    // }

    // Assume the pin group is properly defined (that is, not pin duplicates and all pins exists. If the pin group exists, these should both be met)
    // pub fn slice_pin_group(
    //     &self,
    //     model_id: usize,
    //     name: &str,
    //     start_idx: usize,
    //     stop_idx: usize,
    //     step_size: usize,
    // ) -> Result<PinStore, Error> {
    //     if let Some(p) = self.get_pin_group(model_id, name) {
    //         let names = &p.pin_names;
    //         let mut sliced_names: Vec<String> = vec![];

    //         for i in (start_idx..stop_idx).step_by(step_size) {
    //             if i >= names.len() {
    //                 return Err(Error::new(&format!(
    //                     "Index {} exceeds available pins in group {} (length: {})",
    //                     i,
    //                     name,
    //                     names.len()
    //                 )));
    //             }
    //             let p = names[i].clone();
    //             sliced_names.push(p);
    //         }
    //         Ok(PinStore::new(model_id, &sliced_names, Option::None))
    //     } else {
    //         Err(Error::new(&format!(
    //             "Could not slice pin group {} because it doesn't exists!",
    //             name
    //         )))
    //     }
    // }

    pub fn set_pin_group_nonsticky_mask(&mut self, model_id: usize, name: &str, mask: usize) -> Result<()> {
        let grp = self._get_mut_pin_group(model_id, name)?;
        grp.mask = Some(mask);
        Ok(())
    }
}
