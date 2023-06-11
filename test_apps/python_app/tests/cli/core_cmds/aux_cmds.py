from test_apps_shared_test_helpers.cli.auditors import CmdNamespaceContainerAuditor
from ..shared import CLICommon

class T_AuxCmds(CmdNamespaceContainerAuditor):
    container = CmdNamespaceContainerAuditor.in_app_cmds.aux_cmds
    container_help_offset = 2
    container_nspaces = [
        CmdNamespaceContainerAuditor.aux.ns.dummy_cmds,
        CmdNamespaceContainerAuditor.aux.ns.empty_aux_ns,
    ]
    nspace = CmdNamespaceContainerAuditor.aux.ns.dummy_cmds
    nspace_help_offset = 1
    empty_nspace = CmdNamespaceContainerAuditor.aux.ns.empty_aux_ns
