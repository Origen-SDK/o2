import pytest, origen, _origen
from origen.helpers.env import in_new_origen_proc, run_cli_cmd
from tests import configs as config_funcs
from tests.test_configs import Common as ConfigCommon
from pathlib import WindowsPath, PosixPath

class TestLoadingGlobalPlugins(ConfigCommon):
    def get_configs_and_plugins_from_cli(cls, configs=None, bypass_config_lookup=False):
        header = 'Configs and Plugin Names'
        out = run_cli_cmd(
            ["eval", f"print( '{header}' ); print( origen.__config_metadata__['files'] ); print( origen.plugins.names )"],
            bypass_config_lookup=bypass_config_lookup,
            with_configs=configs or None
        )
        out = out.split("\n")[0:-1]
        print(out)
        i = out.index(header)
        configs = eval(out[i+1].strip())
        plugins = eval(out[i+2].strip())
        return {
            'configs': configs,
            'plugins': plugins,
        }

    def test_plugins_are_collected_by_default(self):
        retn = in_new_origen_proc(mod=config_funcs)
        assert retn['configs'] == []
        # TODO consistent plugin loading
        assert set(retn['plugins']) == set(self.plugins.python_no_app_collected_pl_names)

        # Test from CLI
        retn = self.get_configs_and_plugins_from_cli(bypass_config_lookup=True)
        assert retn['configs'] == []
        # TODO consistent plugin loading
        assert set(retn['plugins']) == set(self.plugins.python_no_app_collected_pl_names)

    def test_plugins_are_accessible(self):
        pls = origen.plugins
        # TODO
        # assert isinstance(pls, _origen.plugins.Plugins)
        assert isinstance(pls, origen.core.plugins.Plugins)
        pl = pls[self.python_plugin.name]
        assert pls.names == self.plugins.python_no_app_config_pl_names
        # TODO
        assert isinstance(pl, origen.application.Base)
        # assert isinstance(pl, _origen.plugins.Plugin)
        # TODO
        assert pl.is_plugin == True

    # TODO needed?
    @pytest.mark.skip
    def test_registering_global_plugins(self):
        assert origen.plugins.names == self.plugins.python_no_app_config_pl_names
        # retn = in_new_origen_proc(mod=config_funcs, func_kwargs={'configs': self.dummy_configs_dir.joinpath("__init__.py")})
        # assert retn['files'] == [
        #     self.python_plugin_config_toml,
        #     *existing_configs
        # ]

    def test_suppressing_plugin_collection(self):
        c = self.configs.suppress_plugin_collecting_config
        retn = in_new_origen_proc(
            mod=config_funcs,
            func_kwargs={'configs': c}
        )
        assert retn['configs'] == [c]
        assert retn['plugins'] == []

        # Try from CLI
        retn = self.get_configs_and_plugins_from_cli(configs=c, bypass_config_lookup=True)
        assert retn['configs'] == [c]
        assert retn['plugins'] == []

    def test_enumerating_plugins_to_load(self):
        c = self.python_plugin_and_2nd_only_config
        exp_pls = [self.plugins.python_plugin.name, self.plugins.python_plugin_the_second.name]
        retn = in_new_origen_proc(
            mod=config_funcs,
            func_kwargs={'configs': c}
        )
        assert retn['configs'] == [c]
        assert retn['plugins'] == exp_pls

        # Try from CLI
        retn = self.get_configs_and_plugins_from_cli(configs=c, bypass_config_lookup=True)
        assert retn['configs'] == [c]
        assert retn['plugins'] == exp_pls

    @pytest.mark.skip
    def test_error_on_missing_plugin(self):
        fail

    @pytest.mark.skip
    def test_error_on_loading_plugins(self):
        fail
