use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

fn main() {
    // This creates a file which defines a map of all files in the 'new' command's template dirs.
    // This file is then included by the 'new' command to allow it to iterate over each template
    // file.
    let cli_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let new_app_templates_dir = cli_dir
        .join("src")
        .join("commands")
        .join("new")
        .join("templates");

    let mut data = "".to_string();

    for entry in fs::read_dir(&new_app_templates_dir)
        .unwrap()
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let template_dir = entry.path();
        let package_name = template_dir
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_uppercase();

        data += &format!(
            "pub static {}: phf::Map<&'static str, &'static str> = phf_map! {{\n",
            package_name
        );

        for entry in WalkDir::new(&template_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() {
                let file = path
                    .strip_prefix(&template_dir)
                    .unwrap()
                    .display()
                    .to_string()
                    .replace(".tera", "");
                let contents = std::fs::read_to_string(path).unwrap();
                data += &format!("\"{}\" => r#\"{}\"#,\n", &file, &contents);
            }
        }
        data += "};\n\n";
    }

    fs::write(&out_dir.join("new_app_templates.rs"), data)
        .expect("Unable to write to new app template manifest file");
}
