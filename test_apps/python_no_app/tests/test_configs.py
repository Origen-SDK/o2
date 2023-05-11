import pytest, origen, shutil, os
from pathlib import Path
from origen.helpers.env import in_new_origen_proc, run_cli_cmd
from tests import configs as config_funcs

from test_apps_shared_test_helpers.cli import CLIShared, CmdOpt, CmdArg

class Common(CLIShared):
    # FOR_PR see how many of these are needed and what can be moved to test apps shared
    tests_root = Path(__file__).parent
    working_dir = Path(__file__).parent.parent
    working_dir_config = working_dir.joinpath("origen.toml")
    cli_dir = working_dir.joinpath("../../rust/origen/target/debug").resolve()
    cli_config = cli_dir.joinpath("origen.toml")

    configs_dir = Path(__file__).parent.joinpath("configs")
    dummy_config = configs_dir.joinpath("dummy_config.toml")
    dummy_configs_dir = configs_dir.joinpath("dummy_dir")
    dummy_origen_config = dummy_configs_dir.joinpath("origen.toml")
    empty_config = configs_dir.joinpath("empty.toml")

    python_plugin_and_2nd_only_config = configs_dir.joinpath("python_plugin_and_2nd_only.toml")

    aux_cmds_dir = Path(__file__).parent.joinpath("dummy_aux_cmds")
    cli_aux_cmds_toml = aux_cmds_dir.joinpath("aux_cmds_from_cli_dir.toml")
    cli_aux_cmds_impl = aux_cmds_dir.joinpath("aux_cmds_from_cli_dir")

    cmd_testers_root = tests_root.joinpath("cmd_building/cmd_testers")
    aux_cmd_configs_dir = configs_dir.joinpath("aux_cmds")

    python_plugin_config_dir_str = "../python_plugin/config"
    python_plugin_config_toml = Path(python_plugin_config_dir_str).joinpath("origen.toml")

    @pytest.fixture
    def existing_configs(self):
        return origen.__config_metadata__['files']

class TestConfig(Common):
    def test_local_config_is_added(self, existing_configs):
        assert self.working_dir_config in origen.__config_metadata__['files']

    @pytest.mark.skip
    def test_package_root_config_is_found(self):
        retn = in_new_origen_proc(mod=config_funcs)
        assert retn['files'] == []

    def test_config_from_cli_source_is_added(self, existing_configs):
        assert self.cli_config not in existing_configs
        shutil.copy(self.empty_config, self.cli_config)
        try:
            out = run_cli_cmd(["eval", "print( origen.__config_metadata__['files'] )"])
            print(out.split("\n")[-2])
            from pathlib import WindowsPath, PosixPath
            configs = eval(out.split("\n")[-2])
            # retn = in_new_origen_proc(mod=config_funcs)
        finally:
            os.remove(self.cli_config)

        # ensure the config at the CLI directory is removed
        assert self.cli_config in configs

    def test_config_dir_from_env_is_added(self, existing_configs):
        # Add directory
        retn = in_new_origen_proc(mod=config_funcs, func_kwargs={'config_dir': self.dummy_configs_dir}) #, with_configs=self.dummy_configs_dir)
        assert retn['files'] == [
            self.dummy_origen_config,
            *existing_configs,
        ]

    def test_direct_config_from_env_is_added(self, existing_configs):
        # Add direct toml source
        retn = in_new_origen_proc(mod=config_funcs, func_kwargs={'config_toml': self.dummy_config}) # , with_configs=self.dummy_origen_config)
        assert retn['files'] == [
            self.dummy_config,
            *existing_configs
        ]

    def test_multiple_configs_from_env_are_added(self, existing_configs):
        retn = in_new_origen_proc(mod=config_funcs, func_kwargs={'configs': [self.dummy_config, self.dummy_configs_dir]})
        assert retn['files'] == [
            self.dummy_config,
            self.dummy_origen_config,
            *existing_configs
        ]

    def test_relative_config_from_env_is_added(self, existing_configs):
        retn = in_new_origen_proc(mod=config_funcs, func_kwargs={'configs': self.python_plugin_config_dir_str}) #, with_configs="../python_no_app/tests/configs/")
        assert retn['files'] == [
            self.python_plugin_config_toml,
            *existing_configs
        ]

    @pytest.mark.xfail
    def test_error_on_non_toml_config_in_env(self, existing_configs):
        retn = in_new_origen_proc(mod=config_funcs, func_kwargs={'configs': self.dummy_configs_dir.parent.joinpath("__init__.py")})
        assert retn['files'] == [
            self.python_plugin_config_toml,
            *existing_configs
        ]

    @pytest.mark.skip
    def test_error_on_missing_config_dir_in_env(self):
        fail

    @pytest.mark.skip
    def test_error_on_missing_toml_config_in_env(self):
        fail

    @pytest.mark.skip
    def test_config_locations_can_stack(self):
        fail
    
    @pytest.mark.xfail
    def test_bypassing_config_lookups(self):
        # No configs
        retn = in_new_origen_proc(mod=config_funcs, bypass_config_lookup=True)
        assert retn['files'] == []

        # Configs from the env are added though
        retn = in_new_origen_proc(func=config_funcs.test_bypassing_config_lookup_with_env, bypass_config_lookup=True, with_config=[...])
        assert retn['files'] == []

    @pytest.mark.skip
    def test_user_config_is_added(self):
        fail

    @pytest.mark.skip
    def test_default_config_lookup_can_be_suppressed(self):
        fail
