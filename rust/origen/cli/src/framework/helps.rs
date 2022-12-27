use std::collections::HashMap;
use super::Plugin;
use crate::commands::_prelude::*;
use std::fmt;
use super::extensions::ExtensionSource;

#[derive(Debug)]
pub struct CmdHelps {
    helps: HashMap<CmdSrc, CmdHelp>
}

impl CmdHelps {
    pub fn new() -> Self {
        Self {
            helps: HashMap::new()
        }
    }

    pub fn core_cmd(&self, cmd: &str) -> Command {
        self.apply_core_cmd_helps(cmd, Command::new(cmd))
    }

    pub fn core_subc(&self, cmd_path: &[&str]) -> Command {
        self.apply_core_subc_helps(cmd_path, Command::new(*cmd_path.last().unwrap()))
    }

    pub fn add_core_cmd(&mut self, cmd_name: &str) -> &mut CmdHelp {
        // self.insert(CmdSrc::Core(cmd_name.to_string()), CmdHelp::new());
        // self.get()
        self.helps.entry(CmdSrc::Core(cmd_name.to_string())).or_default()
    }

    pub fn add_core_sub_cmd(&mut self, cmd_path: &[&str]) -> &mut CmdHelp {
        self.helps.entry(CmdSrc::Core(cmd_path.join("."))).or_default()
    }

    pub fn add_app_cmd(&mut self, cmd_name: &str) -> &mut CmdHelp {
        self.helps.entry(CmdSrc::App(cmd_name.to_string())).or_default()
    }

    pub fn add_pl_cmd(&mut self, pl_name: &str, cmd_name: &str) -> &mut CmdHelp {
        // self.insert(CmdSrc::Core(cmd_name.to_string()), CmdHelp::new());
        // self.get()
        // &mut self.entry(CmdSrc::Core(cmd_name.to_string())).and_modify()
        self.helps.entry(CmdSrc::Plugin(pl_name.to_string(), cmd_name.to_string())).or_default()
    }

    pub fn add_aux_cmd(&mut self, ns: &str, cmd_name: &str) -> &mut CmdHelp {
        self.helps.entry(CmdSrc::Aux(ns.to_string(), cmd_name.to_string())).or_default()
    }

    pub fn apply_core_cmd_helps<'a>(&'a self, cmd_name: &str, mut app: Command<'a>) -> Command<'a> {
        self.apply_helps(&CmdSrc::Core(cmd_name.to_string()), app)
    }

    pub fn apply_core_subc_helps<'a>(&'a self, cmd_path: &[&str], mut app: Command<'a>) -> Command<'a> {
        self.apply_helps(&CmdSrc::Core(cmd_path.join(".")), app)
    }

    pub fn apply_helps<'a>(&'a self, cmd_src: &CmdSrc, mut app: Command<'a>) -> Command<'a> {
        if let Some(helps) = self.helps.get(cmd_src) {
            if let Some(h) = helps.before_help.as_ref() {
                app = app.before_help(h.as_str());
            }
            if let Some(h) = helps.help.as_ref() {
                app = app.about(h.as_str());
            }
            if let Some(h) = helps.after_help.as_ref() {
                app = app.after_help(h.as_str());
            }
        } else {
            log_error!("Could not apply help messages to {} - no such command found", cmd_src);
        }
        app
    }

    pub fn apply_exts(&mut self, extensions: &Extensions) {
        for (target, exts) in extensions.exts() {
            if let Some(help) = self.helps.get_mut(&target) {
                let mut extended_from_app = false;
                let mut pls: Vec<&str> = vec!();
                let mut nspaces: Vec<&str> = vec!();
                for ext in exts.iter() {
                    match ext.source {
                        ExtensionSource::App => extended_from_app = true,
                        ExtensionSource::Plugin(ref n) => pls.push(n),
                        ExtensionSource::Aux(ref n, _) => nspaces.push(n),
                    }
                }
                let mut msg = "This command is extended from:".to_string();
                if extended_from_app {
                    msg += "\n    - the App";
                }
                if !pls.is_empty() {
                    msg += &format!(
                        "\n    - Plugins: {}",
                        pls.iter().map(|n| format!("'{}'", n)).collect::<Vec<String>>().join(", ")
                    );
                }
                if !nspaces.is_empty() {
                    msg += &format!(
                        "\n    - Aux Namespaces: {}",
                        nspaces.iter().map(|n| format!("'{}'", n)).collect::<Vec<String>>().join(", ")
                    );
                }
                //  exts.iter().map( |e| e.source.to_string()).collect::<Vec<String>>().join(", ")
                if let Some(after) = help.after_help.as_ref() {
                    help.after_help = Some(after.to_string() + "\n\n" + &msg);
                } else {
                    help.after_help = Some(msg);
                }
            } else {
                // Suppress any app command errors when in the global context
                // if !app_present!() && target.is_app_cmd() {
                    log_error!("Tried to extend unknown command '{}' from:", target);
                    for ext in exts {
                        log_error!("\t{}", ext.source);
                    }
                // }
            }
        }
    }
}

#[derive(Default, Debug)]
pub struct CmdHelp {
    help: Option<String>,
    after_help: Option<String>,
    before_help: Option<String>,
}

impl CmdHelp {
    pub fn set_help_msg(&mut self, help_msg: &str) -> &mut Self {
        self.help = Some(help_msg.to_string());
        self
    }

    pub fn set_optional_help_msg(&mut self, help_msg: Option<String>) -> &mut Self {
        self.help = help_msg;
        self
    }
}


#[derive(Debug, Hash, Eq, PartialEq)]
pub enum Invocation {
    Global,
    App,
}

#[derive(Debug, Hash, Eq, PartialEq)]
pub enum CmdSrc {
    Core(String), // Core command
    App(String), // App command
    Plugin(String, String), // Plugin command
    Aux(String, String), // Aux command
}

impl CmdSrc {
    pub fn new(target: &str) -> Result<Self> {
        let (scope, t) = target.split_once('.').ok_or_else(|| format!("Could not discern scope from '{}'", target))?;
        Ok(match scope {
            "origen" => Self::Core(t.to_string()),
            "app" => Self::App(t.to_string()),
            "plugin" => {
                let (pl_name, pl_t) = t.split_once('.').ok_or_else(|| format!("Could not discern plugin from '{}'", t))?;
                Self::Plugin(pl_name.to_string(), pl_t.to_string())
            }
            "aux" | "aux_ns" => {
                let (ns_name, aux_t) = t.split_once('.').ok_or_else(|| format!("Could not discern auxillary command namespace from '{}'", t))?;
                Self::Aux(ns_name.to_string(), aux_t.to_string())
            }
            _ => bail!("Unknown target scope '{}'. Expected 'origen', 'app', 'aux', or 'plugin'", scope)
        })
    }

    pub fn is_app_cmd(&self) -> bool {
        match self {
            Self::App(_) => true,
            _ => false,
        }
    }
}

impl fmt::Display for CmdSrc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Core(cmd) => {
                write!(f, "origen.{}", cmd)
            },
            Self::App(cmd) => {
                write!(f, "app.{}", cmd)
            },
            Self::Plugin(pl_name, cmd) => {
                write!(f, "plugin.{}.{}", pl_name, cmd)
            },
            Self::Aux(ns_name, cmd) => {
                write!(f, "aux_ns.{}.{}", ns_name, cmd)
            }
        }
    }
}