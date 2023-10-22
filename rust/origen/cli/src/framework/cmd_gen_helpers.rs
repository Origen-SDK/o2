/// Generate a required, single-value arg
#[macro_export]
macro_rules! req_sv_arg {
    ($name: expr, $value_name: expr, $help: expr) => {{
        clap::Arg::new($name)
            .help($help)
            .action(crate::commands::_prelude::clap_arg_actions::SetArg)
            .value_name($value_name)
            .multiple(false)
            .required(true)
    }}
}

/// Generate an optional, single-value option
#[macro_export]
macro_rules! sv_opt {
    ($name: expr, $value_name: expr, $help: expr) => {{
        clap::Arg::new($name)
            .long($name)
            .help($help)
            .action(crate::commands::_prelude::clap_arg_actions::SetArg)
            .value_name($value_name)
            .multiple(false)
            .required(false)
    }}
}