use std::collections::HashMap;
use origen::{Result, in_app_invocation};
use super::plugins::Plugin;
use super::aux_cmds::{AuxCmdNamespace};
use clap::Command as ClapCommand;
use super::{ArgTOML, Arg, OptTOML, Opt, CmdSrc};
use crate::{from_toml_args, from_toml_opts};
use std::path::PathBuf;
use std::{fmt, env};

// TODO refactor this
use super::helps::CmdSrc as ExtensionTarget;

// macro_rules! core_cmd {
//     // ($src: expr) => { crate::ExtensionSource::Core($src) }
//     // ($src: expr) => { crate::ExtensionSource::Plugin($src) }
//     // ($src: expr, $app: expr) => {
//         // let mut split = cmd.split('.');
//         // let mut sub = app.find_subcommand_mut(split.next().unwrap()).unwrap();
//     // }
// }

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
        where F: Fn(ExtensionTOML) -> Result<Extension>
    {
        let c = CmdSrc::new(&t.extend)?;
        let e = f(t)?;
        if !e.should_extend_in_env()? {
            return Ok(false);
        }
        if in_app_invocation() {
            // App is present
            if !e.should_extend_app_context() {
                return Ok(false)
            }
        } else {
            if c.is_app_cmd() {
                // Outside of an app but extending an app command - implicitly skip these
                return Ok(false)
            } else {
                if !e.should_extend_global_context() {
                    return Ok(false)
                }
            }
        }
        self.extensions.entry(c).or_default().push(e);
        Ok(true)
    }

    pub fn add_from_app_toml(&mut self, ext_toml: ExtensionTOML) -> Result<bool> {
        // println!("Extending");
        // let t = ExtensionTarget::new(&ext_toml.extend)?;
        // let e = Extension::from_extension_toml(ExtensionSource::App, ext_toml)?;
        self.add_ext(ext_toml, |t| Extension::from_extension_toml(ExtensionSource::App, t))

        // self.extensions.entry(ExtensionTarget::new(&ext_toml.extend)?).or_default().push(
        //     Extension::from_extension_toml(ExtensionSource::App, ext_toml)?
        // );
        // println!("Ext: {:?}", self.extensions.keys().collect::<Vec<&ExtensionTarget>>());
        // Ok(())
    }

    pub fn add_from_pl_toml(&mut self, pl: &Plugin, ext_toml: ExtensionTOML) -> Result<bool> {
        // println!("Extending");
        // let t = ExtensionTarget::new(&ext_toml.extend)?;
        // let e = Extension::from_extension_toml(ExtensionSource::Plugin(pl.name.to_owned()), ext_toml)?;
        self.add_ext(ext_toml, |t| Extension::from_extension_toml(ExtensionSource::Plugin(pl.name.to_owned()), t))
        // if self.should_add_ext(t, e) {
        // // if !app_present!() && (t.is_app_cmd() || e.extend_in_app_context_only()) {
        // //     return Ok(())
        // // }
        //     self.extensions.entry(t).or_default().push(e);
        // }
        // // println!("Ext: {:?}", self.extensions.keys().collect::<Vec<&ExtensionTarget>>());
        // Ok(())
    }

    pub fn add_from_aux_toml(&mut self, ns: &AuxCmdNamespace, ext_toml: ExtensionTOML) -> Result<bool> {
        // println!("Extending");
        // let t = ExtensionTarget::new(&ext_toml.extend)?;
        // let e = Extension::from_extension_toml(ExtensionSource::Aux(ns.namespace(), ns.root()), ext_toml)?;
        self.add_ext(ext_toml, |t| Extension::from_extension_toml(ExtensionSource::Aux(ns.namespace(), ns.root()), t))
        // if self.should_add_ext(t, e) {
        //     self.extensions.entry(t).or_default().push(e);
        // }

        // self.extensions.entry(ExtensionTarget::new(&ext_toml.extend)?).or_default().push(
        //     Extension::from_extension_toml(ExtensionSource::Aux(ns.namespace(), ns.root()), ext_toml)?
        // );
        // println!("Ext: {:?}", self.extensions.keys().collect::<Vec<&ExtensionTarget>>());
        // Ok(())
    }

    // pub fn add_from_app_toml(&mut self, ExtensionSource, ExtensionTOML) -> Result<()> {
    //     ?
    // }

    // pub fn add_from_toml(&mut self, src: ExtensionSource, ext_toml: ExtensionTOML) -> Result<()> {
    //     todo!()
    // }

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
            // TODO
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
    pub fn from_extension_toml(ext_source: ExtensionSource, ext: ExtensionTOML) -> Result<Self> {
        let mut slf = Self {
            extends: ext.extend,
            in_global_context: ext.in_global_context,
            in_app_context: ext.in_app_context,
            on_env: ext.on_env,
            args: from_toml_args!(ext.arg),
            opts: from_toml_opts!(ext.opt),
            source: ext_source,
        };
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
        Ok(slf)
    }

    pub fn should_extend_global_context(&self) -> bool {
        self.in_global_context.unwrap_or(true)
    }

    pub fn should_extend_app_context(&self) -> bool {
        self.in_app_context.unwrap_or(true)
    }

    pub fn should_extend_in_env(&self) -> Result<bool> {
        if let Some(envs) = self.on_env.as_ref() {
            for e in envs {
                let mut s = e.splitn(1, '=');
                let e_name= s.next().ok_or_else( || format!("Failed to parse 'on_env' '{}', extending '{}', for {}", e, self.extends, self.source))?.trim();
                let e_val = s.next();
                match env::var(e_name) {
                    Ok(val) => {
                        if let Some(v) = e_val {
                            if v == val {
                                return Ok(true);
                            }
                        } else {
                            return Ok(true);
                        }
                    },
                    Err(err) => match err {
                        env::VarError::NotPresent => {},
                        _ => {
                            return Err(err.into());
                        }
                    }
                }
            }
            Ok(false)
        } else {
            Ok(true)
        }
    }
}
