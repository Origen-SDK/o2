# pylint: disable=undefined-variable
Pin("porta", width=2)
Pin("portb", width=4)
Pin("portc", width=2)
# Pin("portc", width=2, reset_actions=0b11)
Pin("clk", reset_action="0")
Alias("clk", "swd_clk", "swdclk", "tclk")
Alias("porta0", "swdio")
Alias("portc0", "reset")
PinHeader("ports", "porta", "portb", "portc")
PinHeader("clk", "clk")
PinHeader("all", "clk", "porta", "portb", "portc")
PinHeader("pins-for-toggle", "clk", "portc")
PinHeader("pins-for-toggle-rev", "portc", "clk")
PinHeader("swd", "reset", "swdclk", "swdio")
PinHeader("cap_test", "swdclk", "swdio", "portc")
