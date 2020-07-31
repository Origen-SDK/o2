import pytest
import origen, _origen # pylint: disable=import-error

@pytest.fixture
def clean_eagle():
  instantiate_dut("dut.eagle")
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

def check_last_node_type(t):
  assert origen.test_ast()["children"][-1]["attrs"][0] == t

def get_last_node():
  return origen.test_ast()["children"][-1]

def instantiate_dut(name):
    origen.target.load(lambda: origen.app.instantiate_dut(name))