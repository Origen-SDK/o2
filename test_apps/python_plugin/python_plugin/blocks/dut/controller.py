from origen.controller import TopLevel as BaseController
import origen


class Controller(BaseController):
    def write_register(self, reg_or_val, size=None, address=None, **kwargs):
        # Invoke your driver of choice to dispatch this write_register request, 
        # here is a JTAG example:
        #self.jtag.write_ir(0xF, size=8)
        #self.jtag.write_dr(reg_or_val, size)
        raise RuntimeError(f"A request to write a register was received by '{self.path}' ({type(self)}), however the logic to implement it has not been defined yet")

    def verify_register(self, reg_or_val, size=None, address=None, **kwargs):
        # Invoke your driver of choice to dispatch this verify_register request, 
        # here is a JTAG example:
        #self.jtag.write_ir(0x1F, size=8)
        #self.jtag.verify_dr(reg_or_val, size)
        raise RuntimeError(f"A request to verify a register was received by '{self.path}' ({type(self)}), however the logic to implement it has not been defined yet")
