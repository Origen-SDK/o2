use crate::commands::_prelude::*;

pub const BASE_CMD: &'static str = "generate";

gen_core_cmd_funcs!(
    BASE_CMD,
    "Generate patterns or test programs",
    { |cmd: App<'a>| {
        cmd.visible_alias("g")
            .arg(
                Arg::new("files")
                    .help("The name of the file(s) to be generated")
                    .action(AppendArgs)
                    .value_name("FILES")
                    .multiple(true)
                    .required(true),
            )
            .arg(output_dir_opt!())
            .arg(ref_dir_opt!())
            // TODO re-add debug opt
            // .arg(
            //     Arg::new("debug")
            //         .long("debug")
            //         .short('d')
            //         .help("Enable Python caller tracking for debug (takes longer to execute)")
            //         .action(SetArgTrue),
            // )
    }}
);

gen_simple_run_func!();
