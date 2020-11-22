#[derive(Debug, Clone, Serialize)]
pub struct Bin {
    pub number: usize,
    pub description: Option<String>,
    /// Bins are hard bins by default, set this true to make it a soft bin
    pub soft: bool,
    /// Bins are fail bins by default, set this true to make it a pass bin
    pub pass: bool,
}
