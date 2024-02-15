import re
from .dirs import rust_cli_toml
from origen.helpers.regressions.cli.origen import CoreErrorMessages as Errs
from origen_metal.utils.version import from_cargo

class AssertionHelpers:
    @classmethod
    def assert_v(cls, out, v_lvl, v_keywords=None):
        if v_lvl is not False:
            base = r"\[DEBUG\] \(..:..:..\....\): Logger Verbosity:"
            if v_lvl is None or v_lvl < 2:
                assert re.search(f"{base}", out) is None
            else:
                assert re.search(f"{base} {v_lvl}", out) is not None
        if v_keywords is not False:
            if isinstance(v_keywords, str):
                v_keywords = [v_keywords]
            base = r"\[DEBUG\] \(..:..:..\....\): Setting Verbosity Keywords:"
            if v_keywords is None:
                if v_lvl is None or v_lvl < 2:
                    assert re.search(f"{base}", out) is None
                else:
                    assert re.search(rf"{base} \[\]", out) is not None
            else:
                v_keywords = [f"\"{k}\"" for k in v_keywords]
                assert re.search(rf"{base} \[{', '.join(v_keywords)}\]", out) is not None

    assert_verbosity = assert_v

    @classmethod
    def assert_invalid_subc_msg(cls, out, subc):
        assert Errs.invalid_subc_msg(subc) in out

    @classmethod
    def assert_args_required_msg(cls, out, *missing_args):
        assert Errs.missing_required_arg(*missing_args) in out

    @classmethod
    def assert_missing_arg_msg(cls, out, arg):
        assert Errs.missing_required_arg(arg) in out

    @classmethod
    def assert_missing_ln_val_msg(cls, out, arg, value_name=None):
        assert Errs.missing_ln_val_msg(arg, value_name=value_name) in out

    @classmethod
    def assert_invalid_arg_msg(cls, out, arg_or_subc):
        assert Errs.unknown_arg_msg(arg_or_subc) in out

    @classmethod
    def assert_invalid_ln_msg(cls, out, offender):
        assert Errs.unknown_opt_msg(offender, True) in out

    @classmethod
    def assert_invalid_sn_msg(cls, out, offender):
        assert Errs.unknown_opt_msg(offender, False) in out

    @classmethod
    def assert_origen_v(cls, out, version=None, version_only=True, app=None, cli_version=None):
        import origen
        v = version or origen.__version__
        if app:
            if app is True:
                a = origen.app.version
            else:
                a = app
            s = "\n".join([
                f"App:    {a}",
                f"Origen: {v}",
                ''
            ])
        else:
            cli_ver = from_cargo(rust_cli_toml)

            s = "\n".join([
                f"Origen: {v}",
                f"CLI:    {cli_ver}",
                ''
            ])
        if version_only:
            assert s == out
        else:
            assert s in out

    @classmethod
    def assert_no_app_origen_v(cls, out, version=None, version_only=True):
        cls.assert_origen_v(out, version=version, app=False, version_only=version_only)

    @classmethod
    def assert_core_help(cls, out):
        help = cls.HelpMsg(out)
        assert help.root_cmd is True
        assert "Origen: 2." in help.version_str
        help.assert_bare_opts()

        # TODO check order?
        assert set(s["name"] for s in help.subcmds) == set(cls.global_cmds.all_names_add_help)
        assert help.app_cmd_shortcuts == None
        assert help.pl_cmd_shortcuts == {
            "do_actions": ("python_plugin", "do_actions"),
            "plugin_says_hi": ("python_plugin", "plugin_says_hi"),
            "echo": ("python_plugin", "echo"),
            "plugin_test_args": ("python_plugin", "plugin_test_args"),
            "plugin_test_ext_stacking": ("python_plugin", "plugin_test_ext_stacking"),
        }
        assert help.aux_cmd_shortcuts == {
            "python_no_app_tests": ("cmd_testers", "python_no_app_tests"),
            "test_nested_level_1": ("cmd_testers", "test_nested_level_1"),
            "test_arguments": ("cmd_testers", "test_arguments"),
            "error_cases": ("cmd_testers", "error_cases"),
            "say_hi": ("python_no_app_aux_cmds", "say_hi"),
            "say_bye": ("python_no_app_aux_cmds", "say_bye"),
        }
    
    @classmethod
    def assert_ext_non_ext_cmd_msg(self, out, target, offenders):
        offenders = "\n".join([f"\t{o.displayed}" for o in offenders])
        msg = f"Command '{target.full_name}' does not support extensions but an extension was attempted from:\n{offenders}"
        assert msg in out