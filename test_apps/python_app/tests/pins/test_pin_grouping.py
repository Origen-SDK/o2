import pytest
import origen, _origen  # pylint: disable=import-error
from tests.shared import instantiate_dut, clean_falcon  # pylint: disable=import-error
from tests.shared.python_like_apis import Fixture_ListLikeAPI  # pylint: disable=import-error
from tests.pins import is_pin_group, pins, ports, grp  # pylint: disable=import-error


class TestPinGroupListLike(Fixture_ListLikeAPI):
    def parameterize(self):
        return {"slice_klass": _origen.dut.pins.PinCollection}

    def verify_i0(self, i):
        assert isinstance(i, _origen.dut.pins.PinCollection)
        assert i.pin_names == ["pins0"]

    def verify_i1(self, i):
        assert isinstance(i, _origen.dut.pins.PinCollection)
        assert i.pin_names == ["pins1"]

    def verify_i2(self, i):
        assert isinstance(i, _origen.dut.pins.PinCollection)
        assert i.pin_names == ["pins2"]

    def boot_list_under_test(self):
        instantiate_dut("dut.falcon")
        origen.dut.add_pin("pins", width=3)
        return origen.dut.pin("pins")


class TestGrouping:
    def test_grouping_pins_by_name(self, clean_falcon, pins):
        grp = origen.dut.group_pins("grp", "p0", "p1", "p2")
        is_pin_group(grp)
        assert grp.name == "grp"
        assert grp.pin_names == ["p0", "p1", "p2"]
        assert grp.little_endian
        assert "p0" in grp
        assert "p1" in grp
        assert "p2" in grp

    def test_grouping_pins_by_regex(self, clean_falcon, pins, ports):
        import re
        r = re.compile("port.0")
        grp = origen.dut.group_pins("grp", r)
        assert grp.pin_names == ["porta0", "portb0"]

    def test_collecting_using_ruby_like_syntax(self, clean_falcon, pins,
                                               ports):
        grp = origen.dut.group_pins("grp", "/port.0/")
        assert grp.pin_names == ["porta0", "portb0"]

    def test_grouping_pins_by_mixture(self, clean_falcon, pins, ports):
        import re
        r = re.compile("port.0")
        grp = origen.dut.group_pins("grp", "/port.1/", "p1", r)
        assert grp.pin_names == ["porta1", "portb1", "p1", "porta0", "portb0"]

    def test_big_endian(self, clean_falcon):
        pins = origen.dut.add_pin("portc", width=4, little_endian=False)
        assert pins.pin_names == ["portc3", "portc2", "portc1", "portc0"]
        assert not pins.little_endian
        assert pins.big_endian

    def test_little_endian(self, clean_falcon):
        pins = origen.dut.add_pin("portd", width=4, little_endian=True)
        assert pins.pin_names == ["portd0", "portd1", "portd2", "portd3"]
        assert pins.little_endian
        assert not pins.big_endian

    def test_big_endian_with_offset(self, clean_falcon):
        pins = origen.dut.add_pin("porte",
                                  width=2,
                                  little_endian=False,
                                  offset=2)
        assert pins.pin_names == ["porte3", "porte2"]
        assert not pins.little_endian
        assert pins.big_endian

    def test_grouping_mixed_endianness(self, clean_falcon):
        origen.dut.add_pin("portc", width=4, little_endian=False)
        origen.dut.add_pin("portd", width=4, little_endian=True)
        grp = origen.dut.group_pins("mixed_endianness", "portc", "portd")
        assert grp.pin_names == [
            "portc3", "portc2", "portc1", "portc0", "portd0", "portd1",
            "portd2", "portd3"
        ]
        assert grp.little_endian == True
        assert grp.big_endian == False

    def test_grouping_big_endian(self, clean_falcon):
        origen.dut.add_pin("portc", width=4, little_endian=False)
        origen.dut.add_pin("portd", width=4, little_endian=True)
        grp = origen.dut.group_pins("big_endian",
                                    "portc",
                                    "portd",
                                    little_endian=False)
        assert grp.pin_names == [
            "portd3", "portd2", "portd1", "portd0", "portc0", "portc1",
            "portc2", "portc3"
        ]
        assert grp.little_endian == False
        assert grp.big_endian == True

    def test_exception_on_missing_pins(self, clean_falcon, pins):
        assert "fail" not in origen.dut.pins
        with pytest.raises(RuntimeError):
            origen.dut.group_pins("fail", "p0", "p1", "blah")
        assert "fail" not in origen.dut.pins

    def test_exception_on_grouping_duplicates(self, clean_falcon, pins, grp):
        with pytest.raises(RuntimeError):
            origen.dut.group_pins("invalid", "p1", "p1", "p1")
        assert "invalid" not in origen.dut.pins

    def test_exception_on_grouping_aliases_of_the_same_pin(
            self, clean_falcon, pins, grp):
        origen.dut.add_pin_alias("p1", "a1")
        with pytest.raises(RuntimeError):
            origen.dut.group_pins("invalid", "p1", "p2", "a1", "p3")
        assert "invalid" not in origen.dut.pins

    def test_exception_on_grouping_duplicates_when_nested(
            self, clean_falcon, ports):
        assert "porta" in origen.dut.pins
        assert "porta0" in origen.dut.pins
        assert "porta1" in origen.dut.pins
        assert "invalid" not in origen.dut.pins
        with pytest.raises(RuntimeError):
            origen.dut.group_pins("grouping_porta", "porta", "porta0",
                                  "porta1")
        assert "invalid" not in origen.dut.pins
