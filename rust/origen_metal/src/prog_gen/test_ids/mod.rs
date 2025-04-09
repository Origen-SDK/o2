mod bin_array;
mod item;

use crate::Result;
use std::{collections::HashMap, path::Path};
use item::Item;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestIDs {
    pub bins: Item,
    pub softbins: Item,
    pub numbers: Item,
    allocations: HashMap<String, Allocation>,
}

impl TestIDs {
    pub fn new () -> Self {
        Self {
            bins: Item::new(),
            softbins: Item::new(),
            numbers: Item::new(),
            allocations: HashMap::new(),
        }
    }

    pub fn from_file(file: &Path) -> Result<Self> {
        let json = std::fs::read_to_string(file)?;
        let tids: TestIDs = serde_json::from_str(&json)?;
        Ok(tids)
    }
    
    pub fn allocate(&mut self, test_name: &str) -> Result<&Allocation> {
        let opts = AllocationOptions::default();
        self.allocate_with_options(test_name, &opts)
    }

    pub fn allocate_with_options(&mut self, test_name: &str, options: &AllocationOptions) -> Result<&Allocation> {
        let key = test_name.to_lowercase();
        let key = key.trim();
        if !self.allocations.contains_key(key) {
            let bin = if options.no_bin {
                None
            } else {
                if let Some(b) = options.bin {
                    self.bins.reserved.push(b);
                    Some(b)
                } else {
                    self.bins.next()
                }
            }; 
            let softbin = if options.no_softbin {
                None
            } else {
                if let Some(b) = options.softbin {
                    self.softbins.reserved.push(b);
                    Some(b)
                } else {
                    self.softbins.next()
                }
            };
            let number = if options.no_number {
                None
            } else {
                if let Some(n) = options.number {
                    self.numbers.reserved.push(n);
                    Some(n)
                } else {
                    self.numbers.next()
                }
            };
            self.allocations.insert(test_name.to_string(), Allocation {
                bin: bin,
                softbin: softbin,
                number: number,
            });
        }
        Ok(&self.allocations[key])
    }
    
    pub fn save(&self, file: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(file, json)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Allocation {
    pub bin: Option<u32>,
    pub softbin: Option<u32>,
    pub number: Option<u32>,
}

impl Allocation {
    pub fn to_hashmap(&self) -> HashMap<String, u32> {
        let mut h = HashMap::new();
        if let Some(b) = self.bin {
            h.insert("bin".to_string(), b);
        }
        if let Some(b) = self.softbin {
            h.insert("softbin".to_string(), b);
        }
        if let Some(b) = self.number {
            h.insert("number".to_string(), b);
        }
        h
    }
}

#[derive(Debug, Default)]
pub struct AllocationOptions {
    pub bin: Option<u32>,
    pub softbin: Option<u32>,
    pub number: Option<u32>,
    pub no_bin: bool,
    pub no_softbin: bool,
    pub no_number: bool,
}


#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    
    #[test]
    fn is_alive() {
        let mut tids = TestIDs::new();
        tids.bins.include.push(3);
        tids.bins.include.push(4);
        assert_eq!(tids.allocate("t1").unwrap().bin, Some(3));
    }

    #[test]
    fn bin_numbers_increment() {
        let mut tids = TestIDs::new();
        tids.bins.include.push_range(1, 3);
        assert_eq!(tids.allocate("t1").unwrap().bin, Some(1));
        assert_eq!(tids.allocate("t2").unwrap().bin, Some(2));
        assert_eq!(tids.allocate("t3").unwrap().bin, Some(3));
    }

    #[test]
    fn duplicate_tests_pick_up_the_same_number() {
        let mut tids = TestIDs::new();
        tids.bins.include.push_range(1, 3);
        assert_eq!(tids.allocate("t1").unwrap().bin, Some(1));
        assert_eq!(tids.allocate("t2").unwrap().bin, Some(2));
        assert_eq!(tids.allocate("t1").unwrap().bin, Some(1));
        assert_eq!(tids.allocate("t3").unwrap().bin, Some(3));
    }

    #[test]
    fn caller_can_override_bin_number() {
        let mut tids = TestIDs::new();
        tids.bins.include.push_range(1, 4);
        let opts = AllocationOptions {
            bin: Some(3),
            ..Default::default()
        };
        assert_eq!(tids.allocate("t1").unwrap().bin, Some(1));
        assert_eq!(tids.allocate_with_options("t2", &opts).unwrap().bin, Some(3));
        assert_eq!(tids.allocate("t2").unwrap().bin, Some(3));
        assert_eq!(tids.allocate("t3").unwrap().bin, Some(2));
    }

    #[test]
    fn manually_assigned_bins_are_reserved() {
        let mut tids = TestIDs::new();
        tids.bins.include.push_range(1, 4);
        let opts = AllocationOptions {
            bin: Some(3),
            ..Default::default()
        };
        assert_eq!(tids.allocate("t1").unwrap().bin, Some(1));
        assert_eq!(tids.allocate_with_options("t2", &opts).unwrap().bin, Some(3));
        assert_eq!(tids.allocate("t3").unwrap().bin, Some(2));
        assert_eq!(tids.allocate("t4").unwrap().bin, Some(4));
    }
    
    #[test]
    fn assignments_can_be_inhibited() {
        let mut tids = TestIDs::new();
        tids.bins.include.push_range(1, 4);
        let opts = AllocationOptions {
            no_bin: true,
            ..Default::default()
        };
        assert_eq!(tids.allocate_with_options("t1", &opts).unwrap().bin, None);
    }

    #[test]
    fn excluded_bins_are_not_used() {
        let mut tids = TestIDs::new();
        tids.bins.include.push_range(1, 10);
        tids.bins.exclude.push(3);
        tids.bins.exclude.push_range(5, 9);
        assert_eq!(tids.allocate("t1").unwrap().bin, Some(1));
        assert_eq!(tids.allocate("t2").unwrap().bin, Some(2));
        assert_eq!(tids.allocate("t3").unwrap().bin, Some(4));
        assert_eq!(tids.allocate("t4").unwrap().bin, Some(10));
    }

    #[test]
    fn the_system_can_be_saved_to_a_file_and_resumed() {
        let mut tids = TestIDs::new();
        tids.bins.include.push_range(1, 10);
        tids.bins.exclude.push(3);
        tids.bins.exclude.push_range(5, 9);
        assert_eq!(tids.allocate("t1").unwrap().bin, Some(1));
        assert_eq!(tids.allocate("t2").unwrap().bin, Some(2));
        assert_eq!(tids.allocate("t3").unwrap().bin, Some(4));
        
        tids.save(Path::new("tids.json")).unwrap();

        let mut tids = TestIDs::from_file(Path::new("tids.json")).unwrap();
        
        assert_eq!(tids.allocate("t2").unwrap().bin, Some(2));
        assert_eq!(tids.allocate("t4").unwrap().bin, Some(10));
        // remove the file
        std::fs::remove_file("tids.json").unwrap();
    }
}