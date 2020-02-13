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
  assert origen.tester.targets == []
  assert len(origen.tester.ast) == 0
  assert origen.tester.generators == ["::DummyGenerator", "::DummyGeneratorWithInterceptors", "::V93K::ST7", "::Simulator"]
  assert origen.tester.timeset is None