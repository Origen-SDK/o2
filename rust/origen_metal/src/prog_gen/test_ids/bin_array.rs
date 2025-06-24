use std::cmp::Ordering;
#[cfg(feature = "python")]
use pyo3::prelude::*;

#[derive(Debug, Eq, PartialEq, Hash, Clone, Serialize, Deserialize)]
pub enum Bin {
    Single(u32),
    /// (start, end)
    Range(u32, u32),
}

impl Bin {
    fn is_range(&self) -> bool {
        match self {
            Bin::Single(_) => false,
            Bin::Range(_, _) => true,
        }
    }

    fn min_val(&self) -> u32 {
        match self {
            Bin::Single(x) => *x,
            Bin::Range(start, _) => *start,
        }
    }

    fn max_val(&self) -> u32 {
        match self {
            Bin::Single(x) => *x,
            Bin::Range(_, end) => *end,
        }
    }

    fn contains(&self, value: &u32) -> bool {
        match self {
            Bin::Single(x) => value == x,
            Bin::Range(start, end) => value >= start && value <= end,
        }
    }
}

impl PartialOrd for Bin {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Bin {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_min = match self {
            Bin::Single(x) => *x,
            Bin::Range(start, _) => *start,
        };
        let other_min = match other {
            Bin::Single(x) => *x,
            Bin::Range(start, _) => *start,
        };
        self_min.cmp(&other_min)
    }
}

#[cfg_attr(feature = "python", pyclass)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinArray {
    store: Vec<Bin>,
    pointer: Option<usize>, // Still usize for Vec indexing
    next: Option<u32>,     // Changed to u32 for bin values
}

impl BinArray {
    pub fn new() -> Self {
        Self {
            store: vec![],
            pointer: None,
            next: None,
        }
    }

    pub fn iter(&self) -> BinArrayIter {
        let mut ba = self.clone();
        ba.pointer = None;
        ba.next = None;
        BinArrayIter {
            bin_array: ba,
            _after: None,
            _size: None,
        }
    }

    // Could not get pyo3 to update the bin array instance via tids.bins.include.push(3), it seemed to be
    // always working on a copy, so moving these to Rust only methods and will implement setters higher
    // up for Python use
    pub fn push(&mut self, value: u32) {
        if !self.contains(value) {
            self.store.push(Bin::Single(value));
            self.store.sort();
        }
    }
    
    pub fn push_range(&mut self, start: u32, end: u32) {
        self.store.push(Bin::Range(start, end));
    }

    // Same as above
    pub fn next(&mut self, after: Option<u32>, size: Option<u32>) -> Option<u32> {
        if let Some(after) = after {
            self.pointer = None;
            let mut i = 0;
            while self.pointer.is_none() {
                if let Some(v) = self.store.get(i) {
                    if v.contains(&after) {
                        self.pointer = Some(i);
                        self.next = Some(after);
                    } else if after < v.min_val() {
                        if i == 0 {
                            self.pointer = Some(self.store.len() - 1);
                        } else {
                            self.pointer = Some(i - 1);
                        }
                        self.next = Some(v.min_val() - 1);
                    }
                } else {
                    self.pointer = Some(self.store.len() - 1);
                    self.next = Some(self.store[0].min_val() - 1);
                }
                i += 1;
            }
        }
        if let Some(next) = self.next {
            if self.pointer.is_none() {
                self.pointer = Some(0);
            }
            let mut pointer = self.pointer.unwrap();
            if self.store[pointer].is_range() && next != self.store[pointer].max_val() {
                self.next = Some(next + 1);
            } else {
                pointer += 1;
                self.pointer = Some(pointer);
                if pointer == self.store.len() {
                    self.pointer = Some(pointer - 1);
                    return None;
                }
                self.next = Some(self.store[pointer].min_val());
            }
        } else {
            let v = self.store.first()?;
            self.next = Some(v.min_val());
        }
        if let Some(size) = size {
            let mut included = true;
            let next = self.next.unwrap();
            for i in 0..size {
                if !self.contains(next + i) {
                    included = false;
                }
            }
            if included {
                let n = Some(next);
                self.next = Some(next + size - 1);
                n
            } else {
                self.next(self.next, Some(size))
            }
        } else {
            self.next
        }
    }
}

#[cfg_attr(feature = "python", pymethods)]
impl BinArray {
    
    pub fn debug(&self) {
        println!("{:?}", self);
    }

    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }

    pub fn contains(&self, bin: u32) -> bool {
        self.store.iter().any(|b| match b {
            Bin::Single(x) => bin == *x,
            Bin::Range(start, end) => bin >= *start && bin <= *end,
        })
    }

    pub fn min(&self) -> Option<u32> {
        self.store.iter().next().map(|b| match b {
            Bin::Single(x) => *x,
            Bin::Range(start, _) => *start,
        })
    }

    pub fn max(&self) -> Option<u32> {
        self.store.iter().next_back().map(|b| match b {
            Bin::Single(x) => *x,
            Bin::Range(_, end) => *end,
        })
    }
}

pub struct BinArrayIter {
    bin_array: BinArray,
    _after: Option<u32>,
    _size: Option<u32>,
}

