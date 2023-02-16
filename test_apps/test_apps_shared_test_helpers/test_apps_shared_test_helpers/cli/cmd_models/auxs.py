from pathlib import Path
from origen.helpers.regressions import cli
from . import Cmd, CmdArg, CmdOpt

aux_cmds_dir = Path(__file__).parent.parent.parent.joinpath("aux_cmds")

class AuxCmdsFromCliDir(cli.CLI):
    Cmd = Cmd
    
    def __init__(self):
        self.name = "aux_cmds_from_cli_dir"
        self.aux_cmds_from_cli_dir = self.aux_cmd(
            self.name,
            help="Aux Commands from the Origen CLI directory",
            subcmds = [
                Cmd("cli_dir_says_hi")
            ],
        )

    @property
    def base_cmd(self):
        return self.aux_cmds_from_cli_dir

class AddAuxCmds(cli.CLI):
    Cmd = Cmd
    
    def __init__(self):
        self.name = "add_aux_cmd"
        self.add_aux_cmd = self.aux_cmd(
            self.name,
            help=None,
        )

    @property
    def base_cmd(self):
        return self.add_aux_cmd

class CmdTesters(cli.CLI):
    Cmd = Cmd

    def __init__(self):
        self.name = "cmd_testers"
        self.cmd_testers = self.aux_cmd(
            self.name,
            help="Commands to assist in testing aux commands when no app is present",
            subcmds=[
                Cmd(
                    "error_cases",
                    help="Commands to test error messages and improper command configuration",
                    subcmds=[
                        Cmd(
                            "missing_impl_dir",
                            subcmds=[
                                Cmd("missing_impl_dir_subc")
                            ]
                        ),
                        Cmd("missing_impl_file"),
                        Cmd("test_missing_run_function"),
                        Cmd("test_exception_in_run"),
                    ]
                ),
                Cmd("python_no_app_tests", help="Test commands for python-no-app workspace"),
                Cmd(
                    "test_arguments",
                    help="Test various argument and option schemes from commands",
                    subcmds=[
                        Cmd("display_verbosity_opts"),
                        Cmd(
                            "no_args_or_opts",
                            help="Command taking no arguments or options"
                        ),
                        Cmd(
                            "optional_arg",
                            help="Command taking a single, optional argument",
                            args=[CmdArg("single_val", "Single value")],
                        ),
                        Cmd(
                            "required_arg",
                            help="Command taking a required and optional arg",
                            args=[
                                CmdArg("required_val", "Single required value", required=True),
                                CmdArg("optional_val", "Single optional value")
                            ],
                        ),
                        Cmd(
                            "multi_arg",
                            help="Command taking a multi-arg",
                            args=[
                                CmdArg("multi_arg", "Multi-arg value", True)
                            ],
                        ),
                        Cmd(
                            "delim_multi_arg",
                            help="Command taking a delimited multi-arg",
                            args=[
                                CmdArg("delim_m_arg", "Delimited Multi-arg value ('multiple' implied)", True)
                            ],
                        ),
                        Cmd(
                            "single_and_multi_arg",
                            help="Command taking a single and multi-arg",
                            args=[
                                CmdArg("single_val", "Single value"),
                                CmdArg("multi_arg", "Multi-arg value", True)
                            ],
                        ),
                        Cmd(
                            "args_with_value_names",
                            help="Single and multi arg with value custom value names",
                            args=[
                                CmdArg("s_arg", "Single value arg with custom value name", value_name="Single Arg Val"),
                                CmdArg("m_arg", "Multi value arg with custom value name", True, value_name="Multi Arg Val")
                            ],
                        ),
                        Cmd(
                            "single_value_optional_opt",
                            help="Command taking optional, single option",
                            opts=[
                                CmdOpt(
                                    name="implicit_single_val",
                                    help='Implicit non-required single value',
                                    takes_value=True,
                                    required=False,
                                ),
                                CmdOpt(
                                    name="explicit_single_val",
                                    help='Explicit non-required single value',
                                    takes_value=True,
                                    required=False,
                                ),
                            ]
                        ),
                        Cmd(
                            "single_value_required_opt",
                            help="Command with single-value optional and required options",
                            opts=[
                                CmdOpt(
                                    name="non_req_val",
                                    help="Non-required single value",
                                    takes_value=True,
                                ),
                                CmdOpt(
                                    name="req_val",
                                    help="Required single value",
                                    takes_value=True,
                                    required=True,
                                ),
                            ]
                        ),
                        Cmd(
                            "multi_opts",
                            help="Command with multi-value optional and required options",
                            opts=[
                                CmdOpt(
                                    name="m_opt",
                                    help="Opt with multiple values",
                                    multi=True,
                                ),
                                CmdOpt(
                                    name="im_m_opt",
                                    help="Opt accepting multiple values were 'takes value' is implied",
                                    multi=True,
                                ),
                                CmdOpt(
                                    name="req_m_opt",
                                    help="Required opt accepting multiple values",
                                    multi=True,
                                    required=True,
                                ),
                                CmdOpt(
                                    name="d_m_opt",
                                    help="Delimited multi opt",
                                    multi=True,
                                ),
                                CmdOpt(
                                    name="d_im_m_opt",
                                    help="Delimited opt where 'multi' and 'takes value' is implied",
                                    multi=True,
                                ),
                            ]
                        ),
                        Cmd(
                            "flag_opts",
                            help="Command with flag-style options only",
                            opts=[
                                CmdOpt(
                                    name="im_f_opt",
                                    help="Stackable flag opt with 'takes value=false' implied",
                                ),
                                CmdOpt(
                                    name="ex_f_opt",
                                    help="Stackable flag opt with 'takes value=false' set",
                                ),
                            ]
                        ),
                        Cmd(
                            "opts_with_value_names",
                            help="Command with single/multi-opts with custom value names",
                            opts=[
                                CmdOpt(
                                    name="s_opt_nv_im_tv",
                                    help="Single opt with value name, implying 'takes_value'=true",
                                    value_name="s_val_impl",
                                ),
                                CmdOpt(
                                    name="s_opt_nv_ex_tv",
                                    help="Single opt with value name and explicit 'takes_value'=true",
                                    value_name="s_val_expl",
                                    takes_value=True,
                                ),
                                CmdOpt(
                                    name="m_opt_named_val",
                                    help="Multi-opt with value name",
                                    value_name="m_val",
                                    multi=True,
                                ),
                                CmdOpt(
                                    name="s_opt_ln_nv",
                                    help="Single opt with long name and value name",
                                    value_name="ln_nv",
                                ),
                            ]
                        ),
                        Cmd(
                            "opts_with_aliases",
                            help="Command with option aliasing, custom long, and short names",
                            opts=[
                                CmdOpt(
                                    name="single_opt",
                                    help="Single opt with long/short name",
                                    takes_value=True,
                                    ln="s_opt",
                                    sn="s"
                                ),
                                CmdOpt(
                                    name="multi_opt",
                                    help="Multi-opt with long/short name",
                                    takes_value=True,
                                    multi=True,
                                    ln="m_opt",
                                    sn="m"
                                ),
                                CmdOpt(
                                    name="occurrence_counter",
                                    help="Flag opt with long/short name",
                                    ln="cnt",
                                    sn="o",
                                ),
                                CmdOpt(
                                    name="flag_opt_short_name",
                                    help="Flag opt with short name only",
                                    sn="f"
                                ),
                                CmdOpt(
                                    name="flag_opt_long_name",
                                    help="Flag opt with long name only",
                                    ln="ln_f_opt"
                                ),
                                CmdOpt(
                                    name="flag_opt_dupl_ln_sn",
                                    help="Flag opt with ln matching another's sn",
                                    ln="f"
                                ),
                                CmdOpt(
                                    name="fo_sn_aliases",
                                    help="Flag opt with short aliases",
                                    sn_aliases=['a', 'b']
                                ),
                                CmdOpt(
                                    name="fo_sn_and_aliases",
                                    help="Flag opt with short name and short aliases",
                                    sn="c",
                                    sn_aliases=['d', 'e']
                                ),
                                CmdOpt(
                                    name="fo_ln_aliases",
                                    help="Flag opt with long aliases",
                                    ln_aliases=['fa', 'fb']
                                ),
                                CmdOpt(
                                    name="fo_ln_and_aliases",
                                    help="Flag opt with long name and long aliases",
                                    ln="fc",
                                    ln_aliases=['fd', 'fe']
                                ),
                                CmdOpt(
                                    name="fo_sn_ln_aliases",
                                    help="Flag opt with long and short aliases",
                                    ln_aliases=['sn_ln_1', 'sn_ln_2'],
                                    sn_aliases=['z'],
                                ),
                            ]
                        ),
                        Cmd(
                            "hidden_opt",
                            help="Command with a hidden opt",
                            opts=[
                                CmdOpt(
                                    name="hidden_opt",
                                    help="Hidden opt",
                                    hidden=True,
                                ),
                                CmdOpt(
                                    # name="non_hidden_opt",
                                    name="visible_opt",
                                    help="Visible, non-hidden, opt",
                                ),
                            ]
                        ),
                    ]
                ),
                Cmd("test_current_command", help="Tests origen.current_command"),
                Cmd(
                    "test_nested_level_1",
                    help="Tests origen.current_command L1",
                    subcmds=[
                        Cmd(
                            "test_nested_level_2",
                            help="Tests origen.current_command L2",
                            subcmds=[
                                Cmd("test_nested_level_3_a", help="Tests origen.current_command L3a"),
                                Cmd("test_nested_level_3_b", help="Tests origen.current_command L3b"),
                            ]
                        )
                    ]
                ),
            ]
        )

    @property
    def base_cmd(self):
        return self.cmd_testers

    @property
    def test_args(self):
        return self.base_cmd.test_arguments

    @property
    def display_v(self):
        return self.test_args.display_verbosity_opts

    @property
    def error_cases(self):
        return self.base_cmd.error_cases

    @property
    def subc_l1(self):
        return self.base_cmd.test_nested_level_1

    @property
    def subc_l2(self):
        return self.subc_l1.test_nested_level_2

    @property
    def subc_l3_a(self):
        return self.subc_l2.test_nested_level_3_a

    @property
    def subc_l3_b(self):
        return self.subc_l2.test_nested_level_3_b


