use std::env;
use config::{ConfigError, Config, File, Environment};

lazy_static! {
    pub static ref SETTINGS: Settings = Settings::default();
}

#[derive(Debug, Deserialize)]
struct ToolCommands {
    python: String
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    tool_commands: ToolCommands,
}

impl Default for Settings {
    fn default() -> Settings {
        let mut s = Config::new();

        // Start off by merging in the "default" configuration file
        s.merge(File::with_name("config/default"));

        // To get the path of the Origen binary:
        // https://doc.rust-lang.org/std/env/fn.current_exe.html

        //// Add in the current environment file
        //// Default to 'development' env
        //// Note that this file is _optional_
        //let env = env::var("RUN_MODE").unwrap_or("development".into());
        //s.merge(File::with_name(&format!("config/{}", env)).required(false))?;

        //// Add in a local configuration file
        //// This file shouldn't be checked in to git
        //s.merge(File::with_name("config/local").required(false))?;

        // Add in settings from the environment (with a prefix of ORIGEN)
        // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
        s.merge(Environment::with_prefix("origen"));

        //// You may also programmatically change settings
        //s.set("database.url", "postgres://")?;

        //// Now that we're done, let's access our configuration
        //println!("debug: {:?}", s.get_bool("debug"));
        //println!("database: {:?}", s.get::<String>("database.url"));

        s.try_into().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn struct_is_created() {
        assert_eq!(
            SETTINGS.tool_commands.python,
            ""
        );
    }
}
