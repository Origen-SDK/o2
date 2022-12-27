# FOR_PR decide if plugins should be implemented here or in pyapi
import origen
from collections import UserDict
import importlib, os
from pathlib import Path
import _origen

# class Plugin:
#     def __init__(self, name, root):
#         self.name = name
#         self.root = root

def collect_plugins():
    pls = Plugins()
    for n, r in _origen._plugin_roots().items():
        pls.register(n)
        # parts = l.split("|")
        # if len(parts) != 3:
        #     origen.log.error(f"Malformed output encountered when collecting plugin roots: {l}")
        #     continue

        # if not parts[0] == "success":
        #     origen.log_error(f"Unknown status when collecting plugin roots: {parts[0]}")
        # else:
        #     pls.register(parts[1])
    origen._plugins = pls
    return origen._plugins

def from_origen_cli(plugins):
    pls = Plugins()
    if plugins:
        for name in plugins.keys():
            pls.register(name)
    origen._plugins = pls
    return origen._plugins

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
        app = a.Application(root=Path(os.path.abspath(
            a.__file__)).parent.parent,
                            name=name)
        self.data[name] = app
        return app

    # def add_plugin(self, name, app):
    #     self._plugins[name] = app

    # def register_plugin()

    def collect(self):
        ...
    
    def load_from_config(self):
        for pl in _origen.config['plugins']['load']:
            self._load_pl()