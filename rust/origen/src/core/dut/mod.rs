#[derive(Debug)]
pub struct DUT {
    pub id: String,
    pub memory_maps: HashMap<String, MemoryMap>,
}
