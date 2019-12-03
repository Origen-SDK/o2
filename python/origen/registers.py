from contextlib import contextmanager

# A middleman between the Python controller and the associated Rust model and
# which implements the application/user API for working with registers.
# An instance of this class is returned by <my_controller>.regs
class Proxy:
    def __init__(self, model):
        self.model = model

# This defines the methods for defining registers in Python and then handles serializing
# the definitions and handing them over to the Rust model for instantiation.
class Loader:
    def __init__(self, model):
        self.model = model

    @contextmanager
    def reg(self, name, address_offset, size=32):
        try:
            yield self
        finally:
            pass

    def bit(self, number, name, access="rw", reset=0):
        pass

    # Defines the methods that are accessible within blocks/<block>/registers.py
    def api(self):
        return {
            "reg": self.reg, 
            "bit": self.bit
        }
                
