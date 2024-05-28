from test_apps_shared_test_helpers.cli.auditors import CmdNamespaceContainerAuditor
import pytest

class T_AuxCmds(CmdNamespaceContainerAuditor):
    container = CmdNamespaceContainerAuditor.global_cmds.aux_cmds
    container_help_offset = 2
    container_nspaces = [
        CmdNamespaceContainerAuditor.aux.ns.cmd_testers,
        CmdNamespaceContainerAuditor.aux.ns.empty_aux_ns,
        CmdNamespaceContainerAuditor.aux.ns.python_no_app_aux_cmds
    ]
    nspace = CmdNamespaceContainerAuditor.aux.ns.python_no_app_aux_cmds
    nspace_help_offset = 0
    empty_nspace = CmdNamespaceContainerAuditor.aux.ns.empty_aux_ns
