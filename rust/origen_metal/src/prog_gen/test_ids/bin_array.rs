use std::cmp::Ordering;

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum Bin {
    Single(usize),
    /// (start, end)
    Range(usize, usize),
}

impl Bin {
    fn is_range(&self) -> bool {
        match self {
            Bin::Single(_) => false,
            Bin::Range(_, _) => true,
        }
    }

    fn min_val(&self) -> usize {
        match self {
            Bin::Single(x) => *x,
            Bin::Range(start, _) => *start,
        }
    }

    fn max_val(&self) -> usize {
        match self {
            Bin::Single(x) => *x,
            Bin::Range(_, end) => *end,
        }
    }

    fn contains(&self, value: &usize) -> bool {
        match self {
            Bin::Single(x) => return value == x,
            Bin::Range(start, end) => return value >= start && value <= end,
        }
    }
}

// Custom Ord for sorting by min value
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

#[derive(Debug, Clone)]
pub struct BinArray {
    store: Vec<Bin>,
    pointer: Option<usize>,
    next: Option<usize>,
}

impl BinArray {
    pub fn new() -> Self {
        Self {
            store: vec![],
            pointer: None,
            next: None,
        }
    }

    pub fn push(&mut self, value: Bin) {
        self.store.push(value);
        self.store.sort();
    }

    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }

    pub fn contains(&self, bin: usize) -> bool {
        self.store.iter().any(|b| match b {
            Bin::Single(x) => return bin == *x,
            Bin::Range(start, end) => return bin >= *start && bin <= *end,
        })
    }

    pub fn min(&self) -> Option<usize> {
        self.store.iter().next().map(|b| {
            if let Bin::Single(x) = b {
                return *x;
            }
            if let Bin::Range(start, _) = b {
                return *start;
            }
            unreachable!()
        })
    }

    pub fn max(&self) -> Option<usize> {
        self.store.iter().next_back().map(|b| {
            if let Bin::Single(x) = b {
                return *x;
            }
            if let Bin::Range(_, end) = b {
                return *end;
            }
            unreachable!()
        })
    }

    /// Returns the next bin in the array, starting from the first and remembering the last bin
    /// when called the next time.
    /// A bin can optionally be supplied in which case the internal pointer will be reset and the
    /// next bin that occurs after the given number will be returned
    pub fn next(&mut self, after: Option<usize>, size: Option<usize>) -> Option<usize> {
        if let Some(after) = after {
            // Need to work out the pointer here as it is probably out of sync with the
            // last value now
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
                    // Gone past the end of the array
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
                // Return none when we get to the end of the array
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
            // Check that all the numbers in the range to be reserved are included in the allocation,
            // if not call again
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
}

// Iterator struct
pub struct BinArrayIter {
    bin_array: BinArray,
    // Placeholders in case we ever need to pass these to the iterator
    _after: Option<usize>,
    _size: Option<usize>,
}

impl<'a> Iterator for BinArrayIter {
    type Item = usize;

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
        ba.push(Bin::Single(3));
        ba.push(Bin::Range(1, 4));
        ba.push(Bin::Range(2, 6));
        ba.push(Bin::Single(0));

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
        ba.push(Bin::Single(3));
        ba.push(Bin::Range(1, 4));
        ba.push(Bin::Range(6, 8));

        assert!(ba.contains(3), "Should find Single(3)");
        assert!(ba.contains(2), "Should find 2 in Range(1, 4)");
        assert!(ba.contains(4), "Should find 4 in Range(1, 4)");
        assert!(ba.contains(7), "Should find 7 in Range(6, 8)");
        assert!(!ba.contains(5), "Should not find 5 (no match)");
        assert!(!ba.contains(9), "Should not find 9 (out of range)");

        let mut b = BinArray::new();
        b.push(Bin::Single(10));
        b.push(Bin::Range(15, 20));
        b.push(Bin::Single(30));

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

    #[test]
    fn test_min_and_max() {
        let mut b = BinArray::new();
        b.push(Bin::Single(100));
        assert_eq!(b.min(), Some(100), "Min should be 100");
        assert_eq!(b.max(), Some(100), "Max should be 100");

        b.push(Bin::Range(200, 300));
        assert_eq!(b.min(), Some(100), "Min should still be 100");
        assert_eq!(b.max(), Some(300), "Max should be 300");

        b.push(Bin::Range(20, 50));
        assert_eq!(b.min(), Some(20), "Min should be 20");
        assert_eq!(b.max(), Some(300), "Max should still be 300");
    }

    #[test]
    fn test_next() {
        let mut b = BinArray::new();
        b.push(Bin::Single(10));
        b.push(Bin::Range(15, 20));
        b.push(Bin::Single(30));

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
        b.push(Bin::Range(10, 20));
        b.push(Bin::Range(30, 40));

        assert_eq!(b.next(None, Some(4)), Some(10), "Next size 4 should be 10");
        assert_eq!(b.next(None, Some(4)), Some(14), "Next size 4 should be 14");
        assert_eq!(b.next(None, Some(4)), Some(30), "Next size 4 should be 30");
        assert_eq!(b.next(None, Some(4)), Some(34), "Next size 4 should be 34");
        assert_eq!(b.next(None, Some(4)), None, "Next size 4 should be nil");
    }

    #[test]
    fn test_iteration() {
        let mut b = BinArray::new();
        b.push(Bin::Single(10));
        b.push(Bin::Range(15, 20));
        b.push(Bin::Single(30));

        let mut it = b.iter();

        assert_eq!(it.next(), Some(10), "Next should be 10");
        assert_eq!(it.next(), Some(15), "Next should be 15");

        let collected: Vec<usize> = b.iter().collect();

        let expected = vec![10, 15, 16, 17, 18, 19, 20, 30];
        assert_eq!(collected, expected, "Yield_all should produce all numbers");

        assert_eq!(it.next(), Some(16), "Next should resume at 16");
    }
}
