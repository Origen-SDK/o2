from ...controller import Controller as Parent
import origen


class Controller(Parent):
    def startup(self, **kwargs):
        origen.tester.set_timeset("simple")
        origen.tester.pin_header = self.pin_headers[kwargs.get(
            "pin_header", "pins-for-toggle")]
        origen.tester.repeat(100)

    def shutdown(self, **kwargs):
        origen.tester.repeat(10)

    def write_register(self, reg_or_val, size=None, address=None, **kwargs):
        return self.arm_debug.sys.write_register(reg_or_val, size=None, address=None, **kwargs)

    def verify_register(self, reg_or_val, size=None, address=None, **kwargs):
        return self.arm_debug.sys.verify_register(reg_or_val, size=None, address=None, **kwargs)

    def capture_register(self, reg_or_val, **kwargs):
        return self.arm_debug.sys.capture_register(reg_or_val, **kwargs)
