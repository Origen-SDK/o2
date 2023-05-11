from test_apps_shared_test_helpers.cli.auditors import PluginsCmdAudit

class T_Plugins(PluginsCmdAudit):
    loaded_plugins = [
        PluginsCmdAudit.plugins.python_plugin,
        PluginsCmdAudit.plugins.python_plugin_the_second,
        PluginsCmdAudit.plugins.python_plugin_no_cmds,
    ]
