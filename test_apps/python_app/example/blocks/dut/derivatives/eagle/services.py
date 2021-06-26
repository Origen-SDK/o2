from origen.services import *

Service("swd", SWD())
Service("simple_8bit", Simple("porta0", "portb", "porta1", 8))
Service("simple_32bit",
        Simple(width=32, data="portb", clk="portc0", read_nwrite="portc1"))
