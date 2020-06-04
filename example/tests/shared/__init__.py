import pytest, pathlib, inspect
import origen, _origen # pylint: disable=import-error

@pytest.fixture
def clean_eagle():
  origen.app.instantiate_dut("dut.eagle")
  assert origen.dut
  return origen.dut

@pytest.fixture
def clean_falcon():
  origen.app.instantiate_dut("dut.falcon")
  assert origen.dut
  return origen.dut

@pytest.fixture
def clean_tester():
  assert origen.tester
  origen.tester.reset()
  assert len(origen.test_ast()["children"]) == 0
  assert origen.tester.targets == []
  assert origen.tester.testers == ["::DummyRenderer", "::DummyRendererWithInterceptors", "::V93K::ST7", "::Simulator"]
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
