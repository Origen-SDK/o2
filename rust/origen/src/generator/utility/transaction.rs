use crate::{Result, Error, Metadata};
use num_bigint::BigUint;
use num_traits::pow::Pow;
use super::super::nodes::Id;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum Action {
    Write,
    Verify,
    Capture
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Transaction {
    pub action: Option<Action>, // Can keep this as None for a generalized transaction
    pub reg_id: Option<Id>,
    pub address: Option<u128>,
    pub width: usize,
    pub data: BigUint,
    pub write_enable: Option<BigUint>,
    pub verify_enable: Option<BigUint>,
    pub capture_enable: Option<BigUint>,
    pub overlay_enable: Option<BigUint>,
    pub overlay_string: Option<String>,
    pub metadata: Option<Metadata>,
}

impl Transaction {
    pub fn new_write(data: BigUint, width: usize) -> Result<Self> {
        Ok(Self {
            action: Some(Action::Write),
            reg_id: None,
            address: None,
            width: width,
            data: data,
            write_enable: Some(BigUint::from(2 as u32).pow(width as u32) - (1 as u32)),
            verify_enable: None,
            capture_enable: None,
            overlay_enable: None,
            overlay_string: None,
            metadata: None
        })
    }

    pub fn new_write_with_addr(data: BigUint, width: usize, addr: u128) -> Result<Self> {
        let mut t = Self::new_write(data, width)?;
        t.address = Some(addr);
        Ok(t)
    }

    pub fn new_verify(data: BigUint, width: usize) -> Result<Self> {
        Ok(Self {
            action: Some(Action::Verify),
            reg_id: None,
            address: None,
            width: width,
            data: data,
            write_enable: None,
            verify_enable: Some(Self::enable_of_width(width)?),
            capture_enable: None,
            overlay_enable: None,
            overlay_string: None,
            metadata: None
        })
    }

    pub fn addr(&self) -> Result<u128> {
        match self.address {
            Some(a) => Ok(a),
            None => Err(Error::new(&format!(
                "Tried to retrieve address from transaction {:?}, but an address has not be set",
                self
            )))
        }
    }

    pub fn enable_of_width(width: usize) -> Result<BigUint> {
        Ok(BigUint::from(2 as u32).pow(width) - (1 as u8))
    }

    /// Helper method to generate a mask which enables all bits in the transaction
    pub fn enable_width(&self) -> Result<BigUint> {
        Self::enable_of_width(self.width)
    }

    // Creates a dummy transaction from this transaction
    // That is, keeping everything else the same, changes
    // write, verify, and capture enables to be 0 (don't cares)
    // Note: overlays remain the same
    // Also note that the original is untouched, as well as the DUT
    pub fn to_dummy(&self) -> crate::Result<Self> {
        Ok(Self {
            action: self.action.clone(),
            reg_id: self.reg_id.clone(),
            address: self.address,
            width: self.width,
            data: self.data.clone(),
            write_enable: Some(BigUint::from(0 as u8)),
            verify_enable: Some(BigUint::from(0 as u8)),
            capture_enable: Some(BigUint::from(0 as u8)),
            overlay_enable: self.overlay_enable.clone(),
            overlay_string: self.overlay_string.clone(),
            metadata: self.metadata.clone(),
        })
    }
}