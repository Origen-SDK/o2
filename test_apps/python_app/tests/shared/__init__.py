import pytest, pathlib, inspect
import origen, _origen  # pylint: disable=import-error

backend_testers = [
    "V93KSMT7", "V93KSMT8", "J750", "ULTRAFLEX", "SIMULATOR", "DUMMYRENDERER",
    "DUMMYRENDERERWITHINTERCEPTORS"
]


@pytest.fixture
def clean_eagle():
    instantiate_dut("dut.eagle")
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
    assert origen.tester.targets == ["::DummyRenderer"]
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


def tmp_dir():
    t = pathlib.Path(__file__).parent.parent.parent.joinpath('tmp/pytest')
    return t
    if not t.exists():
        pathlib.mkdir_p(t)
    return t


def instantiate_dut(name):
    origen.target.load(lambda: origen.app.instantiate_dut(name))
