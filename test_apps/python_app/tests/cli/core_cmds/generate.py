from ..shared import CLICommon
import re, origen
from pathlib import Path

class T_Generate(CLICommon):
    _cmd = CLICommon.in_app_cmds.generate
    pat = "example/patterns/toggle.py"
    toggle_out = Path("output/j750/toggle.atp")
    custom_dir = CLICommon.tmp_dir()
    toggle_out_custom = custom_dir.joinpath("j750/toggle.atp")
    targets = ["eagle", "j750"]

    def setup_class(cls):
        if cls.toggle_out.exists():
            cls.toggle_out.unlink()
        if cls.toggle_out_custom.exists():
            cls.toggle_out_custom.unlink()

    def test_help_msg(self, cmd, cached_help):
        cached_help.assert_cmd(cmd)

    def test_error_on_no_args(self, cmd):
        out = cmd.gen_error()
        print(out)
        exp = self.err_msgs.missing_required_arg(cmd.files)
        print(exp)
        assert exp in out

    def test_generate_pattern(self, cmd):
        assert not self.toggle_out.exists()
        out = cmd.run(self.pat, run_opts={"targets": self.targets})
        assert re.match(rf"Created: {self.toggle_out} - .*New pattern", out)
        assert self.toggle_out.exists()

    def test_output_dir(self, cmd):
        assert not self.toggle_out_custom.exists()
        out = cmd.run(self.pat, cmd.output_dir.sn_to_cli(), str(self.custom_dir), run_opts={"targets": self.targets})
        assert re.match(rf"Created: {self.toggle_out_custom.relative_to(origen.app.root)} - .*New pattern", out)
        assert self.toggle_out_custom.exists()
