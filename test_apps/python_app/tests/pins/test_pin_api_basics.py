import pytest
import origen, _origen  # pylint: disable=import-error
from origen.pins import PinActions  # pylint: disable=import-error
from tests.shared.python_like_apis import Fixture_DictLikeAPI  # pylint: disable=import-error
from tests.shared import clean_falcon  # pylint: disable=import-error
from tests.pins import is_pin_group, pins, grp  # pylint: disable=import-error


def test_pin_group_base_state(clean_falcon):
    p = origen.dut.add_pin("p0")
    assert isinstance(p, _origen.dut.pins.PinGroup)
    assert p.name == "p0"
    assert p.reset_actions == "Z"
    assert p.actions == "Z"
    assert p.little_endian
    assert not p.big_endian
    assert p.width == 1
    assert p.pin_names == ["p0"]
    assert p.width == 1


def test_pin_base_state(clean_falcon):
    origen.dut.add_pin("p0")
    p = origen.dut.physical_pin("p0")
    assert isinstance(p, _origen.dut.pins.Pin)
    assert p.name == "p0"
    assert p.action == "Z"
    assert p.reset_action is None
    assert p.aliases == []
    assert p.groups == {}


class TestDefiningPins:
    def test_adding_pins_with_width(self, clean_falcon):
        origen.dut.add_pin("porta", width=4)
        assert origen.dut.pins.keys() == [
            "porta0", "porta1", "porta2", "porta3", "porta"
        ]
        assert len(origen.dut.pins) == 5
        assert origen.dut.physical_pins.keys() == [
            "porta0", "porta1", "porta2", "porta3"
        ]
        assert len(origen.dut.physical_pins) == 4
        assert origen.dut.physical_pin("porta0").groups == {"porta": 0}
        assert origen.dut.physical_pin("porta3").groups == {"porta": 3}
        assert origen.dut.pin("porta").pin_names == [
            "porta0", "porta1", "porta2", "porta3"
        ]
        assert origen.dut.pin("porta").width == 4

    def test_adding_pins_with_width_and_offset(self, clean_falcon):
        origen.dut.add_pin("porta", width=2, offset=2)
        assert origen.dut.pins.keys() == ["porta2", "porta3", "porta"]
        assert len(origen.dut.pins) == 3

    def test_adding_pins_with_width_1(self, clean_falcon):
        origen.dut.add_pin("porta", width=1)
        assert origen.dut.pins.keys() == ["porta0", "porta"]
        assert len(origen.dut.pins) == 2

    def test_adding_pins_with_width_1_offset_0(self, clean_falcon):
        origen.dut.add_pin("porta", width=1, offset=0)
        assert origen.dut.pins.keys() == ["porta0", "porta"]
        assert len(origen.dut.pins) == 2

    def test_adding_pins_with_reset_values(self, clean_falcon):
        grp = origen.dut.add_pin("porta",
                                 width=4,
                                 reset_data=0xC,
                                 reset_action="ABCD")
        assert grp.actions == "ABCD"
        assert isinstance(grp.reset_actions, PinActions)
        assert grp.reset_actions == "ABCD"
        # Check the physical pins
        assert origen.dut.physical_pin("porta0").reset_action == "D"
        assert origen.dut.physical_pin("porta1").reset_action == "C"
        assert origen.dut.physical_pin("porta2").reset_action == "B"
        assert origen.dut.physical_pin("porta3").reset_action == "A"

    def test_adding_pins_with_more_involved_reset_actions(self, clean_falcon):
        grp = origen.dut.add_pin("porta",
                                 width=4,
                                 reset_data=0xC,
                                 reset_action=PinActions("ACCB"))
        assert grp.reset_actions == "ACCB"
        assert grp.actions == "ACCB"
        assert origen.dut.physical_pin("porta0").reset_action == "B"
        assert origen.dut.physical_pin("porta1").reset_action == "C"
        assert origen.dut.physical_pin("porta2").reset_action == "C"
        assert origen.dut.physical_pin("porta3").reset_action == "A"

    def test_reset_actions_set_reset_data_when_applicable(self, clean_falcon):
        grp = origen.dut.add_pin("porta",
                                 width=4,
                                 reset_action=PinActions("HL10"))
        assert grp.reset_actions == "HL10"
        assert grp.actions == "HL10"
        assert origen.dut.physical_pin("porta0").reset_action == "0"
        assert origen.dut.physical_pin("porta0").reset_data == 0
        assert origen.dut.physical_pin("porta1").reset_action == "1"
        assert origen.dut.physical_pin("porta1").reset_data == 1
        assert origen.dut.physical_pin("porta2").reset_action == "L"
        assert origen.dut.physical_pin("porta2").reset_data == 0
        assert origen.dut.physical_pin("porta3").reset_action == "H"
        assert origen.dut.physical_pin("porta3").reset_data == 1

    def test_exception_on_invalid_width(self, clean_falcon):
        assert "porta" not in origen.dut.pins
        with pytest.raises(OSError):
            origen.dut.add_pin("porta", width=0)
        with pytest.raises(OverflowError):
            origen.dut.add_pin("porta", width=-1)

    def test_exception_on_invalid_offset(self, clean_falcon):
        assert "porta" not in origen.dut.pins
        with pytest.raises(OverflowError):
            origen.dut.add_pin("porta", offset=-1)

    # Exception raised when the pin name has already been added.
    def test_exception_on_explicit_duplicate(self, clean_falcon):
        origen.dut.add_pin("p0")
        assert len(origen.dut.pins) == 1
        assert len(origen.dut.physical_pins) == 1
        with pytest.raises(OSError):
            origen.dut.add_pin("p0")
        assert len(origen.dut.pins) == 1
        assert len(origen.dut.physical_pins) == 1

    # Exception raised when a would-be pin name conflicts with an existing pin.
    def test_exception_on_implicit_duplicate(self, clean_falcon):
        origen.dut.add_pin("port", width=4)
        assert len(origen.dut.pins) == 5
        with pytest.raises(OSError):
            origen.dut.add_pin("port0")
        assert len(origen.dut.pins) == 5

    def test_exception_on_missing_pins(self, clean_falcon):
        assert origen.dut.pin("blah") is None
        with pytest.raises(KeyError):
            origen.dut.pins['blah']

    def test_exception_on_offset_with_no_width(self, clean_falcon):
        assert len(origen.dut.pins) == 0
        with pytest.raises(OSError):
            origen.dut.add_pin("invalid", offset=2)
        assert len(origen.dut.pins) == 0

    def test_exception_on_invalid_reset_actions(self, clean_falcon):
        assert "invalid" not in origen.dut.pins
        with pytest.raises(OSError):
            origen.dut.add_pin("invalid", reset_action="ZZ")
        with pytest.raises(OSError):
            origen.dut.add_pin("invalid", width=2, reset_action="Z")
        with pytest.raises(OSError):
            origen.dut.add_pin("invalid",
                               width=2,
                               reset_action=PinActions("Z"))
        with pytest.raises(OSError):
            origen.dut.add_pin("invalid", width=2, reset_action="ZZZ")
        with pytest.raises(OSError):
            origen.dut.add_pin("invalid",
                               width=2,
                               reset_action=PinActions("ZZZ"))
        with pytest.raises(TypeError):
            origen.dut.add_pin("invalid", width=2, reset_action=42)
        assert "invalid" not in origen.dut.pins