class DummyCmds(cli.CLI):
    Cmd = Cmd
    cfg_toml = aux_cmds_dir.joinpath("dummy_cmds_cfg.toml")

    def __init__(self):
        self.name = "dummy_cmds"
        self.dummy_cmd = self.aux_sub_cmd(
            self.name,
            "dummy_cmd",
            help="Dummy Aux Command",
            args=[
                CmdArg(
                    name="action_arg",
                    help="Dummy Aux Action",
                    multi=True,
                ),
            ],
            subcmds=[
                Cmd(
                    "subc",
                    help="Dummy Aux Subcommand",
                    args=[
                        CmdArg(
                            name="action_arg",
                            help="Dummy Aux Subc Action",
                            multi=True,
                        ),
                    ],
                    opts=[
                        CmdOpt(
                            name="flag_opt",
                            help="Dummy Aux Subc Flag",
                        ),
                    ],
                )
            ],
            from_config=self.cfg_toml
        )

class PythonNoAppAuxCmds(cli.CLI):
    Cmd = Cmd

    def __init__(self):
        self.name = "python_no_app_aux_cmds"
        self.python_no_app_aux_cmds = self.aux_sub_cmd(
            self.name,
            "python_no_app_aux_cmds"
        )
    
    @property
    def base_cmd(self):
        return self.python_no_app_aux_cmds

