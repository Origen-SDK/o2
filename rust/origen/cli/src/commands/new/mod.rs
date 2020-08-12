use clap::ArgMatches;
use phf::phf_map;
use tera::{Context, Tera};

// This includes a map of all template files, it is built by cli/build.rs at compile time.
// All files in each sub-directory of commands/new/templates are accessible via a map named after the
// uppercased sub_directory, e.g.
//      PYTHON_APP = { "pyproject.toml" => "[tool.poetry]...", "config/application.toml" => "..." }
//
// Doing it this way means that we can just drop new files into the templates dirs and they will
// automatically be picked up and included in the new app.
include!(concat!(env!("OUT_DIR"), "/new_app_templates.rs"));

pub fn run(matches: &ArgMatches) {
    let name = matches.value_of("name").unwrap();
    if name.to_lowercase() != name {
        display_red!("ERROR: ");
        displayln!("The application name must be lowercased");
        std::process::exit(1);
    }
    let app_dir = std::env::current_dir().unwrap().join(name);

    if app_dir.exists() {
        if !app_dir.read_dir().unwrap().next().is_none() {
            display_red!("ERROR: ");
            displayln!("A directory with that name already exists and is not empty, please delete it or use a new name and try again");
            std::process::exit(1);
        }
    } else {
        std::fs::create_dir(&app_dir)
            .expect("Could you create the new application directory, do you have permission?");
    }

    let mut tera = Tera::default();
    let mut context = Context::new();
    //// Converting this to a vector here as the template was printing out the package list
    //// in reverse order when given the index map
    //let packages: Vec<&Package> = bom.packages.iter().map(|(_id, pkg)| pkg).collect();
    context.insert("app_name", name);
    context.insert("origen_version", &origen::STATUS.origen_version.to_string());
    let mut user_info = "".to_string();
    if let Some(username) = origen::USER.name() {
        user_info += &username;
        if let Some(email) = origen::USER.email() {
            user_info += &format!(" <{}>", &email);
        }
    }
    context.insert("user_info", &user_info);

    for (file, content) in PYTHON_APP.entries() {
        let contents = tera.render_str(content, &context).unwrap();

        let file = file.replace("app_namespace_dir", name);
        let file = app_dir.join(file);

        if !file.parent().unwrap().exists() {
            std::fs::create_dir_all(&file.parent().unwrap())
                .expect("Couldn't create dir within the new app");
        }

        std::fs::write(&file, &contents).expect("Couldn't create a file within the new app");
    }
}
