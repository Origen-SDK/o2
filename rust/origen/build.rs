extern crate built;
use std::fs;
use std::path::PathBuf;

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
}
