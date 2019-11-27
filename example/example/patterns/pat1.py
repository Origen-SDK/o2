from origen import dut, tester, Pattern;

with Pattern() as pattern:
    dut.do_something()
    tester.wait()
