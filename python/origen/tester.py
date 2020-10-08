import origen
import _origen
import pickle
from contextlib import contextmanager, ContextDecorator

# Where mutliple are given the first one should be the one use internally by
# Origen, the others are for user convenience
SPECIFIC_TESTER_NAMES = [
    "v93k_smt7", "v93k_smt8", "j750", ["ultraflex", "uflex"]
]


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

    @contextmanager
    def specific(self, name):
        clean_name = next(
            (x for x in SPECIFIC_TESTER_NAMES if _is_name_match(name, x)),
            None)
        if clean_name is None:
            raise ValueError(
                f"unknown specific tester name '{name}', should be one of: {SPECIFIC_TESTER_NAMES}"
            )
        # TODO: open an AST node here
        if name == "v93k_smt7":
            yield V93K(7)
        elif name == "v93k_smt8":
            yield V93K(8)
        # TODO: and close it here


def _is_name_match(name, options):
    name = name.lower().replace("_", "")
    if isinstance(options, list):
        for n in options:
            if name == n.lower().replace("_", ""):
                return True
        return False
    else:
        return name == options.lower().replace("_", "")


class DummyTester:
    def __init__(self):
        pass

    def generate(self, ast):
        for i, n in enumerate(ast.nodes):
            print(f"Python Generator: Node: {i}: {n}")


class V93K(_origen.tester_apis.V93K):
    pass
