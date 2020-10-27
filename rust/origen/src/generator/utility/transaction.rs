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
    pub address_width: Option<usize>,
    pub width: usize,
    pub data: BigUint,
    pub bit_enable: BigUint,
    pub capture_enable: Option<BigUint>,
    pub overlay_enable: Option<BigUint>,
    pub overlay_string: Option<String>,
    pub metadata: Option<Metadata>,
}

impl Default for Transaction {
    fn default() -> Self {
        Self {
            action: None,
            reg_id: None,
            address: None,
            address_width: None,
            width: 0,
            data: BigUint::from(0 as usize),
            bit_enable: BigUint::from(0 as usize),
            capture_enable: None,
            overlay_enable: None,
            overlay_string: None,
            metadata: None
        }
    }
}

impl Transaction {
    pub fn new_write(data: BigUint, width: usize) -> Result<Self> {
        Ok(Self {
            action: Some(Action::Write),
            reg_id: None,
            address: None,
            address_width: None,
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
            address_width: None,
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
            address_width: None,
            width: self.width,
            data: self.data.clone(),
            bit_enable: BigUint::from(0 as u8),
            capture_enable: Some(BigUint::from(0 as u8)),
            overlay_enable: self.overlay_enable.clone(),
            overlay_string: self.overlay_string.clone(),
            metadata: self.metadata.clone(),
        })
    }

    /// Shortcut function to generate a new transaction where the address
    /// of this transaction acts as the data of the new one.
    /// The address_width field will be the new transaction's width. If this
    /// is not provided, it will be taken from the default_addr_size.
    /// If neither widths are provided, or if the address width exceeds the resolved
    /// width, an error is returned.
    pub fn to_addr_trans(&self, default_addr_size: Option<usize>) -> Result<Self> {
        let mut t = Self::default();
        if self.address.is_none() {
            return Err(Error::new("Cannot create an address transaction from a transaction which does not have an address"));
        }
        if let Some(w) = self.address_width {
            t.width = w;
        } else if let Some(w) = default_addr_size {
            t.width = w;
        } else {
            return Err(Error::new("Could not create transaction from address as this transaction does not supply an address width nor was a default one provided"));
        }
        t.data = BigUint::from(self.address.unwrap());
        t.bit_enable = Self::enable_of_width(t.width)?;
        t.action = Some(Action::Write);
        Ok(t)
    }

    pub fn prepend_data(&mut self, data: BigUint, width: usize) -> Result<()> {
        self.data = (&self.data << width) + data;
        self.width += width;
        // Preserve existing bit enables
        self.bit_enable = (&self.bit_enable << width) + Self::enable_of_width(width)?;
        Ok(())
    }
}

impl NumHelpers for Transaction {
    fn even_parity(&self) -> bool {
        self.data.even_parity()
    }
}