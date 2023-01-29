use std::collections::HashMap;
use origen::{Result, in_app_invocation};
use super::plugins::Plugin;
use super::aux_cmds::{AuxCmdNamespace};
use clap::Command as ClapCommand;
use super::{ArgTOML, Arg, OptTOML, Opt, CmdSrc, Applies};
use crate::{from_toml_args, from_toml_opts};
use std::path::PathBuf;
use std::{fmt, env};

// TODO refactor this
use super::helps::CmdSrc as ExtensionTarget;


#[derive(Debug)]
pub struct Extensions {
    extensions: HashMap<ExtensionTarget, Vec<Extension>>,
}

impl Extensions {
    pub fn new() -> Self {
        Self {
            extensions: HashMap::new(),
        }
    }

    pub fn exts(&self) -> &HashMap<ExtensionTarget, Vec<Extension>> {
        &self.extensions
    }

    fn add_ext<F>(&mut self, t: ExtensionTOML, f: F) -> Result<bool>
        where F: Fn(ExtensionTOML) -> Result<Option<Extension>>
    {
        let c = CmdSrc::new(&t.extend)?;
        if let Some(e) = f(t)? {
            self.extensions.entry(c).or_default().push(e);
            Ok(true)
        } else {
            // Extension doesn't apply in this context/env.
            Ok(false)
        }
    }

    pub fn add_from_app_toml(&mut self, ext_toml: ExtensionTOML) -> Result<bool> {
        self.add_ext(ext_toml, |t| Extension::from_extension_toml(ExtensionSource::App, t))
    }

    pub fn add_from_pl_toml(&mut self, pl: &Plugin, ext_toml: ExtensionTOML) -> Result<bool> {
        self.add_ext(ext_toml, |t| Extension::from_extension_toml(ExtensionSource::Plugin(pl.name.to_owned()), t))
    }

    pub fn add_from_aux_toml(&mut self, ns: &AuxCmdNamespace, ext_toml: ExtensionTOML) -> Result<bool> {
        self.add_ext(ext_toml, |t| Extension::from_extension_toml(ExtensionSource::Aux(ns.namespace(), ns.root()), t))
    }

    pub fn apply_to_core_cmd<'a>(&'a self, cmd: &str, app: ClapCommand<'a>) -> ClapCommand<'a> {
        self.apply_to(&ExtensionTarget::Core(cmd.to_string()), app)
    }

    pub fn apply_to_app_cmd<'a>(&'a self, cmd: &str, app: ClapCommand<'a>) -> ClapCommand<'a> {
        self.apply_to(&ExtensionTarget::App(cmd.to_string()), app)
    }

    pub fn apply_to_pl_cmd<'a>(&'a self, pl: &str, cmd: &str, app: ClapCommand<'a>) -> ClapCommand<'a> {
        self.apply_to(&ExtensionTarget::Plugin(pl.to_string(), cmd.to_string()), app)
    }

    pub fn apply_to_aux_cmd<'a>(&'a self, ns: &str, cmd: &str, app: ClapCommand<'a>) -> ClapCommand<'a> {
        self.apply_to(&ExtensionTarget::Aux(ns.to_string(), cmd.to_string()), app)
    }

    pub fn apply_to<'a>(&'a self, cmd: &ExtensionTarget, mut app: ClapCommand<'a>) -> ClapCommand<'a> {
        if let Some(exts) = self.extensions.get(cmd) {
            for ext in exts {
                if let Some(args) = ext.args.as_ref() {
                    app = super::apply_args(args, app);
                }
                if let Some(opts) = ext.opts.as_ref() {
                    app = super::apply_opts(opts, app);
                }
            }
        } else {
            // println!("No extension found for {:?}", cmd);
            // FOR_PR
        }
        app
    }

    pub fn get_core_ext(&self, cmd_path: &str) -> Option<&Vec<Extension>> {
        self.extensions.get(&ExtensionTarget::Core(cmd_path.to_string()))
    }

