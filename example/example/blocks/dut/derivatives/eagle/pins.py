Pin("porta", width= 2)
Pin("portb", width= 4)
Pin("portc", width=2, reset_data=0b11)
Pin("clk", reset_data=0, reset_action="D")
Alias("clk", "swd_clk", "tclk")