class TestSettingStates:
    def test_driving_pins(self, clean_falcon, pins, grp):
        grp = origen.dut.pin("grp")
        is_pin_group(grp.drive(0))
        assert grp.actions == "000"
        assert origen.dut.pins["p1"].actions == "0"
        assert origen.dut.pins["p2"].actions == "0"
        assert origen.dut.pins["p3"].actions == "0"

    def test_driving_pins_with_data(self, clean_falcon, pins, grp):
        grp = origen.dut.pin("grp")
        is_pin_group(grp.drive(0x7))
        assert grp.actions == "111"

    def test_veriying_pins(self, clean_falcon, pins, grp):
        grp = origen.dut.pin("grp")
        is_pin_group(grp.verify(0))
        assert grp.actions == "LLL"

    def test_veriying_pins_with_data(self, clean_falcon, pins, grp):
        grp = origen.dut.pin("grp")
        is_pin_group(grp.verify(0x7))
        assert grp.actions == "HHH"

    # Changes to capture API leads this to fail. Need to review.
    @pytest.mark.xfail
    def test_capturing_pins(self, clean_falcon, pins, grp):
        grp = origen.dut.pin("grp")
        is_pin_group(grp.capture())
        assert grp.actions == "CCC"
        assert origen.dut.pins["p1"].actions == "C"
        assert origen.dut.pins["p2"].actions == "C"
        assert origen.dut.pins["p3"].actions == "C"

    def test_tristating_pins(self, clean_falcon, pins, grp):
        grp = origen.dut.pin("grp")
        grp.drive(0)
        assert grp.actions == "000"
        is_pin_group(grp.highz())
        assert grp.actions == "ZZZ"
        assert origen.dut.pins["p1"].actions == "Z"
        assert origen.dut.pins["p2"].actions == "Z"
        assert origen.dut.pins["p3"].actions == "Z"

    def test_setting_pins_to_arbitrary_actions(self, clean_falcon, pins, grp):
        grp = origen.dut.pin("grp")
        grp.set_actions("ABC")
        assert grp.actions == "ABC"
        assert origen.dut.pins["p1"].actions == "C"
        assert origen.dut.pins["p2"].actions == "B"
        assert origen.dut.pins["p3"].actions == "A"

    def test_setting_pins_using_actions_class(self, clean_falcon, pins, grp):
        grp = origen.dut.pin("grp")
        grp.set_actions(PinActions("DEC"))
        assert grp.actions == "DEC"
        assert origen.dut.pins["p1"].actions == "C"
        assert origen.dut.pins["p2"].actions == "E"
        assert origen.dut.pins["p3"].actions == "D"

    def test_setting_states_with_mask(self, clean_falcon, pins, grp):
        grp = origen.dut.pin("grp")
        assert grp.actions == "ZZZ"
        grp.set_actions(PinActions("ABC"), mask=0x3)
        assert grp.actions == "ZBC"
        assert origen.dut.pins["p1"].actions == "C"
        assert origen.dut.pins["p2"].actions == "B"
        assert origen.dut.pins["p3"].actions == "Z"

    def test_setting_states_updates_data_when_appropriate(
            self, clean_falcon, pins, grp):
        grp = origen.dut.pin("grp")
        assert grp.actions == "ZZZ"
        grp.set_actions("1HL")
        assert grp.actions == "1HL"

    def test_exception_on_overflow_actions(self, clean_falcon, pins, grp):
        grp = origen.dut.pin("grp")
        assert grp.actions == "ZZZ"
        with pytest.raises(OSError):
            grp.drive(8)
        with pytest.raises(OSError):
            grp.verify(8)
        assert grp.actions == "ZZZ"

    def test_exception_on_invalid_actions(self, clean_falcon, pins, grp):
        grp = origen.dut.pin("grp")
        with pytest.raises(TypeError):
            grp.drive({})
        with pytest.raises(TypeError):
            grp.verify([])
