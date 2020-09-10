from origen.controller import Base as BaseController
import origen


class Controller(BaseController):
    def write_register(self, reg_or_val, size=None, address=None, **kwargs):
        # All write register requests originated from within this block (or one of its children)
        # will be sent to the parent block by default, however you can intercept it here and do
        # something else if required
        self.parent.write_register(reg_or_val, size, address, **kwargs)

    def verify_register(self, reg_or_val, size=None, address=None, **kwargs):
        # A verify register requests originated from within this block (or one of its children)
        # will be sent to the parent block by default, however you can intercept it here and do
        # something else if required
        self.parent.verify_register(reg_or_val, size, address, **kwargs)
