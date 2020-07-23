import origen
import _origen
import pickle

class Tester(_origen.tester.PyTester):
  def __init__(self):
    pass
    #self.db = _origen.tester.PyTester("placeholder")
    #_origen.tester.PyTester.init(self, "placeholder")
  
  def set_timeset(self, tset):
    # For simplicity, a timeset can be given as a string which is assumed to be a top-level timeset.
    # Due to lazy loading though, its possible that the timesets haven't been instantiated yet, causing
    # a very confusing 'no timeset found' error, yet then using 'dut.timesets' to check shows them as loaded.
    # Load them here, if they haven't already.
    if origen.dut and not origen.dut.timesets_loaded:
      origen.dut.timesets
    return _origen.tester.PyTester.set_timeset(self, tset)

  # Returns stats on the number of patterns generated, etc.
  def stats(self):
    return pickle.loads(bytes(self._stats()))

class DummyTester:
  def __init__(self):
    pass

  def generate(self, ast):
    for i, n in enumerate(ast.nodes):
      print(f"Python Generator: Node: {i}: {n}")