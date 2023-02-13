import pytest, pathlib

from tests.shared import PythonAppCommon
from test_apps_shared_test_helpers.cli import CLIShared, CmdOpt, CmdArg, CmdExtOpt

Cmd = CLIShared.Cmd

class CLICommon(CLIShared, PythonAppCommon):
    _no_config_run_opts = {
        "with_configs": CLIShared.configs.suppress_plugin_collecting_config,
        "bypass_config_lookup": True
    }

    @pytest.fixture
    def no_config_run_opts(self):
        return self._no_config_run_opts

    @classmethod
    def no_config_run_opts_plus_config(cls, add_configs):
        return {
            "with_configs": [
                CLIShared.configs.suppress_plugin_collecting_config,
                *([add_configs] if isinstance(add_configs, str) or isinstance(add_configs, pathlib.Path) else add_configs)
            ],
            "bypass_config_lookup": True
        }