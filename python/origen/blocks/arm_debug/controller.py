import _origen
from origen.controller import Base


class Controller(_origen.standard_sub_blocks.arm_debug.ArmDebug, Base):
    def __init__(self):
        Base.__init__(self)
        _origen.standard_sub_blocks.arm_debug.ArmDebug.__init__(self)
