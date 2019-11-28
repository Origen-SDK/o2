use origen::{clean_mode, APPLICATION_CONFIG};
use origen::core::application::target;

pub fn run(mname: Option<&str>) {
    if mname.is_none() {
        println!("{}", APPLICATION_CONFIG.mode);
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
