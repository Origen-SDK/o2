import pytest, shutil, os
from test_apps_shared_test_helpers.cli import CLIShared

class CLICommon(CLIShared):
    @pytest.fixture
    def cmd(self):
        return self._cmd

    @pytest.fixture
    def with_cli_aux_cmds(self):
        shutil.copy(self.dummy_config, self.cli_config)
        shutil.copy(self.cli_aux_cmds_toml, self.cli_dir)
        dest_dir = self.cli_dir.joinpath("aux_cmds_from_cli_dir")
        if dest_dir.exists():
            shutil.rmtree(dest_dir)
        shutil.copytree(self.cli_aux_cmds_impl, dest_dir)
        yield
        os.remove(self.cli_config)
        os.remove(self.cli_dir.joinpath("aux_cmds_from_cli_dir.toml"))
        shutil.rmtree(dest_dir)

    @property
    def aux_cmd_ns_subcs(self):
        return [
            self.cmd_testers_cmd,
            self.aux.ns.empty_aux_ns.base_cmd,
            'help',
            self.aux.ns.python_no_app_aux_cmds.base_cmd
        ]
