from origen.controller import TopLevel

class Controller(TopLevel):
    def write_register(self, reg_or_val, **kwargs):
        self.jtag.write_ir()

    def verify_register(self, reg_or_val, **kwargs):
        self.jtag.write_ir()