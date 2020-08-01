import pytest
import origen, _origen # pylint: disable=import-error
from tests.shared import instantiate_dut, clean_eagle, clean_falcon, clean_tester # pylint: disable=import-error
from tests.shared.python_like_apis import Fixture_DictLikeAPI # pylint: disable=import-error
from tests.pins import ports, pins # pylint: disable=import-error

class TestPinHeaders:
  class TestPinHeadersDictLikeAPI(Fixture_DictLikeAPI):
    def parameterize(self):
      return {
        "keys": ["h0", "h1", "h2"],
        "klass": _origen.dut.pins.PinHeader,
        "not_in_dut": "Blah"
      }

    def boot_dict_under_test(self):
      instantiate_dut("dut.falcon")
      origen.dut.add_pin("p", width=3)
      origen.dut.add_pin_header("h0", "p0")
      origen.dut.add_pin_header("h1", "p0", "p1")
      origen.dut.add_pin_header("h2", "p0", "p1", "p2")
      return origen.dut.pin_headers

  def test_empty_pin_headers(self, clean_falcon):
    assert origen.dut.pin_headers.keys() == []

  def test_adding_pin_headers(self, clean_falcon, pins):
    h = origen.dut.add_pin_header("header", "p0", "p1", "p2")
    assert isinstance(h, _origen.dut.pins.PinHeader)
    assert h.pin_names == ["p0", "p1", "p2"]
  
  def test_adding_pin_headers_with_groups(self, clean_falcon, pins, ports):
    h = origen.dut.add_pin_header("header", "p0", "portb")
    assert isinstance(h, _origen.dut.pins.PinHeader)
    assert h.pin_names == ["p0", "portb"]
    assert h.physical_names == ["p0", "portb0", "portb1"]
    assert len(h) == 3
    assert h.width == 3

  def test_adding_pin_headers_with_aliases(self, clean_falcon, pins, ports):
    origen.dut.add_pin_alias("portb", "pb")
    h = origen.dut.add_pin_header("header", "p0", "pb")
    assert isinstance(h, _origen.dut.pins.PinHeader)
    assert h.pin_names == ["p0", "pb"]
    assert h.physical_names == ["p0", "portb0", "portb1"]
    assert len(h) == 3
    assert h.width == 3

  def test_adding_pin_header_based_on_another(self, clean_falcon, pins, ports):
    origen.dut.add_pin_header("header", "p0", "portb")
    h = origen.dut.add_pin_header("header2", *origen.dut.pin_headers["header"].pin_names)
    assert isinstance(h, _origen.dut.pins.PinHeader)
    assert h.pin_names == ["p0", "portb"]
    assert h.physical_names == ["p0", "portb0", "portb1"]
    assert len(h) == 3
    assert h.width == 3

  def test_exception_on_missing_pins(self, clean_falcon, pins):
    assert "invalid" not in origen.dut.pin_headers
    with pytest.raises(OSError):
      origen.dut.add_pin_header("header", "p0", "missing")
    assert "invalid" not in origen.dut.pin_headers
  
  def test_exception_on_duplicate_pins(self, clean_falcon, pins, ports):
    assert "invalid" not in origen.dut.pin_headers
    with pytest.raises(OSError):
      origen.dut.add_pin_header("header", "p0", "p0")
    with pytest.raises(OSError):
      origen.dut.add_pin_header("header", "p0", "a0")
    with pytest.raises(OSError):
      origen.dut.add_pin_header("header", "portb0", "portb")
    assert "invalid" not in origen.dut.pin_headers
