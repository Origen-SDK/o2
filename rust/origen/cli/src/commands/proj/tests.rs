use crate::commands::proj::BOM;
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

#[test]
fn package_version_resolution() {
    let temp = tempdir().unwrap();
    let project_dir = temp.path();
    let project_bom = project_dir.join("bom.toml");
    let workspace = project_dir.join("workspace");
    let workspace_bom = workspace.join("bom.toml");

    let _ = std::fs::create_dir(&workspace);

    let mut f = File::create(&project_bom).unwrap();
    writeln!(
        f,
        r#"
[[group]]
id = "grp1"
packages = ["pkg1", "pkg2"]
version = "develop"

[[package]]
id = "pkg1"
version = "master"
repo = "git@github.com:Origen-SDK/test_repo.git"

[[package]]
id = "pkg2"
repo = "git@github.com:Origen-SDK/test_repo.git"
    "#
    )
    .expect("Couldn't write project BOM");

    let mut f = File::create(&workspace_bom).unwrap();
    writeln!(
        f,
        r#"
[[group]]
id = "grp1"
version = "t1"

[[package]]
id = "pkg1"
#version = "master"

[[package]]
id = "pkg2"
version = "t2"
    "#
    )
    .expect("Couldn't write project BOM");

    // At project level a package version takes priority
    let bom = BOM::for_dir(&project_dir);
    assert_eq!(bom.packages["pkg1"].version.as_ref().unwrap(), "master");
    assert_eq!(bom.packages["pkg2"].version.as_ref().unwrap(), "develop");

    // At workspace level a group version overrides the project version, regardless
    // of whether it was a group or package version upstream
    let bom = BOM::for_dir(&workspace);
    assert_eq!(bom.packages["pkg1"].version.as_ref().unwrap(), "t1");

    // At workspace level a package version takes priority
    assert_eq!(bom.packages["pkg2"].version.as_ref().unwrap(), "t2");
}
