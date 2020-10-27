import _origen
from origen.controller import Base


class Controller(_origen.standard_sub_blocks.arm_debug.DP, Base):
    def __init__(self):
        Base.__init__(self)
        _origen.standard_sub_blocks.arm_debug.DP.__init__(self)
