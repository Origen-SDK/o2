import pytest, pathlib, inspect, os, sys
import multiprocessing as mp
import origen, _origen  # pylint: disable=import-error

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


def in_new_origen_proc(func=None, mod=None, options=None):
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
    assert proc.exitcode == 0
    return results


def setenv(q, bypass_config_lookup=None):
    import os, inspect, pathlib, sys
    if bypass_config_lookup:
        os.environ['origen_bypass_config_lookup'] = "1"
    os.environ['origen_config_paths'] = str(
        pathlib.Path(__file__).parent.joinpath(
            f"{inspect.stack()[1].function}.toml").absolute())


def tmp_dir():
    t = pathlib.Path(__file__).parent.parent.parent.joinpath('tmp/pytest')
    if not t.exists():
        t.mkdir(parents=True, exist_ok=True)
    return t


def instantiate_dut(name):
    origen.target.load(lambda: origen.app.instantiate_dut(name))
