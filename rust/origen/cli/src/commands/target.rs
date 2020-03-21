use origen::core::application::target;
use origen::APPLICATION_CONFIG;

pub fn run(subcmd: Option<&str>, tnames: Option<Vec<&str>>) {
    if subcmd.is_none() {
        run(Some("view"), None)
    } else {
        if tnames.is_none() {
            if APPLICATION_CONFIG.target.is_some() {
                let name = APPLICATION_CONFIG.target.clone().unwrap();
                println!("{}", name);
            } else {
                println!("No default target is currently enabled in this workspace");
            }
        } else {
            for name in tnames.unwrap().iter() {
                //let name = tname.unwrap();
                if name == &"default" {
                    target::delete_val("target");
                } else {
                    let c = target::clean_name(name, "targets", false);
                    target::set_workspace("target", &c);
                    println!("Your workspace target is now set to: {}", c);
                }
            }
        }
    }
}
