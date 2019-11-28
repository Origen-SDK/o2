use origen::core::application::target;
use origen::{APPLICATION_CONFIG};

pub fn run(tname: Option<&str>) {
    if tname.is_none() {
        if APPLICATION_CONFIG.environment.is_some() {
            let name = APPLICATION_CONFIG.environment.clone().unwrap();
            println!("{}", name);
        } else {
            println!("No default environment is currently enabled in this workspace");
        }
    } else {
        let name = tname.unwrap();
        if name == "default" {
            target::delete_val("environment");
        } else {
            let c = target::clean_name(name, "environments", false);
            target::set_workspace("environment", &c);
            println!("Your workspace environment is now set to: {}", c);
        }
    }
}
