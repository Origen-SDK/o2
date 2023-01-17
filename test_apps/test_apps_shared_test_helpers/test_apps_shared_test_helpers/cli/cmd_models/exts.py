from origen.helpers.regressions import cli
from . import CmdExtOpt
from .auxs import aux_cmds_dir

class ExtensionDrivers:
    exts_workout_cfg = aux_cmds_dir.joinpath("exts_workout_cfg.toml")
    exts_workout_toml = aux_cmds_dir.joinpath("exts_workout.toml")
    pl_ext_stacking_from_aux_cfg = aux_cmds_dir.joinpath("pl_ext_stacking_from_aux_cfg.toml")
    pl_ext_stacking_from_aux_toml = aux_cmds_dir.joinpath("pl_ext_stacking_from_aux.toml")
    core_cmd_exts_cfg = aux_cmds_dir.joinpath("core_cmd_exts_cfg.toml")
    core_cmd_exts_toml = aux_cmds_dir.joinpath("core_cmd_exts.toml")
    enable_dummy_cmds_exts_env = {"ORIGEN_DUMMY_AUX_CMDS": "1"}

    test_apps_shared_generic_exts = CmdExtOpt.from_src(
        "test_apps_shared_test_helpers",
        cli.cmd.SrcTypes.PLUGIN,
        CmdExtOpt(
            "test_apps_shared_ext_action",
            help="Action from test_apps_shared_test_helpers plugin",
            multi=True,
        ),
        CmdExtOpt(
            "test_apps_shared_ext_flag",
            help="Flag from test_apps_shared_test_helpers plugin",
        ),
    )

    exts = {
        "core.eval": {
            "exts": [
                *CmdExtOpt.from_src(
                    "exts_workout",
                    cli.cmd.SrcTypes.AUX,
                    CmdExtOpt(
                        "ext_action",
                        multi=True,
                        help="Action for the extended opt",
                        ln="action",
                    ),
                ),
                *CmdExtOpt.from_src(
                    "python_plugin",
                    cli.cmd.SrcTypes.PLUGIN,
                    CmdExtOpt(
                        "say_hi_before_eval",
                        help="Have the plugin say hi before evaluating (app)",
                    ),
                    CmdExtOpt(
                        "say_hi_after_eval",
                        help="Have the plugin say hi after evaluating (app)",
                    ),
                    CmdExtOpt(
                        "say_hi_during_cleanup",
                        help="Have the plugin say hi during cleanup",
                    ),
                ),
            ]
        },
        "plugin.python_plugin.plugin_test_args": {
            "exts": CmdExtOpt.from_src(
                "exts_workout",
                cli.cmd.SrcTypes.AUX,
                CmdExtOpt(
                    "flag_extension",
                    help="Single flag extension",
                    sn="f",
                    ln="flag_ext",
                ),
                CmdExtOpt(
                    "single_val_opt",
                    takes_value=True,
                    sn="s",
                    help="Extended opt taking a single value",
                ),
                CmdExtOpt(
                    "multi_val_opt",
                    ln="multi",
                    sn_aliases=["m"],
                    ln_aliases=["multi_non_delim"],
                    multi=True,
                    value_name="MULTI_VAL",
                    help="Extended opt taking a multiple, non-delimited values",
                ),
                CmdExtOpt(
                    "multi_val_delim_opt",
                    ln_aliases=["multi_delim"],
                    multi=True,
                    use_delimiter=True,
                    help="Extended opt taking a multiple, delimited values",
                ),
                CmdExtOpt(
                    "exts_workout_action",
                    takes_value=True,
                    required=True,
                    multi=True,
                    help="Additional actions for testing purposes",
                ),
                CmdExtOpt(
                    "hidden_opt",
                    hidden=True,
                    help="Hidden extended opt",
                ),
            ),
            "toml": exts_workout_toml,
        },
        "plugin.python_plugin.plugin_test_args.subc": {
            "exts": CmdExtOpt.from_src(
                "exts_workout",
                cli.cmd.SrcTypes.AUX,
                CmdExtOpt(
                    "exts_workout_action",
                    multi=True,
                    help="Action for the extended opt",
                    ln="action",
                ),
            ),
            "toml": exts_workout_toml,
        },
        "plugin.python_plugin.plugin_test_ext_stacking": {
            "exts": [
                *CmdExtOpt.from_src(
                    "exts_workout",
                    cli.cmd.SrcTypes.AUX,
                    CmdExtOpt(
                        "ext_action",
                        multi=True,
                        help="Action for the extended opt",
                        ln="action",
                    ),
                ),
                *CmdExtOpt.from_src(
                    "pl_ext_stacking_from_aux",
                    cli.cmd.SrcTypes.AUX,
                    CmdExtOpt(
                        "pl_ext_stacking_from_aux_action",
                        multi=True,
                        help="Action from pl_ext_stacking aux cmds",
                    ),
                    CmdExtOpt(
                        "pl_ext_stacking_from_aux_flag",
                        help="Flag from pl_ext_stacking aux cmds",
                    ),
                ),
                *CmdExtOpt.from_src(
                    "python_plugin_the_second",
                    cli.cmd.SrcTypes.PLUGIN,
                    CmdExtOpt(
                        "pl_the_2nd_ext_action",
                        help="Action from pl_the_2nd plugin",
                        multi=True,
                    ),
                    CmdExtOpt(
                        "pl_the_2nd_ext_flag",
                        help="Flag from pl_the_2nd plugin",
                    ),
                )
            ]
        },
        "plugin.python_plugin.plugin_test_ext_stacking.subc": {
            "exts": [
                *CmdExtOpt.from_src(
                    "exts_workout",
                    cli.cmd.SrcTypes.AUX,
                    CmdExtOpt(
                        "ext_action_subc",
                        multi=True,
                        help="Action for the extended opt subc",
                        ln="action",
                    ),
                ),
                *CmdExtOpt.from_src(
                    "pl_ext_stacking_from_aux",
                    cli.cmd.SrcTypes.AUX,
                    CmdExtOpt(
                        "pl_ext_stacking_from_aux_action_subc",
                        multi=True,
                        help="Action from pl_ext_stacking aux cmds subc",
                    ),
                    CmdExtOpt(
                        "pl_ext_stacking_from_aux_flag_subc",
                        help="Flag from pl_ext_stacking aux cmds subc",
                    ),
                ),
                *CmdExtOpt.from_src(
                    "python_plugin_the_second",
                    cli.cmd.SrcTypes.PLUGIN,
                    CmdExtOpt(
                        "pl_the_2nd_ext_action_subc",
                        help="Action from pl_the_2nd plugin subc",
                        multi=True,
                    ),
                    CmdExtOpt(
                        "pl_the_2nd_ext_flag_subc",
                        help="Flag from pl_the_2nd plugin subc",
                    ),
                ),
                *test_apps_shared_generic_exts
            ],
        },
        "aux.dummy_cmds.dummy_cmd": {
            "exts": [
                *CmdExtOpt.from_src(
                    "exts_workout",
                    cli.cmd.SrcTypes.AUX,
                    CmdExtOpt(
                        "exts_workout_action",
                        multi=True,
                        help="Action for the extended opt",
                        ln="action",
                    ),
                    CmdExtOpt(
                        "exts_workout_flag",
                        help="Flag for the extended opt",
                    ),
                ),
                *CmdExtOpt.from_src(
                    "pl_ext_stacking_from_aux",
                    cli.cmd.SrcTypes.AUX,
                    CmdExtOpt(
                        "pl_ext_stacking_from_aux_action",
                        multi=True,
                        help="Action from pl_ext_stacking aux cmds",
                    ),
                    CmdExtOpt(
                        "pl_ext_stacking_from_aux_flag",
                        help="Flag from pl_ext_stacking aux cmds",
                    ),
                ),
                *CmdExtOpt.from_src(
                    "python_plugin",
                    cli.cmd.SrcTypes.PLUGIN,
                    CmdExtOpt(
                        "python_plugin_action",
                        help="Action from python_plugin",
                        multi=True,
                    ),
                    CmdExtOpt(
                        "python_plugin_flag",
                        help="Flag from python_plugin",
                    ),
                ),
                *CmdExtOpt.from_src(
                    "python_plugin_the_second",
                    cli.cmd.SrcTypes.PLUGIN,
                    CmdExtOpt(
                        "python_plugin_the_second_action",
                        help="Action from pl_the_2nd plugin",
                        multi=True,
                    ),
                    CmdExtOpt(
                        "python_plugin_the_second_flag",
                        help="Flag from pl_the_2nd plugin",
                    ),
                ),
            ],
            "env": enable_dummy_cmds_exts_env
        },
        "aux.dummy_cmds.dummy_cmd.subc": {
            "exts": [
                *CmdExtOpt.from_src(
                    "exts_workout",
                    cli.cmd.SrcTypes.AUX,
                    CmdExtOpt(
                        "exts_workout_action",
                        multi=True,
                        help="Action for the extended opt subc",
                        ln="action",
                    ),
                    CmdExtOpt(
                        "exts_workout_flag_subc",
                        help="Flag for the extended opt subc",
                    ),
                ),
                *CmdExtOpt.from_src(
                    "pl_ext_stacking_from_aux",
                    cli.cmd.SrcTypes.AUX,
                    CmdExtOpt(
                        "pl_ext_stacking_from_aux_action_subc",
                        multi=True,
                        help="Action from pl_ext_stacking aux cmds subc",
                    ),
                    CmdExtOpt(
                        "pl_ext_stacking_from_aux_flag_subc",
                        help="Flag from pl_ext_stacking aux cmds subc",
                    ),
                ),
                *CmdExtOpt.from_src(
                    "python_plugin",
                    cli.cmd.SrcTypes.PLUGIN,
                    CmdExtOpt(
                        "python_plugin_action_subc",
                        help="Action from python_plugin subc",
                        multi=True,
                    ),
                    CmdExtOpt(
                        "python_plugin_flag_subc",
                        help="Flag from python_plugin subc",
                    ),
                ),
                *CmdExtOpt.from_src(
                    "python_plugin_the_second",
                    cli.cmd.SrcTypes.PLUGIN,
                    CmdExtOpt(
                        "python_plugin_the_second_action_subc",
                        help="Action from pl_the_2nd plugin subc",
                        multi=True,
                    ),
                    CmdExtOpt(
                        "python_plugin_the_second_flag_subc",
                        help="Flag from pl_the_2nd plugin subc",
                    ),
                ),
            ],
            "env": enable_dummy_cmds_exts_env
        },
        "generic_core_ext": {
            "exts": [
                *CmdExtOpt.from_src(
                    "pl_ext_cmds",
                    cli.cmd.SrcTypes.PLUGIN,
                    CmdExtOpt(
                        "pl_ext_cmds_generic_ext",
                        help="Generic ext from pl_ext_cmds plugin",
                    ),
                ),
                *CmdExtOpt.from_src(
                    "core_cmd_exts",
                    cli.cmd.SrcTypes.AUX,
                    CmdExtOpt(
                        "core_cmd_exts_generic_core_ext",
                        help="Generic core ext from aux commands",
                    ),
                ),
            ]
        }
    }