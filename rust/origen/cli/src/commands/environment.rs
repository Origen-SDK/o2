use origen::core::application::target;
use origen::STATUS;
use pathdiff::diff_paths;

pub fn run(tname: Option<&str>) {
    if tname.is_none() {
        if target::CURRENT_TARGET.env_name.is_some() {
            let name = target::CURRENT_TARGET.env_name.clone().unwrap();
            println!("{}", name);
        } else {
            println!("No default environment is currently enabled in this workspace");
        }
    } else {
        let name = tname.unwrap();
        if name == "default" {
            target::delete_val("environment");
        } else {
            let matches = target::matches(name, "environments");

            if matches.len() == 0 {
                println!("No matching environment found, here are the available environments:");
                for file in target::all("environments").iter() {
                    println!(
                        "    {}",
                        diff_paths(&file, &STATUS.root.join("environments"))
                            .unwrap()
                            .display()
                    );
                }
            } else if matches.len() > 1 {
                println!("That environment name is ambiguous, please try again to narrow it down to one of these:");
                for file in matches.iter() {
                    println!(
                        "    {}",
                        diff_paths(&file, &STATUS.root.join("environments"))
                            .unwrap()
                            .display()
                    );
                }
            } else {
                let clean_name = format!(
                    "{}",
                    diff_paths(&matches[0], &STATUS.root.join("environments"))
                        .unwrap()
                        .display()
                );
                target::set_workspace("environment", &clean_name);
                println!("Your workspace environment is now set to: {}", clean_name);
            }
        }
    }
}
