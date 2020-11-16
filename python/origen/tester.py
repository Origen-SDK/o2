import origen
import _origen
import pickle
from contextlib import contextmanager, ContextDecorator


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
    def eq(self, *names):
        (pat_ref_id, prog_ref_id,
         clean_tester_names) = self._start_eq_block(names)
        testers = []
        for t in clean_tester_names:
            if t == "V93K":
                testers.append(V93K())
            elif t == "V93KSMT7":
                testers.append(V93K(7))
            elif t == "V93KSMT8":
                testers.append(V93K(8))
            elif t == "IGXL":
                testers.append(IGXL())
            elif t == "ULTRAFLEX":
                testers.append(IGXL("ULTRAFLEX"))
            elif t == "J750":
                testers.append(IGXL("J750"))
            else:
                raise Exception(
                    f"The API for tester '{t}' has not been implemented yet!")

        if len(testers) == 1:
            yield testers[0]
        elif len(testers) == 2:
            yield testers[0], testers[1]
        elif len(testers) == 3:
            yield testers[0], testers[1], testers[2]
        elif len(testers) == 4:
            yield testers[0], testers[1], testers[2], testers[3]
        elif len(testers) == 5:
            yield testers[0], testers[1], testers[2], testers[3], testers[4]
        elif len(testers) == 6:
            yield testers[0], testers[1], testers[2], testers[3], testers[
                4], testers[5]
        elif len(testers) == 7:
            yield testers[0], testers[1], testers[2], testers[3], testers[
                4], testers[5], testers[6]
        elif len(testers) == 8:
            yield testers[0], testers[1], testers[2], testers[3], testers[
                4], testers[5], testers[6], testers[7]
        elif len(testers) == 9:
            yield testers[0], testers[1], testers[2], testers[3], testers[
                4], testers[5], testers[6], testers[7], testers[8]
        elif len(testers) == 10:
            yield testers[0], testers[1], testers[2], testers[3], testers[
                4], testers[5], testers[6], testers[7], testers[8], testers[9]
        else:
            raise Exception(
                f"Only up to 10 testers are supported in a with-specific-tester block"
            )
        for t in testers:
            del t
        self._end_eq_block(pat_ref_id, prog_ref_id)

    @contextmanager
    def neq(self, *names):
        (pat_ref_id, prog_ref_id,
         clean_tester_names) = self._start_neq_block(names)
        yield
        self._end_neq_block(pat_ref_id, prog_ref_id)


class DummyTester:
    def __init__(self):
        pass

    def generate(self, ast):
        for i, n in enumerate(ast.nodes):
            print(f"Python Generator: Node: {i}: {n}")


class V93K(_origen.tester_apis.V93K):
    pass


class IGXL(_origen.tester_apis.IGXL):
    pass
