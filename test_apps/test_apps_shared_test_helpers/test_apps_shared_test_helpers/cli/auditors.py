import pytest
from abc import ABC, abstractclassmethod
from . import CLIShared

class CLIAudit(CLIShared):
    @pytest.fixture
    def cmd(self):
        return self._cmd

class CmdNamespaceAuditor(CLIAudit, ABC):
    @property
    @abstractclassmethod
    def nspace(self):
        raise NotImplemented

    @property
    @abstractclassmethod
    def nspace_help_offset(cls):
        raise NotImplemented

    @property
    @abstractclassmethod
    def empty_nspace(cls):
        raise NotImplemented

    @property
    def nspace_subcmds(self):
        cmds = list(self.nspace.base_cmd.subcmds.values())
        cmds.insert(self.nspace_help_offset, "help")
        return cmds

    @pytest.fixture
    def cached_no_subc_help(self):
        if not hasattr(self, "_cached_no_subc_help"):
            self._cached_no_subc_help = self.nspace.base_cmd.get_help_msg()
        return self._cached_no_subc_help

    def test_nspace_help_msg(self, cached_no_subc_help):
        help = cached_no_subc_help
        help.assert_args(None)
        help.assert_bare_opts()
        help.assert_subcmds(*self.nspace_subcmds)
        help.assert_not_extendable()

    def test_nspace_help_with_no_subcmds_given(self, cached_no_subc_help):
        out = self.nspace.base_cmd.gen_error()
        assert out == cached_no_subc_help.text

    def test_nspace_without_subcmds(self):
        cmd = self.empty_nspace.base_cmd
        help = cmd.get_help_msg()
        help.assert_args(None)
        help.assert_bare_opts()
        help.assert_subcmds(None)
        help.assert_not_extendable()

class CmdNamespaceContainerAuditor(CmdNamespaceAuditor, ABC):
    @property
    @abstractclassmethod
    def container(self):
        raise NotImplemented

    @property
    @abstractclassmethod
    def container_help_offset(self):
        raise NotImplemented

    @abstractclassmethod
    def container_nspaces(self):
        raise NotImplemented

    @property
    def container_subcmds(self):
        l = [n.base_cmd for n in self.container_nspaces]
        l.insert(self.container_help_offset, "help")
        return l

    @classmethod
    def setup_class(cls):
        cls._cmd = cls.container

    def test_help_msg_with_cmds(self, cmd, cached_help):
        help = cached_help
        help.assert_args(None)
        help.assert_bare_opts()
        help.assert_subcmds(*self.container_subcmds)
        help.assert_not_extendable()
        help.assert_summary(cmd.help)

    def test_help_with_no_subcmds_given(self, cmd, cached_help):
        out = cmd.gen_error()
        assert out == cached_help.text

    def test_no_cmds_present(self, cmd):
        cmd = self.add_no_pl_aux_cfg(cmd)
        help = cmd.get_help_msg()
        help.assert_args(None)
        help.assert_bare_opts()
        help.assert_subcmds(None)
        help.assert_not_extendable()

class PluginsCmdAudit(CLIShared, ABC):
    import origen
    if origen.app:
        cmd = CLIShared.global_cmds.pls
    else:
        cmd = CLIShared.in_app_cmds.pls

    @property
    @abstractclassmethod
    def loaded_plugins(cls):
        raise NotImplementedError

    check_pl_list_order = True

    class TestBaseCmd(CLIShared):
        def test_help_msg(self, cmd, cached_help):
            help = cached_help
            help.assert_args(None)
            help.assert_bare_opts()
            help.assert_subcmds("help", cmd.list)
            help.assert_not_extendable()
            help.assert_summary(cmd.help)

        def test_help_on_no_subc(self, cmd, cached_help):
            assert cmd.gen_error() == cached_help.text

    class TestListSubc(CLIShared):
        def assert_out(self, out, plugins):
            if plugins is None:
                assert out == "There are no available plugins!\n"
            else:
                if self.check_pl_list_order:
                    pls = "\n".join([pl.name for pl in plugins])
                    assert out == f"Available plugins:\n\n{pls}\n"
                else:
                    assert out.startswith(f"Available plugins:\n\n")
                    out = out.split("\n")
                    print(out)
                    assert len(out) == 3 + len(plugins)
                    for pl in plugins:
                        assert pl.name in out

        def test_help_msg(self, cmd):
            help = cmd.get_help_msg()
            help.assert_summary(cmd.help)
            help.assert_args(None)
            help.assert_bare_opts()
            help.assert_subcmds(None)
            help.assert_not_extendable()

        def test_listing_plugins(self, cmd):
            out = cmd.run()
            self.assert_out(
                out,
                self.loaded_plugins
            )

        def test_listing_with_no_plugins(self, cmd):
            out = self.add_no_pl_aux_cfg(cmd).run()
            self.assert_out(out, None)

    def setup_class(cls):
        c = cls.TestBaseCmd
        c._cmd = cls.cmd
        c.loaded_plugins = cls.loaded_plugins

        c = cls.TestListSubc
        c._cmd = cls.cmd.list
        c.loaded_plugins = cls.loaded_plugins
        c.check_pl_list_order = cls.check_pl_list_order
