import origen
from collections import UserDict
import importlib, os
from pathlib import Path
import _origen

def collect_plugins():
    pls = Plugins()
    for n, r in _origen.plugins.get_plugin_roots().items():
        pls.register(n)
    origen._plugins = pls
    return origen._plugins

def from_origen_cli(plugins):
    pls = Plugins()
    if plugins:
        for name in plugins.keys():
            pls.register(name)
    origen._plugins = pls
    return origen._plugins

class Plugin:
    pass

class Plugins(UserDict):
    def __init__(self):
        UserDict.__init__(self)

    @property
    def plugins(self):
        return self.data

    @property
    def names(self):
        return list(self.data.keys())

    def register(self, name):
        a = importlib.import_module(f'{name}.application')
        a_pl = type("Application", (Plugin, a.Application), {})
        app = a_pl(root=Path(os.path.abspath(
            a.__file__)).parent.parent,
                            name=name)
        self.data[name] = app
        return app

    # def load_from_config(self):
    #     for pl in _origen.config['plugins']['load']:
    #         self._load_pl()
