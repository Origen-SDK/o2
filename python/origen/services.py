import origen
import _origen
JTAG = _origen.services.JTAG
import pdb


# This defines the methods for defining sub-blocks in Python and then handles serializing
# the definitions and handing them over to the Rust model for instantiation.
class Loader:
    def __init__(self, controller):
        self.controller = controller

    def service(self, name, obj):
        func = getattr(obj, "set_model", None)
        if callable(func):
            obj = func(name, self.controller.model())
        func = getattr(obj, "set_controller", None)
        if callable(func):
            obj = func(name, self.controller)
        self.controller.services[name] = obj
        return obj

    # Defines the methods that are accessible within blocks/<block>/services.py
    def api(self):
        return {
            "Service": self.service,
        }