class PythonAppAuxCmds(cli.CLI):
    Cmd = Cmd

    def __init__(self):
        self.name = "python_app_aux_cmds"
        self.config_toml = aux_cmds_dir.joinpath(f"{self.name}_cfg.toml")
        self.disabling_app_opts_from_aux = self.aux_sub_cmd(
            self.name,
            "disabling_app_opts_from_aux",
            help="Test disabling standard app opts from plugin commands",
            from_config=self.config_toml,
            subcmds=[
                Cmd(
                    "disable_targets_opt",
                    help="Disable the targets and no-targets opt",
                    subcmds=[
                        Cmd("disable_subc", help="Disables inherited from parent"),
                        Cmd("override_subc", help="Overrides disable inherited from parent"),
                    ]
                ),
                Cmd(
                    "disable_mode_opt",
                    help="Disable the mode opt",
                    subcmds=[
                        Cmd("disable_subc",help="Disables inherited from parent"),
                        Cmd("override_subc", help="Overrides disable inherited from parent"),
                    ]
                ),
                Cmd(
                    "disable_app_opts",
                    help="Disable all app opts",
                    subcmds=[
                        Cmd("disable_subc",help="Disables inherited from parent"),
                        Cmd("override_subc", help="Overrides disable inherited from parent"),
                    ]
                )
            ]
        )

    @property
    def base_cmd(self):
        return self.python_app_aux_cmds

class AuxNamespaces:
    def __init__(self) -> None:
        self.dummy_cmds = DummyCmds()
        self.cmd_testers = CmdTesters()
        self.python_no_app_aux_cmds = PythonNoAppAuxCmds()
        self.python_app_aux_cmds = PythonAppAuxCmds()
        self.aux_cmds_from_cli_dir = AuxCmdsFromCliDir()
        self.add_aux_cmd = AddAuxCmds()

class Aux:
    namespaces = AuxNamespaces()
    ns = namespaces
