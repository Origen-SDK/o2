#[derive(Debug, Clone, Default)]
pub struct Bit {
    pub register_id: usize,
    pub overlay: Option<String>,
    /// The individual bits mean the following:
    /// 0 - Data value
    /// 1 - Value is X when 1
    /// 2 - Value is Z when 1
    /// 3 - Bit is to be read
    /// 4 - Bit is to be captured
    /// 5 - Bit has an overlay (defined by overlay str)
    pub state: u8,
    pub unimplemented: bool,
}

impl Bit {
    /// Returns true if not in X or Z state
    pub fn has_known_value(&self) -> bool {
        self.state & 0b110 == 0
    }
}
