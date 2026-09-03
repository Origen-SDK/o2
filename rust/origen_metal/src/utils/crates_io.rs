//! Minimal wrappers around the crates.io HTTP API.

use crate::Result;

const CRATES_IO_API: &str = "https://crates.io/api/v1/crates";

pub fn get_crate_versions(name: &str) -> Result<Vec<String>> {
    let response = reqwest::blocking::Client::new()
        .get(format!("{}/{}", CRATES_IO_API, name))
        .header("User-Agent", "origen-release")
        .send()?
        .error_for_status()?
        .json::<serde_json::Value>()?;
    parse_versions(&response)
}

pub fn is_crate_version_available(name: &str, version: &str) -> Result<bool> {
    Ok(!get_crate_versions(name)?.iter().any(|v| v == version))
}

fn parse_versions(response: &serde_json::Value) -> Result<Vec<String>> {
    let versions = response["versions"]
        .as_array()
        .ok_or_else(|| crate::Error::new("crates.io response did not contain a versions array"))?;
    versions
        .iter()
        .map(|v| {
            v["num"]
                .as_str()
                .map(|s| s.to_string())
                .ok_or_else(|| crate::Error::new("crates.io version did not contain num"))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_versions() -> Result<()> {
        let value = serde_json::json!({"versions": [{"num": "1.5.0"}, {"num": "1.4.0"}]});
        assert_eq!(parse_versions(&value)?, vec!["1.5.0", "1.4.0"]);
        Ok(())
    }
}
