import pytest, pathlib, inspect
import multiprocessing as mp
import origen, _origen  # pylint: disable=import-error
import tests._shared
tmp_dir = tests._shared.tmp_dir

backend_testers = [
    "ALL", "V93K", "V93KSMT7", "V93KSMT8", "IGXL", "J750", "ULTRAFLEX",
    "SIMULATOR", "DUMMYRENDERER", "DUMMYRENDERERWITHINTERCEPTORS"
]


@pytest.fixture
def clean_eagle():
    instantiate_dut("dut.eagle")
    if len(origen.tester.targets) == 0:
        origen.tester.target("DummyRenderer")
    assert origen.dut
    return origen.dut


@pytest.fixture
def clean_bald_eagle():
    instantiate_dut("dut.eagle.bald_eagle")
    if len(origen.tester.targets) == 0:
        origen.tester.target("DummyRenderer")
    assert origen.dut
    return origen.dut


@pytest.fixture
def clean_falcon():
    instantiate_dut("dut.falcon")
    assert origen.dut
    return origen.dut


@pytest.fixture
def clean_tester():
    assert origen.tester
    origen.tester.reset()
    _origen.start_new_test()
    assert len(origen.test_ast()["children"]) == 0
    assert origen.tester.targets == []
    assert origen.tester.timeset is None


@pytest.fixture
def clean_dummy():
    assert origen.tester
    origen.tester.reset()
    origen.tester.target("DummyRenderer")
    _origen.start_new_test()
    assert len(origen.test_ast()["children"]) == 0
    assert origen.tester.targets == ["DUMMYRENDERER"]
    assert origen.tester.timeset is None


@pytest.fixture
def clean_compiler():
    assert origen.app
    origen.app.compiler.clear()
    assert len(origen.app.compiler.stack) == 0
    assert len(origen.app.compiler.renders) == 0
    assert len(origen.app.compiler.output_files) == 0


def check_last_node_type(t):
    assert origen.test_ast()["children"][-1]["attrs"][0] == t


def get_last_node():
    return origen.test_ast()["children"][-1]


def _get_calling_file_stem():
    return pathlib.Path(inspect.stack()[2].filename).stem


def in_new_origen_proc(func=None, mod=None, options=None, expect_fail=False):
    if func is None:
        func = getattr(mod, inspect.stack()[1].function)
    context = mp.get_context("spawn")
    q = context.Queue()
    proc = context.Process(target=func, args=(q, options))
    proc.start()
    proc.join()
    results = {}
    while not q.empty():
        # Convert the populated Queue to a dictionary
        obj = q.get()
        results[obj[0]] = obj[1]
    if expect_fail:
        assert proc.exitcode == 1
    else:
        assert proc.exitcode == 0
    return results


def instantiate_dut(name):
    origen.target.load(lambda: origen.app.instantiate_dut(name))

class Targets:
    class Target:
        def __init__(self, name, offset, tester_name=None):
            self.name = name
            if offset:
                self.relative_path = f"{offset}/{name}.py"
            else:
                self.relative_path = f"{name}.py"
            self.full_path = str(origen.app.root.joinpath("targets").joinpath(self.rp))
            if tester_name:
                self.tester_name = tester_name

        @property
        def rp(self):
            return self.relative_path
        
        @property
        def fp(self):
            return self.full_path

    hawk = Target("hawk", "dut")
    falcon = Target("falcon", "dut")
    eagle = Target("eagle", "dut")
    o1_dut0 = Target("o1_dut0", "dut")
    uflex = Target("uflex", "tester", tester_name="ULTRAFLEX")
    j750 = Target("j750", "tester", tester_name="J750")
    smt7 = Target("v93k_smt7", "tester", tester_name="V93KSMT7")
    smt8 = Target("v93k_smt8", "tester", tester_name="V93KSMT8")
    sim = Target("sim", "tester", tester_name="SIMULATOR")
    eagle_with_smt7 = Target("eagle_with_smt7", None)
    eagle_with_simulator = Target("eagle_with_simulator", None)

    def show_per_cmd_targets(self, targets=None, **kwargs):
        from tests.cli.shared import CLICommon
        prefix = "Current Targets: "
        if targets is not None:
            if targets is not False:
                if not isinstance(targets, list):
                    targets = [targets]
                targets = [(t.name if isinstance(t, Targets.Target) else t) for t in targets]
            kwargs.setdefault("run_opts", {})["targets"] = targets
        out = CLICommon.eval(f"print( f'{prefix}{{origen.target.current_targets}}' )", **kwargs)
        print(out)
        out = out.split("\n")
        return eval(next(t.replace(prefix, '') for t in out if t.startswith(prefix)))

    all = [
        eagle_with_smt7,
        j750, smt8, smt7, uflex, sim,
        o1_dut0, falcon, hawk, eagle,
        eagle_with_simulator,
    ]

    all_rp = [t.rp for t in all]

    class TargetFixtures:
        @pytest.fixture
        def hawk(self):
            return Targets.hawk

        @pytest.fixture
        def falcon(self):
            return Targets.falcon

        @pytest.fixture
        def eagle(self):
            return Targets.eagle

        @pytest.fixture
        def uflex(self):
            return Targets.uflex

        @pytest.fixture
        def j750(self):
            return Targets.j750

        @pytest.fixture
        def smt7(self):
            return Targets.smt7

        @pytest.fixture
        def smt8(self):
            return Targets.smt8

        @pytest.fixture
        def sim(self):
            return Targets.sim

        @pytest.fixture
        def eagle_with_smt7(self):
            return Targets.eagle_with_smt7

        @pytest.fixture
        def eagle_with_simulator(self):
            return Targets.eagle_with_simulator

class PythonAppCommon:
    targets = Targets()
    bypass_origen_app_lookup_env = {"origen_app_bypass_config_lookup": "1", "origen_app_name": "example"}
    configs_dir = pathlib.Path(__file__).parent.parent.joinpath("configs")

    @classmethod
    def to_config_path(cls, config):
        return cls.configs_dir.joinpath(config)