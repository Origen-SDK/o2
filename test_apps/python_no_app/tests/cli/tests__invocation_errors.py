from test_apps_shared_test_helpers.cli import CLIShared, CmdOpt, CmdArg
import pytest

class T_InvocationErrors(CLIShared):
    def test_with_unknown_cmds(self):
        out = self.gen_error(["blah_cmd"], return_details=True)
        self.assert_v(out["stdout"], None)
        self.assert_invalid_arg_msg(out["stderr"], "blah_cmd")

        out = self.gen_error(["--vk", "vk0", "blah_cmd", "-vv"], return_details=True)
        self.assert_v(out["stdout"], 2, ["vk0"])
        self.assert_invalid_arg_msg(out["stderr"], "blah_cmd")

    def test_with_unknown_options(self):
        out = self.gen_error(["--blah_opt"], return_details=True)
        self.assert_v(out["stdout"], None)
        self.assert_invalid_ln_msg(out["stderr"], "blah_opt")

        out = self.gen_error(["--verbose", "--vk", "vk_opt_1", "--blah_opt", "--verbose", "--vk", "vk_opt_2,vk_opt_3"], return_details=True)
        self.assert_v(out["stdout"], 2, ["vk_opt_1", "vk_opt_2", "vk_opt_3"])
        self.assert_invalid_ln_msg(out["stderr"], "blah_opt")

    inv_subc = "invalid_subc"
    invalid_help_subc_cases = [
        ("help_pre_subc", "invalid_subc", ["help", inv_subc], None, None),
        ("help_post_subc", "invalid_arg", [inv_subc, "help"], None, None),
        ("help_sn_pre", "invalid_arg", [inv_subc, "-h", "-vv"], 2, None),
        ("help_ln", "invalid_arg", [inv_subc, "--help", "-vv", "--vk", "vk_ln"], 2, "vk_ln"),

        # This will parse "-h" first, meaning this will actually display help message, not an error
        ("help_sn_post", None, ["-h", "-vv", inv_subc, "--vk", "vk0,vk1"], 2, ["vk0", "vk1"]),
    ]
    @pytest.mark.parametrize("assert_method,args,vlvl,vks", [h[1:] for h in invalid_help_subc_cases], ids=[h[0] for h in invalid_help_subc_cases])
    def test_help_with_unknown_subcs(self, assert_method, args, vlvl, vks):
        if assert_method is None:
            out = self.run_cli_cmd(args)
            self.assert_core_help(out)
        else:
            out = self.gen_error(args, return_details=True)
            getattr(self, f"assert_{assert_method}_msg")(out["stderr"], self.inv_subc)
            out = out["stdout"]
        self.assert_v(out, vlvl, vks)

    misuses = [
        ("eval_without_files", "missing_arg", CLIShared.global_cmds.eval.code, ["-vv", "eval", "--vk", "vk_g"], 2, "vk_g"),
        ("eval_with_invalid_opt", "invalid_ln", "blah_opt", ["eval", "\"print( 'hi' )\"", "--blah_opt", "-vv", "--vk", "vk_opt"], 2, "vk_opt"),
        ("eval_with_invalid_opt_pre_cmd", "invalid_ln", "blah_opt", ["--blah_opt", "-vv", "--vk", "vk_opt", "eval", "\"print( 'hi' )\"", "-v"], 3, "vk_opt"),
        ("missing_vk_val", "missing_ln_val", CLIShared.opts.vk, ["eval", "\"print( 'hi' )\"", "--vk", "-vv"], None, None),
        ("missing_vk_val", "missing_ln_val", CLIShared.opts.vk, ["-h", "--vk", "-vv"], None, None),

        # Help subc will view anything after as subcommand
        # e.g.: "help -h" will try to "help" the subc "-h", which won't exists
        ("help_subc_and_sn", "invalid_subc", "-h", ["help", "-h"], None, None),
    ]
    @pytest.mark.parametrize("assert_method,offender,args,vlvl,vks", [h[1:] for h in misuses], ids=[h[0] for h in misuses])
    def test_command_misuse_with_verbosity(self, assert_method, offender, args, vlvl, vks):
        out = self.gen_error(args, return_details=True)
        self.assert_v(out["stdout"], vlvl, vks)
        getattr(self, f"assert_{assert_method}_msg")(out["stderr"], offender)
