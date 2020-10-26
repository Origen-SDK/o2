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
    assert p.reset_data == 0
    assert p.reset_actions == "Z"
    assert p.data == 0
    assert p.pin_actions == "Z"
    assert p.little_endian
    assert not p.big_endian
    assert p.width == 1
    assert p.physical_names == ["p0"]
    assert p.pin_names == ["p0"]
    assert p.width == 1


def test_pin_base_state(clean_falcon):
    origen.dut.add_pin("p0")
    p = origen.dut.physical_pin("p0")
    assert isinstance(p, _origen.dut.pins.Pin)
    assert p.name == "p0"
    assert p.data == 0
    assert p.action == "Z"
    assert p.reset_data is None
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
                                 reset_action="DVDV")
        assert grp.data == 0xC
        assert grp.pin_actions == "1H0L"
        assert grp.reset_data == 0xC
        assert isinstance(grp.reset_actions, PinActions)
        assert grp.reset_actions == "1H0L"
        # Check the physical pins
        assert origen.dut.physical_pin("porta0").reset_action == "L"
        assert origen.dut.physical_pin("porta0").reset_data == 0
        assert origen.dut.physical_pin("porta1").reset_action == "0"
        assert origen.dut.physical_pin("porta1").reset_data == 0
        assert origen.dut.physical_pin("porta2").reset_action == "H"
        assert origen.dut.physical_pin("porta2").reset_data == 1
        assert origen.dut.physical_pin("porta3").reset_action == "1"
        assert origen.dut.physical_pin("porta3").reset_data == 1

    def test_adding_pins_with_more_involved_reset_actions(self, clean_falcon):
        grp = origen.dut.add_pin("porta",
                                 width=4,
                                 reset_data=0xC,
                                 reset_action=PinActions("|A|CC|B|"))
        assert grp.data == 0xC
        assert grp.reset_actions == "|A|CC|B|"
        assert grp.pin_actions == "|A|CC|B|"
        assert origen.dut.physical_pin("porta0").reset_action == "|B|"
        assert origen.dut.physical_pin("porta0").reset_data == 0
        assert origen.dut.physical_pin("porta1").reset_action == "C"
        assert origen.dut.physical_pin("porta1").reset_data == 0
        assert origen.dut.physical_pin("porta2").reset_action == "C"
        assert origen.dut.physical_pin("porta2").reset_data == 1
        assert origen.dut.physical_pin("porta3").reset_action == "|A|"
        assert origen.dut.physical_pin("porta3").reset_data == 1

    def test_reset_actions_set_reset_data_when_applicable(self, clean_falcon):
        grp = origen.dut.add_pin("porta",
                                 width=4,
                                 reset_action=PinActions("HL10"))
        assert grp.reset_data == 0xA
        assert grp.reset_actions == "HL10"
        assert grp.data == 0xA
        assert grp.pin_actions == "HL10"
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

    def test_exception_on_invalid_reset_data(self, clean_falcon):
        assert "invalid" not in origen.dut.pins
        with pytest.raises(OSError):
            origen.dut.add_pin("invalid", reset_data=2)
        with pytest.raises(OSError):
            origen.dut.add_pin("invalid", width=2, reset_data=4)
        with pytest.raises(OverflowError):
            origen.dut.add_pin("invalid", width=2, reset_data=-1)
        with pytest.raises(TypeError):
            origen.dut.add_pin("invalid", width=2, reset_data="hi!")
        assert "invalid" not in origen.dut.pins

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
        with pytest.raises(OSError):
            origen.dut.add_pin("invalid", width=2, reset_action="HI")
        with pytest.raises(OSError):
            origen.dut.add_pin("invalid", width=2, reset_action="**")
        with pytest.raises(TypeError):
            origen.dut.add_pin("invalid", width=2, reset_action=42)
        assert "invalid" not in origen.dut.pins

    def test_exception_when_reset_actions_alter_reset_data(self, clean_falcon):
        assert "invalid" not in origen.dut.pins
        with pytest.raises(OSError):
            origen.dut.add_pin("invalid",
                               width=2,
                               reset_data=0x3,
                               reset_action="00")
        assert "invalid" not in origen.dut.pins


