// This implements the new application command, for the code generators, e.g. 'origen new dut' etc.,
// see new_resource.rs

mod new_resource;

use clap::ArgMatches;
use origen::STATUS;
use phf::map::Map;
use phf::phf_map;
use std::path::PathBuf;
use tera::{Context, Tera};

// This includes a map of all template files, it is built by cli/build.rs at compile time.
// All files in each sub-directory of commands/new/templates are accessible via a map named after the
// uppercased sub_directory, e.g.
//      PYTHON_APP = { "pyproject.toml" => "[tool.poetry]...", "config/application.toml" => "..." }
//
// Doing it this way means that we can just drop new files into the templates dirs and they will
// automatically be picked up and included in the new app.
include!(concat!(env!("OUT_DIR"), "/new_app_templates.rs"));

struct App {
    name: String,
    dir: PathBuf,
}

pub fn run(matches: &ArgMatches) {
    if STATUS.is_app_present {
        new_resource::run(matches);
        return;
    }
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

    let mut context = Context::new();
    //// Converting this to a vector here as the template was printing out the package list
    //// in reverse order when given the index map
    //let packages: Vec<&Package> = bom.packages.iter().map(|(_id, pkg)| pkg).collect();
    context.insert("app_name", name);
    context.insert("origen_version", &origen::STATUS.origen_version.to_string());
    let mut user_info = "".to_string();
    let users = crate::om::users();
    if let Ok(u) = users.current_user() {
        if let Ok(username) = u.username() {
            user_info += &username;
            match u.get_email() {
                Ok(e) => {
                    if let Some(email) = e {
                        user_info += &format!(" <{}>", &email);
                    }
                }
                Err(e) => {
                    origen::display_redln!("{}", e.msg);
                }
            }
        }
    }
    context.insert("user_info", &user_info);

    let new_app = App {
        name: name.to_string(),
        dir: app_dir,
    };

    new_app.apply_template(&PY_APP, &context);

    if !matches.is_present("no-setup") {
        new_app.setup();
    }
}

impl App {
    fn apply_template(&self, template: &Map<&str, &str>, context: &Context) {
        let mut tera = Tera::default();

        for (file, content) in template.entries() {
            let contents = tera.render_str(content, &context).unwrap();

            let file = file.replace("app_namespace_dir", &self.name);
            let path = self.dir.join(file.clone());

            if !path.parent().unwrap().exists() {
                std::fs::create_dir_all(&path.parent().unwrap())
                    .expect("Couldn't create dir within the new app");
            }

            display_green!("      create  ");
            displayln!("{}", &file);

            std::fs::write(&path, &contents).expect("Couldn't create a file within the new app");
        }
    }

    fn setup(&self) {
        std::env::set_current_dir(&self.dir).expect("Couldn't cd to the new app");

        let _ = std::process::Command::new("origen")
            .arg("env")
            .arg("setup")
            .spawn()
            .expect("Couldn't execute origen setup")
            .wait();
    }
}
