from ...controller import Controller as Parent
import origen


class Controller(Parent):
    def write_register(self, reg_or_val, size=None, address=None, **kwargs):
        # All write register requests originated from within this block (or one of its children)
        # will be sent to the parent class by default, however you can intercept it here and do
        # something else if required
        super().write_register(reg_or_val, size, address, **kwargs)

    def verify_register(self, reg_or_val, size=None, address=None, **kwargs):
        # A verify register requests originated from within this block (or one of its children)
        # will be sent to the parent class by default, however you can intercept it here and do
        # something else if required
        super().verify_register(reg_or_val, size, address, **kwargs)

    @property
    def blocks(self):
        return [self.b0, self.b1, self.b2]
