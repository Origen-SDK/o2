from origen.helpers.regressions import cli
from . import CmdExtOpt
from .auxs import aux_cmds_dir
from types import SimpleNamespace

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

    @classmethod
    def partition_exts(cls, exts):
        tas = {}
        ec = {}
        pypl = {}
        app = {}
        other = []
        for e in exts:
            n = e.name
            if e.src_name == "test_apps_shared_test_helpers":
                tas[n] = e
            elif e.src_name == "ext_conflicts":
                ec[n] = e
            elif e.src_name == "python_plugin":
                pypl[n] = e
            elif e.src_type == cli.cmd.SrcTypes.APP:
                app[n] = e
            else:
                other.append(e)

        partitioned = SimpleNamespace(**{
            "tas": SimpleNamespace(**tas),
            "ec": SimpleNamespace(**ec),
            "pypl": SimpleNamespace(**pypl),
            "app": SimpleNamespace(**app),
            "_other_": other
        })
        return partitioned

    exts = {
        "origen.eval": {
            "global_exts": [
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
                        help="Have the plugin say hi before evaluating (global)",
                        sn="b",
                    ),
                    CmdExtOpt(
                        "say_hi_after_eval",
                        help="Have the plugin say hi after evaluating (global)",
                        sn="a",
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

    ext_conflicts = {
        "plugin.python_plugin.plugin_test_args": {
            "exts": [
                *CmdExtOpt.from_src(
                    "ext_conflicts",
                    cli.cmd.SrcTypes.AUX,
                    CmdExtOpt(
                        "single_arg",
                        help="Conflict with cmd arg",
                    ),
                    CmdExtOpt(
                        "opt_taking_value",
                        help="Conflict with cmd opt name but long name okay",
                        ln="otv",
                    ),
                    CmdExtOpt(
                        "conflict_sn",
                        help="Conflict with cmd opt sn",
                    ),
                    CmdExtOpt(
                        "pl_aux_conflict",
                        help="plugin-aux conflict (Aux)",
                        access_with_full_name=True,
                    ),
                    CmdExtOpt(
                        "pl_aux_sn_conflict_aux",
                        help="plugin-aux conflict (sn) (Aux)",
                    ),
                    CmdExtOpt(
                        "aux_conflict_ln_and_aliases",
                        help="Conflict with cmd opt ln and some aliases (Aux)",
                        ln_aliases=["other_alias_aux"],
                        sn_aliases=["d"],
                    ),
                    CmdExtOpt(
                        "flag",
                        help="Conflict with inferred long name (Aux)",
                        access_with_full_name=True,
                    ),
                    CmdExtOpt(
                        "alias",
                        help="Conflict with inferred long name with aliases (Aux)",
                        ln_aliases=["alias_aux"],
                        access_with_full_name=True,
                    ),
                    CmdExtOpt(
                        "subc",
                        help="Conflict with plugin subcommand"
                    ),
                    CmdExtOpt(
                        "ns_self_conflict",
                        help="Conflict within the namespace",
                    ),
                    CmdExtOpt(
                        "ext_self_conflict",
                        help="Ln conflict within the extension",
                    ),
                    CmdExtOpt(
                        "ext_self_conflict_2",
                        help="Ilna conflicts within the extension",
                        ln_aliases = ["ext_self_conflict_2_1"],
                    ),
                    CmdExtOpt(
                        "ext_opt_in_ln",
                        help="Reserved prefix in ln",
                    ),
                    CmdExtOpt(
                        "ext_opt_in_lna",
                        help="Reserved prefix in lna",
                        ln_aliases = ["ext_opt_in_lna_2"],
                    ),
                    CmdExtOpt(
                        "same_ln_and_ln_alias",
                        help="Same ln and lna",
                    ),
                    CmdExtOpt(
                        "same_iln_and_ln_alias",
                        help="Same inferred ln and lna",
                    ),
                    CmdExtOpt(
                        "repeated_sn_and_aliases",
                        help="Repeated sna and lna, with sn conflict",
                        sn="g",
                        ln_aliases=["repeated_lna", "repeated_lna_2"],
                        sn_aliases=["e"],
                    ),
                ),
                *CmdExtOpt.from_src(
                    "test_apps_shared_test_helpers",
                    cli.cmd.SrcTypes.PLUGIN,
                    CmdExtOpt(
                        "pl_aux_conflict",
                        help="plugin-aux conflict (PL)"
                    ),
                    CmdExtOpt(
                        "pl_aux_sn_conflict_pl",
                        help="plugin-aux conflict (sn) (PL)",
                        sn="s",
                    ),
                    CmdExtOpt(
                        "opt_taking_value",
                        help="Conflict with cmd opt (PL)",
                    ),
                    CmdExtOpt(
                        "pl_conflict_ln_and_aliases",
                        help="Conflict with cmd opt ln and some aliases (PL)",
                        ln_aliases=["other_alias_pl"],
                        sn_aliases=["c"]
                    ),
                    CmdExtOpt(
                        "flag",
                        help="Conflict with inferred long name (PL)",
                        access_with_full_name=True,
                    ),
                )
            ],
            "env": {"ORIGEN_EXT_CONFLICTS_PL_TEST_ARGS": "1"},
            "cfg": aux_cmds_dir.joinpath("ext_conflicts_cfg.toml"),
        },
        "plugin.python_plugin.plugin_test_args.subc": {
            "exts": [
                *CmdExtOpt.from_src(
                    "ext_conflicts",
                    cli.cmd.SrcTypes.AUX,
                    CmdExtOpt(
                        "subc_pl_aux_conflict",
                        help="plugin-aux conflict (AUX)",
                        ln_aliases=["aux0"],
                        sn_aliases=["e"],
                    ),
                    CmdExtOpt(
                        "flag_opt",
                        help="Conflict with inferred long name (AUX)",
                        access_with_full_name=True,
                    ),
                    CmdExtOpt(
                        "more_conflicts",
                        help="More conflicts (AUX)",
                        access_with_full_name=True,
                    ),
                ),
                *CmdExtOpt.from_src(
                    "test_apps_shared_test_helpers",
                    cli.cmd.SrcTypes.PLUGIN,
                    CmdExtOpt(
                        "subc_pl_aux_conflict",
                        help="plugin-aux conflict (PL)",
                        ln="subc_pl_aux",
                        sn="c",
                        ln_aliases=["pl0", "pl1"],
                    ),
                    CmdExtOpt(
                        "flag_opt",
                        help="Conflict with inferred long name (PL)",
                        access_with_full_name=True,
                    ),
                    CmdExtOpt(
                        "more_conflicts",
                        help="More conflicts (PL)",
                        sn_aliases=["d"],
                    ),
                )
            ],
            "env": {"ORIGEN_EXT_CONFLICTS_PL_TEST_ARGS_SUBC": "1"},
            "cfg": aux_cmds_dir.joinpath("ext_conflicts_cfg.toml"),
        },
        "origen.eval": {
            "exts": [
                *CmdExtOpt.from_src(
                    "ext_conflicts",
                    cli.cmd.SrcTypes.AUX,
                    CmdExtOpt(
                        "code",
                        help="Conflict with core cmd arg name from AUX",
                        access_with_full_name=True,
                    )
                ),
                *CmdExtOpt.from_src(
                    "test_apps_shared_test_helpers",
                    cli.cmd.SrcTypes.PLUGIN,
                    CmdExtOpt(
                        "code",
                        help="Conflict with core cmd arg name from PL",
                    )
                ),
            ],
            "env": {"ORIGEN_EXT_CONFLICTS_CORE_CMD_EVAL": "1"},
            "cfg": aux_cmds_dir.joinpath("ext_conflicts_cfg.toml"),
        },
        "origen.credentials.clear": {
            "exts": [
                *CmdExtOpt.from_src(
                    "ext_conflicts",
                    cli.cmd.SrcTypes.AUX,
                    CmdExtOpt(
                        "all",
                        help="Conflict with core cmd opt name. Uses full name",
                        access_with_full_name=True,
                    ),
                    CmdExtOpt(
                        "cmd_conflicts_aux",
                        help="Conflict with core cmd opt from AUX",
                        sn_aliases=["f"],
                    ),
                ),
                *CmdExtOpt.from_src(
                    "test_apps_shared_test_helpers",
                    cli.cmd.SrcTypes.PLUGIN,
                    CmdExtOpt(
                        "all",
                        help="Conflict with core cmd opt name. Uses full name",
                        access_with_full_name=True,
                    ),
                    CmdExtOpt(
                        "cmd_conflicts_pl",
                        help="Conflict with core cmd opt from PL",
                        ln_aliases=["pl_datasets"],
                        sn_aliases=["e"],
                    ),
                )
            ],
            "env": {"ORIGEN_EXT_CONFLICTS_CORE_CMD_CREDS_CLEAR": "1"},
            "cfg": aux_cmds_dir.joinpath("ext_conflicts_cfg.toml"),
        },
    }

    def init_conflicts(self, plugins, aux):
        ext_conflicts = self.ext_conflicts
        n = "exts_by_ns"

        _cmd_str_ = "plugin.python_plugin.plugin_test_args"
        cmd = plugins.python_plugin.plugin_test_args
        exts = self.partition_exts(ext_conflicts[_cmd_str_]["exts"])
        ext_conflicts[_cmd_str_][n] = exts
        ec_displayed = exts.ec.ns_self_conflict.displayed
        ext_conflicts[_cmd_str_]["conflicts_list"] = [
            ["duplicate", exts.tas.pl_aux_conflict, 0],
            ["duplicate", exts.ec.ns_self_conflict, 9],
            ["duplicate", exts.ec.ext_self_conflict, 11],
            ["self_lna_iln", exts.ec.ext_self_conflict_2],
            ["reserved_prefix_ln", exts.ec.ext_opt_in_ln, "ext_opt.in_ln"],
            ["reserved_prefix_lna", exts.ec.ext_opt_in_lna, "ext_opt.in_lna"],
            ["reserved_prefix_opt_name", "ext_opt.reserved_name", ec_displayed],
            ["inter_ext_lna_ln", exts.ec.same_ln_and_ln_alias, "same_ln_and_ln_alias"],
            ["inter_ext_lna_iln", exts.ec.same_iln_and_ln_alias],
            ["inter_ext_sna_sn", exts.ec.repeated_sn_and_aliases, "g"],
            ["repeated_sna", exts.ec.repeated_sn_and_aliases, "e", 1], # Purposefully repeated
            ["repeated_sna", exts.ec.repeated_sn_and_aliases, "e", 1],
            ["repeated_lna", exts.ec.repeated_sn_and_aliases, "repeated_lna", 0],
            ["ln", "lna", exts.tas.pl_conflict_ln_and_aliases, cmd.opt_with_aliases, "alias"],
            ["sn", "sn", exts.tas.pl_conflict_ln_and_aliases, cmd.sn_only, "n"],
            ["lna", "lna", exts.tas.pl_conflict_ln_and_aliases, cmd.opt_with_aliases, "opt_alias"],
            ["sna", "sna", exts.tas.pl_conflict_ln_and_aliases, cmd.opt_with_aliases, "a"],
            ["sna", "sna", exts.tas.pl_conflict_ln_and_aliases, cmd.opt_with_aliases, "b"],
            ["iln", "ln", exts.tas.flag, cmd.flag_opt, "flag"],
            ["sn", "sn", exts.ec.conflict_sn, cmd.sn_only, "n"],
            ["iln", "iln", exts.ec.pl_aux_conflict, exts.tas.pl_aux_conflict, "pl_aux_conflict"],
            ["sn", "sn", exts.ec.pl_aux_sn_conflict_aux, exts.tas.pl_aux_sn_conflict_pl, "s"],
            ["ln", "lna", exts.ec.aux_conflict_ln_and_aliases, cmd.opt_with_aliases, "alias"],
            ["lna", "lna", exts.ec.aux_conflict_ln_and_aliases, cmd.opt_with_aliases, "opt_alias"],
            ["lna", "lna", exts.ec.aux_conflict_ln_and_aliases, exts.tas.pl_conflict_ln_and_aliases, "other_alias_pl"],
            ["sna", "sna", exts.ec.aux_conflict_ln_and_aliases, cmd.opt_with_aliases, "a"],
            ["sna", "sna", exts.ec.aux_conflict_ln_and_aliases, cmd.opt_with_aliases, "b"],
            ["sna", "sna", exts.ec.aux_conflict_ln_and_aliases, exts.tas.pl_conflict_ln_and_aliases, "c"],
            ["iln", "ln", exts.ec.flag, cmd.flag_opt, 'flag'],
            ["iln", "lna", exts.ec.alias, cmd.opt_with_aliases, 'alias'],
            ["ln", "iln", exts.ec.ns_self_conflict, exts.ec.subc, "subc"],
            ["sn", "sna", exts.ec.ns_self_conflict, exts.ec.aux_conflict_ln_and_aliases, "d"],
            ["ln", "iln", exts.ec.ext_self_conflict, exts.ec.ns_self_conflict, "ns_self_conflict"],
            ["lna", "iln", exts.ec.ext_self_conflict_2, exts.ec.ext_self_conflict, "ext_self_conflict"],
        ]

        _cmd_str_ = "plugin.python_plugin.plugin_test_args.subc"
        cmd = plugins.python_plugin.plugin_test_args.subc
        exts = self.partition_exts(ext_conflicts[_cmd_str_]["exts"])
        ext_conflicts[_cmd_str_][n] = exts
        ext_conflicts[_cmd_str_]["conflicts_list"] = [
            ["reserved_prefix_opt_name", "ext_opt.subc_reserved",  exts.tas.subc_pl_aux_conflict.displayed],
            ["inter_ext_sna_sn", exts.tas.subc_pl_aux_conflict, "c"],
            ["inter_ext_lna_ln", exts.tas.subc_pl_aux_conflict, "subc_pl_aux"],
            ["duplicate", exts.tas.subc_pl_aux_conflict, 1],
            ["reserved_prefix_lna", exts.tas.more_conflicts, "ext_opt.subc_lna"],

            ["inter_ext_lna_ln", exts.ec.subc_pl_aux_conflict, "subc_pl_aux"],
            ["duplicate", exts.ec.subc_pl_aux_conflict, 0],
            ["inter_ext_lna_iln", exts.ec.more_conflicts],

            ["sna", "sna", exts.tas.subc_pl_aux_conflict, cmd.subc_opt_with_aliases, "a"],
            ["iln", "iln", exts.tas.flag_opt, cmd.flag_opt],
            ["lna", "ln", exts.tas.more_conflicts, exts.tas.subc_pl_aux_conflict, "subc_pl_aux"],
            ["lna", "ln", exts.tas.more_conflicts, cmd.subc_opt_with_aliases, "subc_opt"],
            ["lna", "lna", exts.tas.more_conflicts, cmd.subc_opt_with_aliases, "subc_opt_alias"],
            ["sna", "sn", exts.tas.more_conflicts, cmd.subc_sn_only, "n"],

            ["ln", "ln", exts.ec.subc_pl_aux_conflict, exts.tas.subc_pl_aux_conflict, "subc_pl_aux"],
            ["lna", "lna", exts.ec.subc_pl_aux_conflict, exts.tas.subc_pl_aux_conflict, "pl0"],
            ["sna", "sna", exts.ec.subc_pl_aux_conflict, cmd.subc_opt_with_aliases, "a"],
            ["sna", "sna", exts.ec.subc_pl_aux_conflict, cmd.subc_opt_with_aliases, "b"],
            ["iln", "iln", exts.ec.flag_opt, cmd.flag_opt],
            ["iln", "iln", exts.ec.more_conflicts, exts.tas.more_conflicts],
            ["lna", "ln", exts.ec.more_conflicts, cmd.subc_opt_with_aliases, "subc_opt"],
            ["sna", "sna", exts.ec.more_conflicts, exts.tas.more_conflicts, "d"],
            ["sna", "sn", exts.ec.more_conflicts, cmd.subc_sn_only, "n"],
        ]

        _cmd_str_ = "origen.eval"
        ext_conflicts[_cmd_str_]["conflicts_list"] = [
            ["iln", "iln", ext_conflicts[_cmd_str_]["exts"][0], ext_conflicts[_cmd_str_]["exts"][1]]
        ]

        _cmd_str_ = "origen.credentials.clear"
        cmd = cli.CLI.cmds.creds.clear
        exts = self.partition_exts(ext_conflicts[_cmd_str_]["exts"])
        ext_conflicts[_cmd_str_][n] = exts
        ext_conflicts[_cmd_str_]["conflicts_list"] = [
            ["iln", "ln", exts.tas.all, cmd.all],
            ["ln", "ln", exts.tas.cmd_conflicts_pl, cmd.all, "all"],
            ["sn", "sn", exts.tas.cmd_conflicts_pl, cmd.all, "a"],
            ["lna", "ln", exts.tas.cmd_conflicts_pl, cmd.datasets, "datasets"],
            ["sna", "sn", exts.tas.cmd_conflicts_pl, cmd.datasets, "d"],
            ["iln", "ln", exts.ec.all, cmd.all],
            ["ln", "ln", exts.ec.cmd_conflicts_aux, cmd.all, "all"],
            ["sn", "sn", exts.ec.cmd_conflicts_aux, cmd.all, "a"],
            ["lna", "ln", exts.ec.cmd_conflicts_aux, cmd.datasets, "datasets"],
            ["lna", "lna", exts.ec.cmd_conflicts_aux, exts.tas.cmd_conflicts_pl, "pl_datasets"],
            ["sna", "sn", exts.ec.cmd_conflicts_aux, cmd.datasets, "d"],
            ["sna", "sna", exts.ec.cmd_conflicts_aux, exts.tas.cmd_conflicts_pl, "e"],
        ]
