# Test pattern that attempts to drive the clk pin, cycle for a bit, drive it low again, and cycle a bit more.
with produce_pattern() as pat:
    dut.pin("clk").drive(1)
    tester.repeat(100)
    dut.pin("clk").drive(0)
    tester.repeat(100)
