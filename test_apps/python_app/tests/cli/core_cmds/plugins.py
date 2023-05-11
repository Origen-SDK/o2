from test_apps_shared_test_helpers.cli.auditors import PluginsCmdAudit
from ..shared import CLICommon

class T_Plugins(PluginsCmdAudit):
    loaded_plugins = CLICommon.loaded_plugins()