impl Iterator for BinArrayIter {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        self.bin_array.next(None, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_and_sort() {
        let mut ba = BinArray::new();
        ba.push(3);
        ba.push_range(1, 4);
        ba.push_range(2, 6);
        ba.push(0);

        let expected: Vec<Bin> = [
            Bin::Single(0),
            Bin::Range(1, 4),
            Bin::Range(2, 6),
            Bin::Single(3),
        ]
        .into_iter()
        .collect();

        assert_eq!(&ba.store, &expected, "Store should be sorted");
    }

    #[test]
    fn test_contains() {
        let mut ba = BinArray::new();
        ba.push(3);
        ba.push_range(1, 4);
        ba.push_range(6, 8);

        assert!(ba.contains(3), "Should find Single(3)");
        assert!(ba.contains(2), "Should find 2 in Range(1, 4)");
        assert!(ba.contains(4), "Should find 4 in Range(1, 4)");
        assert!(ba.contains(7), "Should find 7 in Range(6, 8)");
        assert!(!ba.contains(5), "Should not find 5 (no match)");
        assert!(!ba.contains(9), "Should not find 9 (out of range)");

        let mut b = BinArray::new();
        b.push(10);
        b.push_range(15, 20);
        b.push(30);

        assert!(!b.contains(5), "Should not include 5");
        assert!(b.contains(10), "Should include 10");
        assert!(!b.contains(14), "Should not include 14");
        assert!(b.contains(15), "Should include 15");
        assert!(b.contains(19), "Should include 19");
        assert!(b.contains(20), "Should include 20");
        assert!(!b.contains(21), "Should not include 21");
        assert!(b.contains(30), "Should include 30");
        assert!(!b.contains(31), "Should not include 31");
    }

    // #[test]
    // fn test_min_and_max() {
    //     let mut b = BinArray::new();
    //     b.push(100);
    //     assert_eq!(b.min(), Some(100), "Min should be 100");
    //     assert_eq!(b.max(), Some(100), "Max should be 100");

    //     b.push_range(200, 300);
    //     assert_eq!(b.min(), Some(100), "Min should still be 100");
    //     assert_eq!(b.max(), Some(300), "Max should be 300");

    //     b.push_range(20, 50);
    //     assert_eq!(b.min(), Some(20), "Min should be 20");
    //     assert_eq!(b.max(), Some(300), "Max should still be 300");
    // }

    #[test]
    fn test_next() {
        let mut b = BinArray::new();
        b.push(10);
        b.push_range(15, 20);
        b.push(30);

        assert_eq!(b.next(None, None), Some(10), "Next should be 10");
        assert_eq!(b.next(None, None), Some(15), "Next should be 15");
        assert_eq!(b.next(None, None), Some(16), "Next should be 16");
        assert_eq!(
            b.next(Some(18), None),
            Some(19),
            "Next after 18 should be 19"
        );
        assert_eq!(b.next(None, None), Some(20), "Next should be 20");
        assert_eq!(b.next(None, None), Some(30), "Next should be 30");
        assert_eq!(b.next(None, None), None, "Next should be nil at end");
        assert_eq!(
            b.next(Some(13), None),
            Some(15),
            "Next after 13 should be 15"
        );
        assert_eq!(
            b.next(Some(25), None),
            Some(30),
            "Next after 25 should be 30"
        );
        assert_eq!(
            b.next(Some(100), None),
            None,
            "Next after 100 should be nil"
        );
    }

    #[test]
    fn test_next_with_size() {
        let mut b = BinArray::new();
        b.push_range(10, 20);
        b.push_range(30, 40);

        assert_eq!(b.next(None, Some(4)), Some(10), "Next size 4 should be 10");
        assert_eq!(b.next(None, Some(4)), Some(14), "Next size 4 should be 14");
        assert_eq!(b.next(None, Some(4)), Some(30), "Next size 4 should be 30");
        assert_eq!(b.next(None, Some(4)), Some(34), "Next size 4 should be 34");
        assert_eq!(b.next(None, Some(4)), None, "Next size 4 should be nil");
    }

    #[test]
    fn test_iteration() {
        let mut b = BinArray::new();
        b.push(10);
        b.push_range(15, 20);
        b.push(30);

        let mut it = b.iter();

        assert_eq!(it.next(), Some(10), "Next should be 10");
        assert_eq!(it.next(), Some(15), "Next should be 15");

        let collected: Vec<u32> = b.iter().collect();

        let expected = vec![10, 15, 16, 17, 18, 19, 20, 30];
        assert_eq!(collected, expected, "Yield_all should produce all numbers");

        assert_eq!(it.next(), Some(16), "Next should resume at 16");
    }

    // #[test]
    // fn single_allocations_should_wrap() {
    //     let mut b = BinArray::new();
    //     b.push(3);

    //     assert_eq!(b.next(None, None), Some(3), "Sshould be 3");
    //     assert_eq!(b.next(None, None), Some(3), "Sshould be 3");
    //     assert_eq!(b.next(None, None), Some(3), "Sshould be 3");
    // }
}