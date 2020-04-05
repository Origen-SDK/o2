use origen::core::application::target;
use origen::{clean_mode, app_config};

pub fn run(mname: Option<&str>) {
    if mname.is_none() {
        println!("{}", app_config().mode);
    } else {
        let name = mname.unwrap();
        if name == "default" {
            target::delete_val("mode");
        } else {
            let c = clean_mode(name);
            target::set_workspace("mode", &c);
            println!("Your default workspace mode is now set to: {}", c);
        }
    }
}
