import origen
from origen.controller import Base

class GenericClkCtrl(Base):
    def __init__(self):
        Base.__init__(self)
        self.callbacks = []

    @property
    def ctrl(self):
        return self.reg("ctrl")

    @Base.startup
    def enable_cc(self, **startup_opts):
        # self.cc("Enabling clks", include_source=True)
        self.callbacks.append("base__enable_cc_called")
        self.ctrl.fields["enable"].write(1)
    
    @Base.startup
    def wait_for_enable(self):
        # self.cc("Waiting for clks to enable", include_source=True)
        self.callbacks.append("base__wait_for_enable_called")
        origen.tester.repeat(100)

    @Base.shutdown
    def disable_cc(self, *startup_opts):
        self.callbacks.append("base__disable_cc_called")
        # self.cc("Disabling clks", include_source=True)
        self.ctrl.fields["enable"].write(0)
    
    @Base.shutdown
    def wait_for_disable(self):
        self.callbacks.append("base__wait_for_disable_called")
        # self.cc("Waiting for clks to disable", include_source=True)
        origen.tester.repeat(100)