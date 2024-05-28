from test_apps_shared_test_helpers.cli.auditors import CmdNamespaceContainerAuditor
from ..shared import CLICommon

class T_Plugin(CmdNamespaceContainerAuditor):
    container = CLICommon.in_app_cmds.pl
    container_help_offset = 0
    container_nspaces = CLICommon.loaded_plugins_alpha()
    nspace = CmdNamespaceContainerAuditor.plugins.python_plugin
    nspace_help_offset = 3
    empty_nspace = CmdNamespaceContainerAuditor.plugins.python_plugin_no_cmds
