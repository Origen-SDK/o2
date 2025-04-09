use crate::Result;
use super::bin_array::BinArray;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    /// Numbers to be included
    pub include: BinArray,
    /// Numbers to be excluded
    pub exclude: BinArray,
    /// Numbers that have been manually assigned
    pub reserved: BinArray,
    pub algorithm: Option<String>,
    pub size: u32,
    pub needs: Vec<String>,
}

impl Item {
    pub fn new() -> Self {
        Self {
            include: BinArray::new(),
            exclude: BinArray::new(),
            reserved: BinArray::new(),
            algorithm: None,
            size: 1,
            needs: Vec::new(),
        }
    }
    
    pub fn needs(&self, type_: &str) -> bool {
        !self.is_empty() && self.is_function() && 
        (self.needs.contains(&type_.to_string()) ||
         self.algorithm.as_ref().map_or(false, |alg| {
             alg.to_lowercase().contains(&type_[0..1].to_lowercase())
         }))
    }

    pub fn is_empty(&self) -> bool {
        self.include.is_empty() && self.exclude.is_empty() && 
        self.algorithm.is_none()
    }

    pub fn is_function(&self) -> bool {
        self.algorithm.is_some()
    }

    pub fn is_valid(&self, number: u32) -> Result<bool> {
        if self.is_function() {
            bail!("valid? is not supported for algorithm-based assignments")
        } else {
            Ok(self.include.contains(number) && !self.exclude.contains(number))
        }
    }
    
    pub fn next(&mut self) -> Option<u32> {
        if self.is_function() {
            None
        } else {
            loop {
                if let Some(n) = self.include.next(None, None) {
                    if !self.exclude.contains(n) && !self.reserved.contains(n) {
                        return Some(n);
                    }
                } else {
                    return None;
                }
            }
        }
    }
    
    // Returns an iterator over all included numbers
    pub fn iter(&self) -> impl Iterator<Item = u32> + '_ {
        self.include.iter().filter(move |&num| !self.exclude.contains(num) && !self.reserved.contains(num))
    }

}
