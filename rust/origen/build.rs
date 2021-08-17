extern crate built;
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

fn main() {
    let rust_origen_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    let cargo_verifying_pkg = rust_origen_dir
        .clone()
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .starts_with("origen-");
    if cargo_verifying_pkg {
        // Not really a warning, but this is the only way I can see to get output out
        // of the build script without also getting crazy amounts of other output
        println!("cargo:warning=Copying Cargo.lock file...");
        println!(
            "cargo:warning={:?}",
            rust_origen_dir
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .join("cargo.toml")
        );
        fs::copy(
            rust_origen_dir
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .join("Cargo.lock"),
            rust_origen_dir.clone().join("Cargo.lock"),
        )
        .expect("Unable to copy Cargo.lock from root directory");
    }

    built::write_built_file_with_opts(
        &{
            let mut opts = built::Options::default();
            opts.set_dependencies(true);
            opts
        },
        &rust_origen_dir,
        &{
            let mut built_out = out_dir.clone();
            built_out.push("built.rs");
            built_out
        },
    )
    .expect("Failed to acquire build-time information");

    if cargo_verifying_pkg {
        fs::remove_file(rust_origen_dir.clone().join("Cargo.lock"))
            .expect("Unable to remove Cargo.lock from verification directory");
    }

    // This creates a file which defines a map of all files in the test program test_templates dir.
    // This file is then included by the test program module to give it access to the template files.
    let test_templates_dir = rust_origen_dir
        .join("src")
        .join("prog_gen")
        .join("test_templates");

    let mut data = "".to_string();

    data += "pub static TEST_TEMPLATES: phf::Map<&'static str, &'static str> = phf_map! {\n";

    for entry in WalkDir::new(&test_templates_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() {
            let file = path
                .strip_prefix(&test_templates_dir)
                .unwrap()
                .display()
                .to_string()
                .replace(".tera", "")
                .replace("\\", "/");
            let contents = std::fs::read_to_string(path).unwrap();
            data += &format!("r#\"{}\"# => r#\"{}\"#,\n", &file, &contents);
        }
    }

    data += "};\n\n";

    fs::write(&out_dir.join("test_templates.rs"), data).expect("Unable to write to test templates");
}
