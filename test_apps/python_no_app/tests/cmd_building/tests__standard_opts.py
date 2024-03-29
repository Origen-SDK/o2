from .shared import CLICommon

# Uses custom format output from ./cmd_building/cmd_testers/test_arguments/display_verbosity_opts.py
class T_StandardOpts(CLICommon):
    def test_empty_verbosity_is_accessible(self):
        out = self.cmd_testers.display_v.run()
        assert r'Args: {}' in out
        assert "verbosity: 0" in out

    def test_single_verbosity_is_accessible(self):
        out = self.cmd_testers.display_v.run("-v")
        assert r'Args: {}' in out
        assert "verbosity: 1" in out

    def test_stacked_verbosity_is_accessible(self):
        out = self.cmd_testers.display_v.run("-vv")
        assert r'Args: {}' in out
        assert "verbosity: 2" in out

    def test_stacked_split_verbosity_is_accessible(self):
        out = self.cmd_testers.display_v.run("-vv", "-v")
        assert r'Args: {}' in out
        assert "verbosity: 3" in out

    def test_empty_verbosity_keywords_are_accessible(self):
        out = self.cmd_testers.display_v.run()
        assert r'Args: {}' in out
        assert "keywords: []" in out

    def test_verbosity_keywords_are_accessible(self):
        out = self.cmd_testers.display_v.run("--vk", "t1,t2", "-v")
        assert r'Args: {}' in out
        assert "keywords: ['t1', 't2']" in out
        assert "verbosity: 1" in out

    def test_verbosity_help(self):
        help = self.cmd_testers.display_v.get_help_msg()
        help.assert_bare_opts()
