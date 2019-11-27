from origen import dut, Flow;

with Flow() as flow:
    flow.test("opens")
    flow.test("shorts")
    flow.bin(1)
