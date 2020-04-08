# Test pattern that attempts to drive the clk pin, cycle for a bit, drive it low again, and cycle a bit more.
with produce_pattern() as pat:
    dut().pin("portc").drive(1)
    dut().pin("clk").drive(1)
    tester().repeat(15)
    for i in range(0, 10):
        if i % 2:
            dut().pin("clk").drive(0)
            tester().cycle()
        else:
            dut().pin("clk").drive(1)
            tester().cycle()
    dut().pin("clk").drive(0)
    dut().pin("portc").drive(0)
    tester().repeat(15)
