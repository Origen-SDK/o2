pub trait NumHelpers {
    fn even_parity(&self) -> bool;
}

impl NumHelpers for u32 {
    fn even_parity(&self) -> bool {
        if self.count_ones() % 2 == 0 {
            false
        } else {
            true
        }
    }
}