import pytest
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

def check_last_node_type(t):
  assert origen.test_ast()["children"][-1]["attrs"][0] == t

def get_last_node():
  return origen.test_ast()["children"][-1]
