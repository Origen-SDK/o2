import _origen

with Flow() as flow:
    flow.add_test("opens")
    flow.add_test("shorts")
    flow.func("test_a")
    flow.include("wt1_start.py")
    flow.include("wt1_end")
    flow.bin(1)
    #import pdb; pdb.set_trace();
    _origen.flow()
