import origen
import _origen
import importlib

# The base class of all application classes
class Base:
    config = _origen.app_config()

    def instantiate_block(self, path):
        path = '.derivatives.'.join(path.split("."))
        path = self.config["name"] + ".blocks." + path + ".model"
        m = importlib.import_module(path)
        block = m.Model()
        return block

    #def __repr__(self):
    #    return "<an app>"

# Load the application
def load():
    a = importlib.import_module(f'{_origen.app_config()["name"]}.application')
    app = a.Application()
    origen.app = app