    pub fn get_core_subc_ext(&self, cmd_path: &[&str]) -> Option<&Vec<Extension>> {
        self.extensions.get(&ExtensionTarget::Core(cmd_path.join(".")))
    }

    pub fn get_app_ext(&self, cmd_path: &str) -> Option<&Vec<Extension>> {
        self.extensions.get(&ExtensionTarget::App(cmd_path.to_string()))
    }

    pub fn get_pl_ext(&self, pl: &str, cmd_path: &str) -> Option<&Vec<Extension>> {
        self.extensions.get(&ExtensionTarget::Plugin(pl.to_string(), cmd_path.to_string()))
    }

    pub fn get_aux_ext(&self, ns: &str, cmd_path: &str) -> Option<&Vec<Extension>> {
        self.extensions.get(&ExtensionTarget::Aux(ns.to_string(), cmd_path.to_string()))
    }
}

#[derive(Debug, Deserialize)]
pub struct ExtensionTOML {
    pub extend: String, // Command to extend
    pub in_global_context: Option<bool>, // Extend in the global context
    pub in_app_context: Option<bool>, // Extend in application context
    pub on_env: Option<Vec<String>>,
    pub arg: Option<Vec<ArgTOML>>,
    pub opt: Option<Vec<OptTOML>>,
    // TODO see about supporting some of these in the future?
    // pub name: String,
    // pub help: String,
    // pub alias: Option<String>,
    // pub arg: Option<Vec<Arg>>,
    // pub subcommands: Option<Vec<String>>,
    // pub full_name: String,
}

#[derive(Debug, Hash, Eq, PartialEq)]
pub enum ExtensionSource {
    App,
    Plugin(String),
    Aux(String, PathBuf),
}

impl fmt::Display for ExtensionSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::App => write!(f, "the App"),
            Self::Plugin(pl_name) => write!(f, "plugin '{}'", pl_name),
            Self::Aux(ns, _) => write!(f, "aux namespace '{}'", ns)
        }
    }
}

#[derive(Debug)]
pub struct Extension {
    pub extends: String,
    pub in_global_context: Option<bool>,
    pub in_app_context: Option<bool>,
    pub on_env: Option<Vec<String>>,
    pub args: Option<Vec<Arg>>,
    pub opts: Option<Vec<Opt>>,

    pub source: ExtensionSource,
}

impl Extension {
    pub fn from_extension_toml(ext_source: ExtensionSource, ext: ExtensionTOML) -> Result<Option<Self>> {
        let mut slf = Self {
            in_global_context: ext.in_global_context,
            in_app_context: ext.in_app_context,
            on_env: ext.on_env,
            args: None,
            opts: None,
            source: ext_source,
            extends: ext.extend,
        };
        if !slf.applies()? {
            return Ok(None)
        }
        slf.args = from_toml_args!(ext.arg);
        slf.opts = from_toml_opts!(ext.opt, &slf.extends, &slf.source, Some(&slf.extends));
        if let Some(opts) = slf.opts.as_mut() {
            for opt in opts {
                opt.help += &format!(" [Extended from {}]",
                    match slf.source {
                        ExtensionSource::App => {
                            "the app".to_string()
                        },
                        ExtensionSource::Plugin(ref pl_name) => {
                            format!("plugin: '{}'", pl_name)
                        },
                        ExtensionSource::Aux(ref ns, _) => {
                            format!("aux namespace: '{}'", ns)
                        }
                    }
                );
            }
        }
        Ok(Some(slf))
    }
}

impl Applies for Extension {
    fn in_global_context(&self) -> Option<bool> {
        self.in_global_context
    }

    fn in_app_context(&self) -> Option<bool> {
        self.in_app_context
    }

    fn on_env(&self) -> Option<&Vec<String>> {
        self.on_env.as_ref()
    }

    fn on_env_error_msg(&self, e: &String) -> String {
        format!("Failed to parse 'on_env' '{}', extending '{}', for {}", e, self.extends, self.source)
    }
}
