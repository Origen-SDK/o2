from test_apps_shared_test_helpers.cli.auditors import CmdNamespaceContainerAuditor
import pytest

class T_Plugin(CmdNamespaceContainerAuditor):
    container = CmdNamespaceContainerAuditor.global_cmds.pl
    container_help_offset = 0
    container_nspaces = [
        CmdNamespaceContainerAuditor.plugins.python_plugin,
        CmdNamespaceContainerAuditor.plugins.python_plugin_no_cmds,
        CmdNamespaceContainerAuditor.plugins.python_plugin_the_second
    ]
    nspace = CmdNamespaceContainerAuditor.plugins.python_plugin
    nspace_help_offset = 2
    empty_nspace = CmdNamespaceContainerAuditor.plugins.python_plugin_no_cmds
