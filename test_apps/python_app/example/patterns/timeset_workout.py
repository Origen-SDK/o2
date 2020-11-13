# Test pattern that attempts to drive the clk pin, cycle for a bit, drive it low again, and cycle a bit more.
with Pattern(pin_header="all") as pat:
    tester().cc("Toggle 'clk' for a few pulses with the 'simple' timeset")
    tester().set_timeset('simple')
    dut().pin("clk").drive(1).cycle()
    dut().pin("clk").drive(0).cycle()
    dut().pin("clk").drive(1).cycle()
    dut().pin("clk").drive(0).cycle()

    tester().cc("Toggle 'clk' for a few pulses with the 'backwards' timeset")
    tester().set_timeset('backwards')
    dut().pin("clk").drive(1).cycle()
    dut().pin("clk").drive(0).cycle()
    dut().pin("clk").drive(1).cycle()
    dut().pin("clk").drive(0).cycle()

    tester().set_timeset('simple')
    dut().timeset('simple').symbol_map['a'] = 'a'
    dut().timeset('simple').symbol_map['b'] = 'b'
    dut().timeset('simple').symbol_map['6'] = '6'
    dut().timeset('simple').symbol_map['7'] = '7'
    dut().timeset('simple').symbol_map['8'] = '8'
    dut().timeset('simple').symbol_map['9'] = '9'

    tester().cc("Set the clk to an arbitrary symbol")
    # This should not cycle the tester
    dut().pin("clk").set_actions("|9|")
    tester().cycle()

    tester().cc("Set porta to various symbols")
    dut().pin("porta").set_actions("|b||a|")
    tester().cycle()
