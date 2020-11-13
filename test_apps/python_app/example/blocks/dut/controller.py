from origen.controller import TopLevel


class Controller(TopLevel):
    def write_register(self, reg_or_val, size=None, address=None, **kwargs):
        self.jtag.write_ir(0xF, size=8)
        self.jtag.write_dr(reg_or_val, size)

    def verify_register(self, reg_or_val, size=None, address=None, **kwargs):
        self.jtag.write_ir(0x1F, size=8)
        self.jtag.verify_dr(reg_or_val, size)
