import importlib
import origen;
import _origen;

# The base class of all application instances
class Base:
    config = _origen.app_config()
    current_target = _origen.app.current_target()

    def instantiate_block(self, path):
        path = '.derivatives.'.join(path.split("."))
        path = self.config["name"] + ".blocks." + path + ".model"
        m = importlib.import_module(path)
        block = m.Model()
        origen.dut = block
        return block

    #def __repr__(self):
    #    return "<an app>"
