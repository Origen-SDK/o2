use origen::core::application::target;
use origen::{backend_expect, backend_fail};

pub fn run(subcmd: Option<&str>, tnames: Option<Vec<&str>>, full_paths: bool) {
    if let Some(cmd) = subcmd {
        match cmd {
            "add" => {
                target::add(backend_expect!(
                    tnames,
                    "No targets given to 'target add' cmd!"
                ));
            }
            "default" => {
                target::reset();
            }
            "remove" => {
                target::remove(backend_expect!(
                    tnames,
                    "No targets given to 'target add' cmd!"
                ));
            }
            "set" => {
                target::set(backend_expect!(
                    tnames,
                    "No targets given to 'target set' cmd!"
                ));
            }
            "view" => {
                if let Some(targets) = target::get(full_paths) {
                    println!("The targets currently enabled are:");
                    println!("{}", targets.join("\n"))
                } else {
                    println!("No targets have been enabled and this workspace does not enable any default targets")
                }
                return ();
            }
            _ => {
                // Shouldn't hit this. Should be caught by clap before getting here
                backend_fail!("Unknown subcommand in target processor");
            }
        }
    }
    run(Some("view"), None, full_paths)
}
