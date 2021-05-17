import origen
from origen.controller import Base
from .base_controller import GenericClkCtrl

class FastClkCtrl(GenericClkCtrl):
    def __init__(self):
        GenericClkCtrl.__init__(self)
    
    # This should override the base's equivalent method
    @Base.startup
    def enable_cc(self, **startup_opts):
        self.callbacks.append("fast_clk_ctrl__enable_cc_called")
        self.ctrl.fields["src"].set_data(0xC)
        super

    # This also should override the base's
    @Base.shutdown
    def wait_for_disable(self, *startup_opts):
        self.callbacks.append("fast_clk_ctrl__wait_for_disable_called")
        origen.tester.repeat(50)
    
    @Base.startup
    def check_enabled(self):
        self.callbacks.append("fast_clk_ctrl__check_enabled_called")
        self.ctrl.fields["enable"].verify(1)

class SlowClkCtrl(GenericClkCtrl):
    @Base.startup
    def enable(self, **startup_opts):
        self.callbacks.append("slow_clk_ctrl__enable")
        self.ctrl.fields["enable"].set_data(1)
        self.ctrl.fields["src"].set_data(0xE)
        self.ctrl.fields["slow_nfast"].set_data(1)
        self.ctrl.write()

    @Base.shutdown
    def do_nothing(self):
        self.callbacks.append("slow_clk_ctrl__do_nothing")