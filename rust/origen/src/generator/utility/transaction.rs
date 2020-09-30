use crate::{Result, Error, Metadata};
use num_bigint::BigUint;
use num_traits::pow::Pow;
use super::super::nodes::Id;
use crate::standards::actions::*;
use num_traits;
use crate::utility::num_helpers::NumHelpers;

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
    pub bit_enable: BigUint,
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
            bit_enable: Self::enable_of_width(width)?,
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
            bit_enable: Self::enable_of_width(width)?,
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

    pub fn to_symbols(&self) -> Result<Vec<&str>> {
        let low_sym;
        let high_sym;
        if let Some(action) = &self.action {
            match action {
                Action::Write => {
                    low_sym = DRIVE_LOW;
                    high_sym = DRIVE_HIGH;
                },
                Action::Verify => {
                    low_sym = VERIFY_LOW;
                    high_sym = VERIFY_HIGH;
                },
                Action::Capture => {
                    low_sym = CAPTURE;
                    high_sym = CAPTURE;
                }
            }
        } else {
            low_sym = HIGHZ;
            high_sym = HIGHZ;
        }

        let mut bits: Vec<&str> = Vec::with_capacity(self.width);
        let enables = self.bit_enable.clone();
        let t = BigUint::from(1 as u8);
        for i in 0..self.width {
            if ((&enables >> i) & &t) == t {
                if ((&self.data >> i) & &t) == t {
                    bits.push(high_sym);
                } else {
                    bits.push(low_sym);
                }
            } else {
                bits.push(HIGHZ);
            }
        }
        // Should probably add this
        // if !lsb_first {
        //     bits.reverse();
        // }
        Ok(bits)
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
            bit_enable: BigUint::from(0 as u8),
            capture_enable: Some(BigUint::from(0 as u8)),
            overlay_enable: self.overlay_enable.clone(),
            overlay_string: self.overlay_string.clone(),
            metadata: self.metadata.clone(),
        })
    }
}

impl NumHelpers for Transaction {
    fn even_parity(&self) -> bool {
        self.data.even_parity()
    }
}