class TestSettingStates:
    def test_setting_data(self, clean_falcon, pins, grp):
        # Observe that the underlying pin state has changed,
        # therefore changing ALL references to that/those pin(s)
        grp = origen.dut.pin("grp")
        assert grp.data == 0
        grp.data = 0x3
        assert grp.data == 3
        assert origen.dut.pins["p1"].data == 1
        assert origen.dut.pins["p2"].data == 1
        assert origen.dut.pins["p3"].data == 0

        # Set the data in a pin reference in the above group and ensure
        # that the update is reflected there.
        origen.dut.pins["p3"].data = 1
        assert origen.dut.pins["p3"].data == 1
        assert grp.data == 7

    def test_driving_pins(self, clean_falcon, pins, grp):
        grp = origen.dut.pin("grp")
        is_pin_group(grp.drive())
        assert grp.pin_actions == "000"
        assert grp.data == 0
        assert origen.dut.pins["p1"].pin_actions == "0"
        assert origen.dut.pins["p2"].pin_actions == "0"
        assert origen.dut.pins["p3"].pin_actions == "0"

    def test_driving_pins_with_data(self, clean_falcon, pins, grp):
        grp = origen.dut.pin("grp")
        is_pin_group(grp.drive(0x7))
        assert grp.data == 0x7
        assert grp.pin_actions == "111"

    def test_veriying_pins(self, clean_falcon, pins, grp):
        grp = origen.dut.pin("grp")
        is_pin_group(grp.verify())
        assert grp.data == 0
        assert grp.pin_actions == "LLL"

    def test_veriying_pins_with_data(self, clean_falcon, pins, grp):
        grp = origen.dut.pin("grp")
        is_pin_group(grp.verify(0x7))
        assert grp.data == 0x7
        assert grp.pin_actions == "HHH"

    def test_capturing_pins(self, clean_falcon, pins, grp):
        grp = origen.dut.pin("grp")
        is_pin_group(grp.capture())
        assert grp.pin_actions == "CCC"
        assert origen.dut.pins["p1"].pin_actions == "C"
        assert origen.dut.pins["p2"].pin_actions == "C"
        assert origen.dut.pins["p3"].pin_actions == "C"

    def test_tristating_pins(self, clean_falcon, pins, grp):
        grp = origen.dut.pin("grp")
        grp.drive()
        assert grp.pin_actions == "000"
        is_pin_group(grp.highz())
        assert grp.pin_actions == "ZZZ"
        assert origen.dut.pins["p1"].pin_actions == "Z"
        assert origen.dut.pins["p2"].pin_actions == "Z"
        assert origen.dut.pins["p3"].pin_actions == "Z"

    def test_setting_pins_to_arbitrary_actions(self, clean_falcon, pins, grp):
        grp = origen.dut.pin("grp")
        grp.set_actions("|A||B||C|")
        assert grp.pin_actions == "|A||B||C|"
        assert origen.dut.pins["p1"].pin_actions == "|C|"
        assert origen.dut.pins["p2"].pin_actions == "|B|"
        assert origen.dut.pins["p3"].pin_actions == "|A|"

    def test_setting_pins_using_pin_actions_class(self, clean_falcon, pins,
                                                  grp):
        grp = origen.dut.pin("grp")
        grp.set_actions(PinActions("|D||E|C"))
        assert grp.pin_actions == "|D||E|C"
        assert origen.dut.pins["p1"].pin_actions == "C"
        assert origen.dut.pins["p2"].pin_actions == "|E|"
        assert origen.dut.pins["p3"].pin_actions == "|D|"

    def test_setting_states_with_nonsticky_mask(self, clean_falcon, pins, grp):
        grp = origen.dut.pin("grp")
        grp.with_mask(0x3).set_actions(PinActions("|A||B||C|"))
        assert grp.pin_actions == "Z|B||C|"
        assert origen.dut.pins["p1"].pin_actions == "|C|"
        assert origen.dut.pins["p2"].pin_actions == "|B|"
        assert origen.dut.pins["p3"].pin_actions == "Z"

    def test_setting_states_updates_data_when_appropriate(
            self, clean_falcon, pins, grp):
        grp = origen.dut.pin("grp")
        assert grp.pin_actions == "ZZZ"
        assert grp.data == 0
        grp.set_actions("1HL")
        assert grp.pin_actions == "1HL"
        assert grp.data == 0x6

    def test_resetting_data(self, clean_falcon):
        grp = origen.dut.add_pin("porta",
                                 width=4,
                                 reset_data=0xC,
                                 reset_action="DVDV")
        assert grp.data == 0xC
        assert grp.pin_actions == "1H0L"
        grp.drive(0x3)
        assert grp.data == 0x3
        assert grp.pin_actions == "0011"
        grp.reset()
        assert grp.data == 0xC
        assert grp.pin_actions == "1H0L"

    def test_exception_on_overflow_data(self, clean_falcon, pins, grp):
        grp = origen.dut.pin("grp")
        grp.data = 7
        assert grp.data == 7
        with pytest.raises(OSError):
            grp.data = 8
        with pytest.raises(OSError):
            grp.set(8)
        assert grp.data == 7

    def test_exception_on_invalid_data(self, clean_falcon, pins, grp):
        grp = origen.dut.pin("grp")
        with pytest.raises(TypeError):
            grp.data = "hi"
        with pytest.raises(TypeError):
            grp.set({})
        with pytest.raises(TypeError):
            grp.data = []
        with pytest.raises(TypeError):
            grp.set(origen)
        with pytest.raises(OverflowError):
            grp.data = -1

    def test_exception_on_overflow_actions(self, clean_falcon, pins, grp):
        grp = origen.dut.pin("grp")
        grp.data = 7
        assert grp.data == 7
        assert grp.pin_actions == "ZZZ"
        with pytest.raises(OSError):
            grp.drive(8)
        with pytest.raises(OSError):
            grp.verify(8)
        assert grp.pin_actions == "ZZZ"
        assert grp.data == 7

    def test_exception_on_invalid_actions(self, clean_falcon, pins, grp):
        grp = origen.dut.pin("grp")
        with pytest.raises(TypeError):
            grp.drive({})
        with pytest.raises(TypeError):
            grp.verify([])