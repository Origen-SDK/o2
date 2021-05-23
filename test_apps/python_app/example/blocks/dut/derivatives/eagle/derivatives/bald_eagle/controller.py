from example.blocks.dut.derivatives.eagle.controller import Controller as Parent
import origen
from origen.controller import TopLevel

class Controller(Parent):
    @TopLevel.startup
    def startup(self, **kwargs):
        Parent.startup(self, **kwargs)
        self.startup_source = "bald_eagle"
        self.arm_debug.switch_to_swd()
