/// Wrappers around the PyPi JSON API instead of pip
/// https://warehouse.pypa.io/api-reference/json.html

use crate::Result;

lazy_static! {
    pub static ref PYPI_URL_BASE: &'static str = "https://pypi.org";
    pub static ref PYPI_TEST_URL_BASE: &'static str = "https://test.pypi.org";
}

pub fn get_package_details<S: AsRef<str>>(package_name: S) -> Result<serde_json::Value> {
    get_package_details_from(package_name, *PYPI_URL_BASE)
}

pub fn get_package_details_from_test_server<S: AsRef<str>>(package_name: S) -> Result<serde_json::Value> {
    get_package_details_from(package_name, *PYPI_TEST_URL_BASE)
}

pub fn get_package_details_from<S1: AsRef<str>, S2: AsRef<str>>(package_name: S1, url: S2) -> Result<serde_json::Value>
{
    let url = format!("{}/pypi/{}/json", url.as_ref(), package_name.as_ref());
    let response = reqwest::blocking::get(&url)?;
    Ok(response.json::<serde_json::Value>()?)
}

pub fn get_package_versions<S: AsRef<str>>(package_name: S) -> Result<Vec<String>> {
    get_package_versions_from(package_name, *PYPI_URL_BASE)
}

pub fn get_package_versions_from_test_server<S: AsRef<str>>(package_name: S) -> Result<Vec<String>> {
    get_package_versions_from(package_name, *PYPI_TEST_URL_BASE)
}

pub fn get_package_versions_from<S1: AsRef<str>, S2: AsRef<str>>(package_name: S1, url: S2) -> Result<Vec<String>> {
    let res = get_package_details_from(package_name.as_ref(), url)?;
    let versions = res["releases"].as_object().unwrap().keys().map( |k| k.to_string()).collect::<Vec<String>>();
    Ok(versions)
}

pub fn is_package_version_available<S1: AsRef<str>, S2: AsRef<str>>(package_name: S1, version: S2) -> Result<bool> {
    let pkgs = get_package_versions(package_name)?;
    Ok(pkgs.iter().find(|v| v == &version.as_ref()).is_none())
}

pub fn is_package_version_available_on_test_server<S1: AsRef<str>, S2: AsRef<str>>(package_name: S1, version: S2) -> Result<bool> {
    let pkgs = get_package_versions_from_test_server(package_name)?;
    Ok(pkgs.iter().find(|v| v == &version.as_ref()).is_none())
}
