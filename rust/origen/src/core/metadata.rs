#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum Metadata {
    String(String),
    Usize(usize),
    // ... as needed
}