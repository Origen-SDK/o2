import pytest
import origen, _origen # pylint: disable=import-error
from tests.shared import clean_falcon # pylint: disable=import-error
from tests.pins import is_pin_group, pins, ports, grp, check_alias # pylint: disable=import-error

class TestPinAliasing:
  def test_aliasing_pins(self, clean_falcon, pins, grp, ports):
    assert "a1" not in origen.dut.pins
    origen.dut.add_pin_alias("p1", "a1")
    assert "a1" in origen.dut.pins
    a1 = origen.dut.pin("a1")
    is_pin_group(a1)
    assert len(a1) == 1
    assert a1.pin_names == ["p1"]

  def test_adding_multiple_aliases(self, clean_falcon, pins):
    origen.dut.add_pin_alias("p1", "a1_a", "a1_b", "a1_c")
    check_alias("p1", "a1_a")
    check_alias("p1", "a1_b")
    check_alias("p1", "a1_c")

  def test_aliasing_pins_and_groups(self, clean_falcon, ports):
    origen.dut.add_pin_alias("porta", "pta", "pA")
    assert origen.dut.pin("pta").pin_names == origen.dut.pin("porta").pin_names
    assert origen.dut.pin("pA").pin_names == origen.dut.pin("porta").pin_names

  def test_actions_and_data_using_aliases(self, clean_falcon, pins):
    origen.dut.add_pin_alias("p1", "a1")
    p1 = origen.dut.pin("p1")
    a1 = origen.dut.pin("a1")
    a1.drive(1)
    assert a1.actions == "1"
    assert p1.actions == "1"

    a1.verify(0)
    assert a1.actions == "L"
    assert p1.actions == "L"

  def test_aliasing_an_alias(self, clean_falcon, pins, grp, ports):
    origen.dut.add_pin_alias("p1", "a1")
    origen.dut.add_pin_alias("a1", "_a1")
    assert "_a1" in origen.dut.pins
    assert origen.dut.pins["_a1"].pin_names == ["p1"]

  def test_exception_on_duplicate_aliases(self, clean_falcon, pins, grp, ports):
    assert "a1" not in origen.dut.pins
    origen.dut.add_pin_alias("p1", "a1")
    assert "a1" in origen.dut.pins
    with pytest.raises(OSError):
      origen.dut.add_pin_alias("p1", "a1")

  def test_exception_on_aliasing_missing_pin(self, clean_falcon, pins, grp, ports):
    assert "blah" not in origen.dut.pins
    assert "alias_blah" not in origen.dut.pins
    with pytest.raises(OSError):
      origen.dut.add_pin_alias("blah", "alias_blah")
    assert "blah" not in origen.dut.pins
    assert "alias_blah" not in origen.dut.pins
