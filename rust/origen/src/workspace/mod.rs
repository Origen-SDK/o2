use std::env;
use std::path::PathBuf;

// Global configuration singleton
pub struct Config {
    pub is_app_present: bool,
    pub root: PathBuf,
}

impl Default for Config {
    fn default () -> Config {
        let (p, r) = search_for_app_root();
        Config {
            is_app_present: p,
            root: r,
        }
    }
}

fn search_for_app_root() -> (bool, PathBuf) {
    let mut aborted = false;
    let path = env::current_dir();
    let mut path = match path {
        Ok(p) => p,
        Err(_error) => {
            return (false, PathBuf::new());
        }
    };

    while !path.join("config").join("application.toml").is_file() && !aborted {
        if !path.pop() {
            aborted = true;
        }
    }

    if aborted {
        (false, PathBuf::new())
    } else {
        (true, path)
    }
}
