from .cli.tests__global_cmds import T_GlobalCmds
from .cli.tests__cmd__aux_cmds import T_AuxCmds
from .cli.tests__cmd__credentials import T_Credentials
from .cli.tests__cmd__eval import T_Eval
from .cli.tests__cmd__exec import T_Exec
from .cli.tests__cmd__plugin import T_Plugin
from .cli.tests__cmd__plugins import T_Plugins
from .cli.tests__origen_v import T_OrigenVersion
from .cli.tests__origen_help import T_OrigenHelp
from .cli.tests__invocation_errors import T_InvocationErrors

class TestGlobalCmds(T_GlobalCmds):
    pass

class TestOrigenHelp(T_OrigenHelp):
    pass

class TestAuxCmds(T_AuxCmds):
    pass

class TestCredentials(T_Credentials):
    pass

class TestEval(T_Eval):
    pass

class TestExec(T_Exec):
    pass

class TestPlugin(T_Plugin):
    pass

class TestPlugins(T_Plugins):
    pass

class TestOrigenVersion(T_OrigenVersion):
    pass

class TestInvocationErrors(T_InvocationErrors):
    pass