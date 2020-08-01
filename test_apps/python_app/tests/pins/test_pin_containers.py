import pytest
import origen, _origen # pylint: disable=import-error
from tests.shared.python_like_apis import Fixture_DictLikeAPI # pylint: disable=import-error
from tests.shared import instantiate_dut, clean_falcon # pylint: disable=import-error

def test_empty_pins(clean_falcon):
  assert len(origen.dut.pins) == 0
  assert len(origen.dut.physical_pins) == 0

class TestPinContainerDictLike(Fixture_DictLikeAPI):
  def parameterize(self):
    return {
      "keys": ["p0", "p1", "p2", "p3"],
      "klass": _origen.dut.pins.PinGroup,
      "not_in_dut": "p4"
    }

  def boot_dict_under_test(self):
    instantiate_dut("dut.falcon")
    dut = origen.dut
    dut.add_pin("p0")
    dut.add_pin("p1")
    dut.add_pin("p2")
    dut.add_pin("p3")
    return dut.pins

class TestPhysicalPinContainerDictLike(Fixture_DictLikeAPI):
  def parameterize(self):
    return {
      "keys": ["p0", "p1", "p2", "p3"],
      "klass": _origen.dut.pins.Pin,
      "not_in_dut": "p4"
    }

  def boot_dict_under_test(self):
    instantiate_dut("dut.falcon")
    dut = origen.dut
    dut.add_pin("p0")
    dut.add_pin("p1")
    dut.add_pin("p2")
    dut.add_pin("p3")
    return dut.physical_pins
