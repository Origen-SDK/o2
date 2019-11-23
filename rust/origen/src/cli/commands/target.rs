use core::application::target;
use pathdiff::diff_paths;
use core::STATUS;

pub fn main(name: &str) {

    if name == "__none__" {

    } else {
        if name == "default" {
        } else {
            let matches = target::matches(name, "targets");

            if matches.len() == 0 {
                println!("No matching target found, here are the available targets:");
                for file in  target::all("targets").iter() {
                    println!("    {}", diff_paths(&file, &STATUS.root.join("targets")).unwrap().display());
                }
                
            } else if matches.len() > 1 {
                println!("That target name is ambiguous, please try again to narrow it down to one of these:");
                for file in  matches.iter() {
                    println!("    {}", diff_paths(&file, &STATUS.root.join("targets")).unwrap().display());
                }

            } else {
                let clean_name = format!("{}", diff_paths(&matches[0], &STATUS.root.join("targets")).unwrap().display());
                target::set_workspace("target", &clean_name);
                println!("Your workspace target is now set to: {}", clean_name);
            }
        }
    }
}
