#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Group {
    pub id: String,
    pub packages: Vec<String>,
    pub version: Option<String>,
}

impl Group {
    pub fn merge(&mut self, g: &Group) {
        match &g.version {
            Some(x) => {
                self.version = Some(x.clone());
            }
            None => {}
        }
    }
}
