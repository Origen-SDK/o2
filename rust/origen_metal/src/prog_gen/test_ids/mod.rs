mod bin_array;
mod item;

use crate::Result;
use std::collections::HashMap;
use item::Item;
#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "python")]
pub fn define(py: Python, m: &PyModule) -> PyResult<()> {
    let subm = PyModule::new(py, "test_ids")?;
    subm.add_class::<TestIDs>()?;
    subm.add_class::<AllocationOptions>()?;
    subm.add_class::<Pool>()?;
    m.add_submodule(subm)?;
    Ok(())
}

#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestIDs {
    pub bins: Item,
    pub softbins: Item,
    pub numbers: Item,
    allocations: HashMap<String, Allocation>,
}

#[cfg(not(feature = "python"))]
impl TestIDs {
    pub fn new() -> Self {
        TestIDs {
            bins: Item::new(),
            softbins: Item::new(),
            numbers: Item::new(),
            allocations: HashMap::new(),
        }
    }

    pub fn from_file(file: &str) -> Result<TestIDs> {
        let json = std::fs::read_to_string(file)?;
        let tids: TestIDs = serde_json::from_str(&json)?;
        Ok(tids)
    }
}

#[cfg_attr(feature = "python", pymethods)]
impl TestIDs {
    #[cfg(feature = "python")]
    #[new]
    pub fn new() -> PyResult<Self> {
        Ok(TestIDs {
            bins: Item::new(),
            softbins: Item::new(),
            numbers: Item::new(),
            allocations: HashMap::new(),
        })
    }

    #[cfg(feature = "python")]
    #[staticmethod]
    pub fn from_file(file: &str) -> Result<TestIDs> {
        let json = std::fs::read_to_string(file)?;
        let tids: TestIDs = serde_json::from_str(&json)?;
        Ok(tids)
    }

    #[cfg(feature = "python")]
    // Sidesteps ownership issues when trying to mutate self.bins.include directly from Python
    pub fn push(&mut self, pool: Pool, value: u32) {
        match pool {
            Pool::BinInclude => self.bins.include.push(value),
            Pool::BinExclude => self.bins.exclude.push(value),
            Pool::SoftBinInclude => self.softbins.include.push(value),
            Pool::SoftBinExclude => self.softbins.exclude.push(value),
            Pool::NumberInclude => self.numbers.include.push(value),
            Pool::NumberExclude => self.numbers.exclude.push(value),
        }
    }
    
    #[cfg(feature = "python")]
    pub fn push_range(&mut self, pool: Pool, start: u32, end: u32) {
        match pool {
            Pool::BinInclude => self.bins.include.push_range(start, end),
            Pool::BinExclude => self.bins.exclude.push_range(start, end),
            Pool::SoftBinInclude => self.softbins.include.push_range(start, end),
            Pool::SoftBinExclude => self.softbins.exclude.push_range(start, end),
            Pool::NumberInclude => self.numbers.include.push_range(start, end),
            Pool::NumberExclude => self.numbers.exclude.push_range(start, end),
        }
    }

    #[cfg(feature = "python")]
    pub fn set_increment(&mut self, pool: Pool, size: u32) {
        match pool {
            Pool::BinInclude | Pool::BinExclude => self.bins.increment = size,
            Pool::SoftBinInclude | Pool::SoftBinExclude => self.softbins.increment = size,
            Pool::NumberInclude | Pool::NumberExclude => self.numbers.increment = size,
        }
    }
    
    pub fn allocate(&mut self, test_name: &str) -> Result<Allocation> {
        let opts = AllocationOptions::default();
        self.allocate_with_options(test_name, opts)
    }

