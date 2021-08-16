import origen
import _origen
JTAG = _origen.services.JTAG
SWD = _origen.services.SWD
Simple = _origen.services.Simple


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


class Base:
    def set_controller(self, name, controller):
        self.name = name
        self.controller = controller
        return self

    def preface(self):
        return f"{self.name}: "

    def cc(self, message):
        return origen.tester.cc(f"{self.preface()}{message}")
