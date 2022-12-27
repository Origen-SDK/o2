# FOR_PR add T_EVAL
from .cli.tests__global_cmds import T_GlobalCmds
from .cli.tests__cmd__credentials import T_Credentials
# from .cli.tests__cmd__eval import T_Eval

class TestGlobalCmds(T_GlobalCmds):
    pass

class TestCredentials(T_Credentials):
    pass

# class TestEval(T_Credentials):
#     pass