    pub fn allocate_with_options(&mut self, test_name: &str, options: AllocationOptions) -> Result<Allocation> {
        let key = test_name.to_lowercase();
        let key = key.trim();
        if !self.allocations.contains_key(key) || !options.is_default() {
            let a = {
                if let Some(a) = self.allocations.get_mut(key) {
                    a
                } else {
                    self.allocations.insert(key.to_string(), Allocation::default());
                    self.allocations.get_mut(key).unwrap()
                }
            };

            if options.no_bin {
                a.bin = None;
            } else {
                if let Some(b) = options.bin {
                    if a.bin != Some(b) {
                        self.bins.reserved.push(b);
                        a.bin = Some(b);
                    }
                } else {
                    if a.bin.is_none() {
                        a.bin = self.bins.next(options.size);
                    }
                }
            }
            if options.no_softbin {
                a.softbin = None;
            } else {
                if let Some(b) = options.softbin {
                    if a.softbin != Some(b) {
                        self.softbins.reserved.push(b);
                        a.softbin = Some(b);
                    }
                } else {
                    if a.softbin.is_none() {
                        a.softbin = self.softbins.next(options.size);
                    }
                }
            }
            if options.no_number {
                a.number = None;
            } else {
                if let Some(b) = options.number {
                    if a.number != Some(b) {
                        self.numbers.reserved.push(b);
                        a.number = Some(b);
                    }
                } else {
                    if a.number.is_none() {
                        a.number = self.numbers.next(options.size);
                    }
                }
            }
        }
        let allocation = self.allocations[key].clone();
        if let Some(b) = allocation.bin {
            let size = options.size.unwrap_or(self.bins.increment);
            for i in 0..size {
                self.bins.record_reference(b + i);
            }
        }
        if let Some(b) = allocation.softbin {
            let size = options.size.unwrap_or(self.softbins.increment);
            for i in 0..size {
                self.softbins.record_reference(b + i);
            }
        }
        if let Some(n) = allocation.number {
            let size = options.size.unwrap_or(self.numbers.increment);
            for i in 0..size {
                self.numbers.record_reference(n + i);
            }
        }
        Ok(allocation)
    }
    
    pub fn save(&self, file: &str) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(file, json)?;
        Ok(())
    }
}

#[cfg_attr(feature = "python", pyclass)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Pool {
    BinInclude,
    BinExclude,
    SoftBinInclude,
    SoftBinExclude,
    NumberInclude,
    NumberExclude,
}

#[cfg_attr(feature = "python", pyclass(get_all))]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Allocation {
    pub bin: Option<u32>,
    pub softbin: Option<u32>,
    pub number: Option<u32>,
}

#[cfg_attr(feature = "python", pymethods)]
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

#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[derive(Debug, Default, Clone, PartialEq)]
pub struct AllocationOptions {
    pub bin: Option<u32>,
    pub softbin: Option<u32>,
    pub number: Option<u32>,
    pub no_bin: bool,
    pub no_softbin: bool,
    pub no_number: bool,
    pub size: Option<u32>,
}

#[cfg_attr(feature = "python", pymethods)]
impl AllocationOptions {
    #[cfg(feature = "python")]
    #[new]
    pub fn new() -> PyResult<Self> {
        Ok(AllocationOptions::default())
    }
}

impl AllocationOptions {
    fn is_default(&self) -> bool {
        self == &AllocationOptions::default()
    }
}

#[cfg(test)]
mod tests {
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
        assert_eq!(tids.allocate_with_options("t2", opts).unwrap().bin, Some(3));
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
        assert_eq!(tids.allocate_with_options("t2", opts).unwrap().bin, Some(3));
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
        assert_eq!(tids.allocate_with_options("t1", opts).unwrap().bin, None);
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
        
        tids.save("tids.json").unwrap();

        let mut tids = TestIDs::from_file("tids.json").unwrap();
        
        assert_eq!(tids.allocate("t2").unwrap().bin, Some(2));
        assert_eq!(tids.allocate("t4").unwrap().bin, Some(10));
        // remove the file
        std::fs::remove_file("tids.json").unwrap();
    }
    
    #[test]
    fn tests_can_reserve_multiple_bins() {
        let mut tids = TestIDs::new();
        tids.bins.include.push_range(10, 30);
        tids.bins.increment = 5;
        let opts = AllocationOptions {
            size: Some(2),
            ..Default::default()
        };

        assert_eq!(tids.allocate_with_options("t1", opts).unwrap().bin, Some(10));
        assert_eq!(tids.allocate("t2").unwrap().bin, Some(12));  // Incremented by 2
        assert_eq!(tids.allocate("t3").unwrap().bin, Some(17));  // Incremented by 5 by default
        assert_eq!(tids.allocate("t4").unwrap().bin, Some(22));  
        assert_eq!(tids.allocate("t5").unwrap().bin, Some(10));  // Reusing the oldest
        assert_eq!(tids.allocate("t6").unwrap().bin, Some(15));
    }
}