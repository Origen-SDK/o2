#[derive(Debug, Clone, Serialize)]
pub struct Bin {
    pub number: usize,
    pub description: Option<String>,
    pub priority: Option<usize>,
    /// Bins are fail bins by default, set this true to make it a pass bin
    pub pass: bool,
}
