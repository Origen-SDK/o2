use crate::generator::PAT;
use crate::Result;
use num_bigint::BigUint;
use origen_metal::ast::Node;

fn err_msg(obj: &str, field: &str) -> String {
    format!(
        "Tried to retrieve {}'s field {} but this field has not been set",
        obj, field
    )
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Capture {
    pub symbol: Option<String>,
    pub cycles: Option<usize>,
    pub enables: Option<BigUint>,
    pub pin_ids: Option<Vec<usize>>,
}

impl Capture {
    pub fn default() -> Self {
        Self {
            symbol: None,
            cycles: None,
            enables: None,
            pin_ids: None,
        }
    }

    pub fn new(
        symbol: Option<String>,
        cycles: Option<usize>,
        enables: Option<BigUint>,
        pin_ids: Option<Vec<usize>>,
    ) -> Result<Self> {
        Ok(Self {
            symbol: symbol,
            cycles: cycles,
            enables: enables,
            pin_ids: pin_ids,
        })
    }

    pub fn placeholder(
        symbol: Option<String>,
        cycles: Option<usize>,
        enables: Option<BigUint>,
    ) -> Self {
        Self {
            symbol: symbol,
            cycles: cycles,
            enables: enables,
            pin_ids: None,
        }
    }

    pub fn to_node(&self) -> Node<PAT> {
        node!(PAT::Capture, self.clone(), None)
    }

    pub fn get_symbol(&self) -> Result<&String> {
        if let Some(s) = self.symbol.as_ref() {
            Ok(s)
        } else {
            bail!(&err_msg("capture", "symbol"))
        }
    }

    pub fn get_cycles(&self) -> Result<usize> {
        if let Some(c) = self.cycles {
            Ok(c)
        } else {
            bail!(&err_msg("capture", "cycles"))
        }
    }

    pub fn get_enables(&self) -> Result<&BigUint> {
        if let Some(e) = self.enables.as_ref() {
            Ok(e)
        } else {
            bail!(&err_msg("capture", "enables"))
        }
    }

    pub fn get_pin_ids(&self) -> Result<&Vec<usize>> {
        if let Some(p) = self.pin_ids.as_ref() {
            Ok(p)
        } else {
            bail!(&err_msg("capture", "pin_ids"))
        }
    }

    pub fn enabled_capture_pins(&self) -> Result<Vec<usize>> {
        let mut retn: Vec<usize> = vec![];
        if let Some(ppids) = self.pin_ids.as_ref() {
            if let Some(enables) = self.enables.as_ref() {
                let mut e = enables.clone();
                let big_one = BigUint::from(1 as u8);
                for p in ppids.iter().rev() {
                    if &e & &big_one == big_one {
                        retn.push(*p);
                    }
                    e >>= 1;
                }
            } else {
                retn = ppids.clone();
            }
        }
        Ok(retn)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Overlay {
    pub label: Option<String>,
    pub symbol: Option<String>,
    pub cycles: Option<usize>,
    pub enables: Option<BigUint>,
    pub pin_ids: Option<Vec<usize>>,
}

impl Overlay {
    pub fn default() -> Self {
        Self {
            label: None,
            symbol: None,
            cycles: None,
            enables: None,
            pin_ids: None,
        }
    }

    pub fn new(
        label: Option<String>,
        symbol: Option<String>,
        cycles: Option<usize>,
        enables: Option<BigUint>,
        pin_ids: Option<Vec<usize>>,
    ) -> Result<Self> {
        Ok(Self {
            label: label,
            symbol: symbol,
            cycles: cycles,
            enables: enables,
            pin_ids: pin_ids,
        })
    }

    pub fn placeholder(
        label: Option<String>,
        symbol: Option<String>,
        cycles: Option<usize>,
        enables: Option<BigUint>,
    ) -> Self {
        Self {
            label: label,
            symbol: symbol,
            cycles: cycles,
            enables: enables,
            pin_ids: None,
        }
    }

    pub fn to_node(&self) -> Node<PAT> {
        node!(PAT::Overlay, self.clone(), None)
    }

    pub fn enabled_overlay_pins(&self) -> Result<Vec<usize>> {
        let mut retn: Vec<usize> = vec![];
        if let Some(ppids) = self.pin_ids.as_ref() {
            if let Some(enables) = self.enables.as_ref() {
                let mut e = enables.clone();
                let big_one = BigUint::from(1 as u8);
                for p in ppids.iter().rev() {
                    if &e & &big_one == big_one {
                        retn.push(*p);
                    }
                    e >>= 1;
                }
            } else {
                retn = ppids.clone();
            }
        }
        Ok(retn)
    }
}
