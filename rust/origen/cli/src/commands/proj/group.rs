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

    pub fn to_string(&self, indent: usize) -> String {
        let i = " ".repeat(indent);
        let mut s = format!("{}[[group]]\n", i);
        s += &format!("{}id = \"{}\"\n", i, self.id);
        s += &format!(
            "{}packages = [{}]\n",
            i,
            self.packages
                .iter()
                .map(|pid| format!("\"{}\"", pid))
                .collect::<Vec<String>>()
                .join(", ")
        );
        s += "\n";
        s
    }
}
