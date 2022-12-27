import pytest
from .cmd_building.tests__standard_opts import T_StandardOpts
from .cmd_building.tests__arg_buildling import T_ArgBuilding
from .cmd_building.tests__opt_building import T_OptBuilding
from .cmd_building.tests__loading_aux_cmds import T_LoadingAuxCommands
from .cmd_building.tests__loading_plugin_cmds import T_LoadingPluginCmds

class TestLoadingAuxCommands(T_LoadingAuxCommands):
    pass

class TestStandardOpts(T_StandardOpts):
    pass

class TestArgBuilding(T_ArgBuilding):
    pass

class TestOptBuilding(T_OptBuilding):
    pass

class TestLoadingPluginCmds(T_LoadingPluginCmds):
    pass