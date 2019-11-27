use origen::core::application::target;
use origen::STATUS;
use pathdiff::diff_paths;

pub fn run(tname: Option<&str>) {
    if tname.is_none() {
        if target::CURRENT_TARGET.target_name.is_some() {
            let name = target::CURRENT_TARGET.target_name.clone().unwrap();
            println!("{}", name);
        } else {
            println!("No default target is currently enabled in this workspace");
        }
    } else {
        let name = tname.unwrap();
        if name == "default" {
            target::delete_val("target");
        } else {
            let matches = target::matches(name, "targets");

            if matches.len() == 0 {
                println!("No matching target found, here are the available targets:");
                for file in target::all("targets").iter() {
                    println!(
                        "    {}",
                        diff_paths(&file, &STATUS.root.join("targets"))
                            .unwrap()
                            .display()
                    );
                }
            } else if matches.len() > 1 {
                println!("That target name is ambiguous, please try again to narrow it down to one of these:");
                for file in matches.iter() {
                    println!(
                        "    {}",
                        diff_paths(&file, &STATUS.root.join("targets"))
                            .unwrap()
                            .display()
                    );
                }
            } else {
                let clean_name = format!(
                    "{}",
                    diff_paths(&matches[0], &STATUS.root.join("targets"))
                        .unwrap()
                        .display()
                );
                target::set_workspace("target", &clean_name);
                println!("Your workspace target is now set to: {}", clean_name);
            }
        }
    }
}
