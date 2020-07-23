use origen::clean_mode;
use origen::core::application::target;

pub fn run(mname: Option<&str>) {
    if mname.is_none() {
        let _ = origen::app().unwrap().with_config(|config| {
            println!("{}", config.mode);
            Ok(())
        });
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
