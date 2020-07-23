#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Group {
    pub id: String,
    pub packages: Vec<String>,
    pub version: Option<String>,
}

/// This is used to make the packages optional when parsing a BOM (e.g. it doesn't
/// need to be defined in the workspace BOM), but it is quickly converted to a real group
/// where the pacakges field is required
#[derive(Debug, Deserialize)]
pub struct TempGroup {
    pub id: String,
    pub packages: Option<Vec<String>>,
    pub version: Option<String>,
}

impl TempGroup {
    pub fn to_group(&self) -> Group {
        Group {
            id: self.id.clone(),
            packages: match &self.packages {
                None => vec![],
                Some(packages) => packages.clone(),
            },
            version: self.version.clone(),
        }
    }
}

impl Group {
    pub fn merge(&mut self, g: &Group) {
        // Take the newer groups version if it has one, otherwise keep the existing
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

    pub fn validate(&self) {}
}
