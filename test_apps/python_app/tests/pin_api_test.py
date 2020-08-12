import pytest
import origen, _origen  # pylint: disable=import-error
from tests.shared import *
from tests.shared.python_like_apis import Fixture_DictLikeAPI, Fixture_ListLikeAPI  # pylint: disable=import-error


class MyRandomClass:
    pass


def is_pin_group(obj):
    assert isinstance(obj, _origen.dut.pins.PinGroup)


def is_pin_collection(obj):
    assert isinstance(obj, _origen.dut.pins.PinCollection)


def is_pin(obj):
    assert isinstance(obj, _origen.dut.pins.Pin)


def check_alias(pin_name, alias_name):
    assert alias_name in origen.dut.pins
    assert origen.dut.pins[alias_name].pin_names == [pin_name]
    assert alias_name in origen.dut.physical_pin(pin_name).aliases


# Most basic operations will be hit just by the DictLikeAPIs passing.
# That is, if those pass then we already know that adding/retrieving a single pin,
#   length, and contains already work. So, focus the remaining tests on aspects
#   not covered by the API testers.


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


class TestPinCollectionListLike(Fixture_ListLikeAPI):
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
        return origen.dut.pins.collect("pins0", "pins1", "pins2")


@pytest.fixture
def ports():
    origen.dut.add_pin("porta", width=4)
    origen.dut.add_pin("portb", width=2)


@pytest.fixture
def pins():
    origen.dut.add_pin("p0")
    origen.dut.add_pin("p1")
    origen.dut.add_pin("p2")
    origen.dut.add_pin("p3")


@pytest.fixture
def grp():
    grp = origen.dut.group_pins("grp", "p1", "p2", "p3")
    assert grp.data == 0
    assert grp.pin_actions == "ZZZ"


def test_empty_pins(clean_falcon):
    assert len(origen.dut.pins) == 0
    assert len(origen.dut.physical_pins) == 0


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
    assert p.action == "HighZ"
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
        assert grp.pin_actions == "VDVD"
        assert grp.reset_data == 0xC
        assert grp.reset_actions == "VDVD"
        # Check the physical pins
        assert origen.dut.physical_pin("porta0").reset_action == "V"
        assert origen.dut.physical_pin("porta0").reset_data == 0
        assert origen.dut.physical_pin("porta1").reset_action == "D"
        assert origen.dut.physical_pin("porta1").reset_data == 0
        assert origen.dut.physical_pin("porta2").reset_action == "V"
        assert origen.dut.physical_pin("porta2").reset_data == 1
        assert origen.dut.physical_pin("porta3").reset_action == "D"
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
            origen.dut.add_pin("invalid", width=2, reset_action="ZZZ")
        with pytest.raises(OSError):
            origen.dut.add_pin("invalid", width=2, reset_action="HI")
        with pytest.raises(OSError):
            origen.dut.add_pin("invalid", width=2, reset_action="**")
        with pytest.raises(TypeError):
            origen.dut.add_pin("invalid", width=2, reset_action=42)
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
        assert grp.drive() is None
        assert grp.pin_actions == "DDD"
        assert grp.data == 0
        assert origen.dut.pins["p1"].pin_actions == "D"
        assert origen.dut.pins["p2"].pin_actions == "D"
        assert origen.dut.pins["p3"].pin_actions == "D"

    def test_driving_pins_with_data(self, clean_falcon, pins, grp):
        grp = origen.dut.pin("grp")
        assert grp.drive(0x7) is None
        assert grp.data == 0x7
        assert grp.pin_actions == "DDD"

    def test_veriying_pins(self, clean_falcon, pins, grp):
        grp = origen.dut.pin("grp")
        assert grp.verify() is None
        assert grp.data == 0
        assert grp.pin_actions == "VVV"

    def test_veriying_pins_with_data(self, clean_falcon, pins, grp):
        grp = origen.dut.pin("grp")
        assert grp.verify(0x7) is None
        assert grp.data == 0x7
        assert grp.pin_actions == "VVV"

    def test_capturing_pins(self, clean_falcon, pins, grp):
        grp = origen.dut.pin("grp")
        assert grp.capture() is None
        assert grp.pin_actions == "CCC"
        assert origen.dut.pins["p1"].pin_actions == "C"
        assert origen.dut.pins["p2"].pin_actions == "C"
        assert origen.dut.pins["p3"].pin_actions == "C"

    def test_tristating_pins(self, clean_falcon, pins, grp):
        grp = origen.dut.pin("grp")
        grp.drive()
        assert grp.pin_actions == "DDD"
        assert grp.highz() is None
        assert grp.pin_actions == "ZZZ"
        assert origen.dut.pins["p1"].pin_actions == "Z"
        assert origen.dut.pins["p2"].pin_actions == "Z"
        assert origen.dut.pins["p3"].pin_actions == "Z"

    def test_resetting_data(self, clean_falcon):
        grp = origen.dut.add_pin("porta",
                                 width=4,
                                 reset_data=0xC,
                                 reset_action="DVDV")
        assert grp.data == 0xC
        assert grp.pin_actions == "VDVD"
        grp.drive(0x3)
        assert grp.data == 0x3
        assert grp.pin_actions == "DDDD"
        grp.reset()
        assert grp.data == 0xC
        assert grp.pin_actions == "VDVD"

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
        assert origen.dut.pin("pta").pin_names == origen.dut.pin(
            "porta").pin_names
        assert origen.dut.pin("pA").pin_names == origen.dut.pin(
            "porta").pin_names

    def test_actions_and_data_using_aliases(self, clean_falcon, pins):
        origen.dut.add_pin_alias("p1", "a1")
        p1 = origen.dut.pin("p1")
        a1 = origen.dut.pin("a1")
        assert a1.drive(1) is None
        assert a1.data == 1
        assert a1.pin_actions == "D"
        assert p1.data == 1
        assert p1.pin_actions == "D"

        assert a1.verify(0) is None
        assert a1.data == 0
        assert a1.pin_actions == "V"
        assert p1.data == 0
        assert p1.pin_actions == "V"

    def test_aliasing_an_alias(self, clean_falcon, pins, grp, ports):
        origen.dut.add_pin_alias("p1", "a1")
        origen.dut.add_pin_alias("a1", "_a1")
        assert "_a1" in origen.dut.pins
        assert origen.dut.pins["_a1"].pin_names == ["p1"]

    def test_exception_on_duplicate_aliases(self, clean_falcon, pins, grp,
                                            ports):
        assert "a1" not in origen.dut.pins
        origen.dut.add_pin_alias("p1", "a1")
        assert "a1" in origen.dut.pins
        with pytest.raises(OSError):
            origen.dut.add_pin_alias("p1", "a1")

    def test_exception_on_aliasing_missing_pin(self, clean_falcon, pins, grp,
                                               ports):
        assert "blah" not in origen.dut.pins
        assert "alias_blah" not in origen.dut.pins
        with pytest.raises(OSError):
            origen.dut.add_pin_alias("blah", "alias_blah")
        assert "blah" not in origen.dut.pins
        assert "alias_blah" not in origen.dut.pins


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
        with pytest.raises(OSError):
            origen.dut.group_pins("fail", "p0", "p1", "blah")
        assert "fail" not in origen.dut.pins

    def test_exception_on_grouping_duplicates(self, clean_falcon, pins, grp):
        with pytest.raises(OSError):
            origen.dut.group_pins("invalid", "p1", "p1", "p1")
        assert "invalid" not in origen.dut.pins

    def test_exception_on_grouping_aliases_of_the_same_pin(
            self, clean_falcon, pins, grp):
        origen.dut.add_pin_alias("p1", "a1")
        with pytest.raises(OSError):
            origen.dut.group_pins("invalid", "p1", "p2", "a1", "p3")
        assert "invalid" not in origen.dut.pins

    def test_exception_on_grouping_duplicates_when_nested(
            self, clean_falcon, ports):
        assert "porta" in origen.dut.pins
        assert "porta0" in origen.dut.pins
        assert "porta1" in origen.dut.pins
        assert "invalid" not in origen.dut.pins
        with pytest.raises(OSError):
            origen.dut.group_pins("grouping_porta", "porta", "porta0",
                                  "porta1")
        assert "invalid" not in origen.dut.pins


class TestCollecting:
    def test_collecting_pins(self, clean_falcon, pins):
        n = len(origen.dut.pins)
        # Create an anonymous pin group (pin collection)
        c = origen.dut.pins.collect("p1", "p2")
        assert len(origen.dut.pins) == n
        is_pin_collection(c)
        assert c.pin_names == ["p1", "p2"]

    def test_pin_collection_initial_state(self, clean_falcon, pins):
        origen.dut.pin("p1").drive(1)
        origen.dut.pin("p2").drive(0)
        origen.dut.pin("p3").drive(1)
        c = origen.dut.pins.collect("p1", "p2", "p3")
        assert c.data == 0x5
        assert c.pin_actions == "DDD"

    def test_collecting_with_single_regex(self, clean_falcon, pins, ports):
        import re
        r = re.compile("port.0")
        c = origen.dut.pins.collect(r)
        assert c.pin_names == ["porta0", "portb0"]

    def test_collecting_using_ruby_like_syntax(self, clean_falcon, pins,
                                               ports):
        c = origen.dut.pins.collect("/port.0/")
        assert c.pin_names == ["porta0", "portb0"]

    def test_collecting_with_mixed_inputs(self, clean_falcon, pins, ports):
        import re
        r = re.compile("port.0")
        c = origen.dut.pins.collect("/port.1/", "p1", r)
        assert c.pin_names == ["porta1", "portb1", "p1", "porta0", "portb0"]

    def test_pin_collection_getting_and_setting_data(self, clean_falcon, pins):
        c = origen.dut.pins.collect("p1", "p2", "p3")
        assert c.data == 0x0
        c.data = 0x7
        assert c.data == 0x7
        assert origen.dut.physical_pin("p1").data == 1
        assert origen.dut.physical_pin("p2").data == 1
        assert origen.dut.physical_pin("p3").data == 1
        c.set(0x1)
        assert c.data == 0x1
        assert origen.dut.physical_pin("p1").data == 1
        assert origen.dut.physical_pin("p2").data == 0
        assert origen.dut.physical_pin("p3").data == 0

    def test_driving_pin_collection(self, clean_falcon, pins):
        c = origen.dut.pins.collect("p1", "p2", "p3")
        c.drive()
        assert c.pin_actions == "DDD"
        assert origen.dut.physical_pin("p1").action == "Drive"
        assert origen.dut.physical_pin("p2").action == "Drive"
        assert origen.dut.physical_pin("p3").action == "Drive"

    def test_driving_pin_collection_with_data(self, clean_falcon, pins):
        c = origen.dut.pins.collect("p1", "p2", "p3")
        c.drive(0x7)
        assert c.pin_actions == "DDD"
        assert c.data == 0x7

    def test_verifying_pin_collection(self, clean_falcon, pins):
        c = origen.dut.pins.collect("p1", "p2", "p3")
        c.verify()
        assert c.pin_actions == "VVV"
        assert origen.dut.physical_pin("p1").action == "Verify"
        assert origen.dut.physical_pin("p2").action == "Verify"
        assert origen.dut.physical_pin("p3").action == "Verify"

    def test_verifying_pin_collection_with_data(self, clean_falcon, pins):
        c = origen.dut.pins.collect("p1", "p2", "p3")
        c.verify(0x5)
        assert c.pin_actions == "VVV"
        assert c.data == 0x5

    def test_tristating_pin_collection(self, clean_falcon, pins):
        c = origen.dut.pins.collect("p1", "p2", "p3")
        c.drive()
        assert c.pin_actions == "DDD"
        c.highz()
        assert c.pin_actions == "ZZZ"
        assert origen.dut.physical_pin("p1").action == "HighZ"
        assert origen.dut.physical_pin("p2").action == "HighZ"
        assert origen.dut.physical_pin("p3").action == "HighZ"

    def test_capturing_pin_collection(self, clean_falcon, pins):
        c = origen.dut.pins.collect("p1", "p2", "p3")
        c.capture()
        assert c.pin_actions == "CCC"
        assert origen.dut.physical_pin("p1").action == "Capture"
        assert origen.dut.physical_pin("p2").action == "Capture"
        assert origen.dut.physical_pin("p3").action == "Capture"

    def test_reset_values_persist_in_collections(self, clean_falcon, pins):
        origen.dut.add_pin("port", width=2, reset_data=0x3, reset_action="DD")
        c = origen.dut.pins.collect("p0", "p1", "port")
        assert c.reset_data == 0xC
        assert c.reset_actions == "ZZDD"
        assert c.data == 0xC
        assert c.pin_actions == "ZZDD"

    def test_resetting_collection(self, clean_falcon, pins):
        origen.dut.add_pin("port", width=2, reset_data=0x3, reset_action="DD")
        c = origen.dut.pins.collect("p0", "p1", "port")
        c.verify(0x3)
        assert c.data == 0x3
        assert c.pin_actions == "VVVV"
        c.reset()
        assert c.data == 0xC
        assert c.pin_actions == "ZZDD"

    def test_chaining_method_calls_with_nonsticky_mask(self, clean_falcon,
                                                       pins):
        # This should set the data to 0x3, then drive the pins using mask 0x2.
        # The mask should then be cleared.
        c = origen.dut.pins.collect("p0", "p1")
        c.data = 0x0
        c.highz()
        assert c.data == 0
        assert c.pin_actions == "ZZ"

        c.set(0x3).with_mask(0x2).drive()
        assert c.data == 0x3
        assert c.pin_actions == "DZ"

        # This should set the data and action regardless of the mask being used previously.
        c.data = 0x0
        c.highz()
        assert c.data == 0
        assert c.pin_actions == "ZZ"

        # This should set the data and pin action using the mask 0x1.
        # The mask should then be cleared.
        c.with_mask(0x1).set(0x3).verify()
        assert c.data == 0x1
        assert c.pin_actions == "ZV"

        # This should set the data and action regardless of the mask being used previously.
        c.data = 0x0
        c.highz()
        assert c.data == 0
        assert c.pin_actions == "ZZ"

    def test_collecting_mixed_endianness(self, clean_falcon):
        origen.dut.add_pin("portc", width=4, little_endian=False)
        origen.dut.add_pin("portd", width=4, little_endian=True)
        c = origen.dut.pins.collect("portc", "portd")
        assert c.pin_names == [
            "portc3", "portc2", "portc1", "portc0", "portd0", "portd1",
            "portd2", "portd3"
        ]
        assert c.little_endian == True
        assert c.big_endian == False

    def test_collecting_big_endian(self, clean_falcon):
        origen.dut.add_pin("portc", width=4, little_endian=False)
        origen.dut.add_pin("portd", width=4, little_endian=True)
        c = origen.dut.pins.collect("portc", "portd", little_endian=False)
        assert c.pin_names == [
            "portd3", "portd2", "portd1", "portd0", "portc0", "portc1",
            "portc2", "portc3"
        ]
        assert c.little_endian == False
        assert c.big_endian == True

    def test_getting_nested_pin_data(self, clean_falcon, ports):
        grp = origen.dut.group_pins("ports", "porta", "portb")
        c = origen.dut.pins.collect("porta", "portb")
        assert grp.data == 0
        assert c.data == 0

    def test_getting_nested_pin_actions(self, clean_falcon, ports):
        grp = origen.dut.group_pins("ports", "porta", "portb")
        c = origen.dut.pins.collect("porta", "portb")
        assert grp.pin_actions == "ZZZZZZ"
        assert c.pin_actions == "ZZZZZZ"

    def test_setting_nested_pin_data(self, clean_falcon, ports):
        grp = origen.dut.group_pins("ports", "porta", "portb")
        c = origen.dut.pins.collect("porta", "portb")
        grp.data = 0x2A
        assert grp.data == 0x2A
        assert c.data == 0x2A
        c.data = 0x15
        assert c.data == 0x15
        assert grp.data == 0x15

    def test_setting_nested_pin_actions(self, clean_falcon, ports):
        grp = origen.dut.group_pins("ports", "porta", "portb")
        c = origen.dut.pins.collect("porta", "portb")
        grp.capture()
        assert grp.pin_actions == "CCCCCC"
        assert c.pin_actions == "CCCCCC"
        c.verify()
        assert c.pin_actions == "VVVVVV"
        assert grp.pin_actions == "VVVVVV"

    def test_exception_on_collecting_missing_pins(self, clean_falcon, pins):
        with pytest.raises(OSError):
            origen.dut.pins.collect("p1", "p2", "blah")

    def test_exception_on_collecting_duplicate_pins(self, clean_falcon, pins):
        origen.dut.add_pin_alias("p1", "a1")
        origen.dut.add_pin_alias("a1", "a1_a", "a1_b", "a1_c")
        with pytest.raises(OSError):
            origen.dut.pins.collect("p1", "p1", "p1")
        with pytest.raises(OSError):
            origen.dut.pins.collect("p1", "a1")
        with pytest.raises(OSError):
            origen.dut.pins.collect("a1_a", "a1_b", "a1_c")

    def test_exception_on_overflow_data(self, clean_falcon, pins):
        c = origen.dut.pins.collect("p1", "p2", "p3")
        with pytest.raises(OSError):
            c.data = 0xF
        with pytest.raises(OverflowError):
            c.data = -1
        assert c.data == 0x0

    def test_exception_on_collecting_duplicates_when_nested(
            self, clean_falcon, ports):
        assert "porta" in origen.dut.pins
        assert "porta0" in origen.dut.pins
        assert "porta1" in origen.dut.pins
        assert "invalid" not in origen.dut.pins
        with pytest.raises(OSError):
            origen.dut.pins.collect("invalid", "porta", "porta0", "porta1")
        assert "invalid" not in origen.dut.pins

    def test_exception_on_collecting_duplicates_when_nested_2(
            self, clean_falcon, ports):
        origen.dut.group_pins("index_0s", "porta0", "portb0")
        origen.dut.group_pins("index_0s_rev", "portb0", "porta0")
        assert "invalid" not in origen.dut.pins
        with pytest.raises(OSError):
            origen.dut.pins.collect("invalid", "index_0s", "index_0s_rev")
        assert "invalid" not in origen.dut.pins


def test_pins_in_subblocks(clean_falcon, pins):
    # We should have pins at the DUT level, but not in any subblocks.
    assert len(origen.dut.pins) == 4
    assert len(origen.dut.sub_blocks["core1"].pins) == 0

    # Add a pin at the subblock.
    assert origen.dut.sub_blocks["core1"].add_pin("p1")
    assert len(origen.dut.sub_blocks["core1"].pins) == 1
    p = origen.dut.sub_blocks["core1"].pin("p1")
    is_pin_group(p)

    # Add another pin
    assert origen.dut.sub_blocks["core1"].add_pin("_p1")
    assert len(origen.dut.sub_blocks["core1"].pins) == 2
    _p = origen.dut.sub_blocks["core1"].pin("_p1")
    is_pin_group(_p)

    # Verify the pins at origen.dut are unchanged.
    assert len(origen.dut.pins) == 4
    assert origen.dut.pin("_p1") is None


def test_physical_pin_has_empty_metadata(clean_eagle):
    assert origen.dut.physical_pin("porta0").added_metadata == []


def test_adding_metadata_to_physical_pin():
    # Essentially just check that nothing here throws an exception
    origen.dut.physical_pin("porta0").add_metadata("meta1", 1)
    origen.dut.physical_pin("porta0").add_metadata("meta2", "meta2!")
    origen.dut.physical_pin("porta0").add_metadata("meta3", {})
    origen.dut.physical_pin("porta0").add_metadata("meta4", MyRandomClass())


def test_getting_all_metadata_keys():
    assert origen.dut.physical_pin("porta0").added_metadata == [
        "meta1", "meta2", "meta3", "meta4"
    ]


def test_getting_metadata_from_physical_pin():
    assert origen.dut.physical_pin("porta0").get_metadata("meta1") == 1
    assert origen.dut.physical_pin("porta0").get_metadata("meta2") == "meta2!"
    assert isinstance(
        origen.dut.physical_pin("porta0").get_metadata("meta3"), dict)
    assert isinstance(
        origen.dut.physical_pin("porta0").get_metadata("meta4"), MyRandomClass)


def test_setting_existing_metadata_on_physical_pin():
    assert origen.dut.physical_pin("porta0").set_metadata("meta1", "hi!")
    assert origen.dut.physical_pin("porta0").set_metadata(
        "meta2", "meta2 updated!")
    assert origen.dut.physical_pin("porta0").get_metadata("meta1") == "hi!"
    assert origen.dut.physical_pin("porta0").get_metadata(
        "meta2") == "meta2 updated!"


def test_setting_nonexistant_metadata_adds_it():
    assert origen.dut.physical_pin('porta0').get_metadata("meta5") is None
    assert origen.dut.physical_pin("porta0").set_metadata("meta5",
                                                          5.0) == False
    assert origen.dut.physical_pin("porta0").get_metadata("meta5") == 5.0


def test_interacting_with_reference_metadata():
    d = origen.dut.physical_pin("porta0").get_metadata("meta3")
    assert isinstance(d, dict)
    assert "test" not in d
    d["test"] = True
    assert "test" in d
    d2 = origen.dut.physical_pin("porta0").get_metadata("meta3")
    assert "test" in d2


def test_nonetype_on_retrieving_nonexistant_metadata():
    assert origen.dut.physical_pin("porta0").get_metadata("blah") is None


def test_exception_on_adding_duplicate_metadata():
    with pytest.raises(OSError):
        origen.dut.physical_pin("porta0").add_metadata("meta1", False)


def test_additional_metadata():
    origen.dut.physical_pin('porta1').add_metadata("m1", 1.0)
    origen.dut.physical_pin('porta1').add_metadata("m2", -2)
    assert origen.dut.physical_pin('porta1').get_metadata("m1") == 1.0
    assert origen.dut.physical_pin('porta1').get_metadata("m2") == -2
    assert origen.dut.physical_pin('porta0').get_metadata("m1") is None
    assert origen.dut.physical_pin('porta0').get_metadata("m2") is None


def test_metadata_with_same_name_on_different_objects():
    origen.dut.physical_pin('porta0').add_metadata("index", 0)
    origen.dut.physical_pin('porta1').add_metadata("index", 1)
    assert origen.dut.physical_pin('porta0').get_metadata("index") == 0
    assert origen.dut.physical_pin('porta1').get_metadata("index") == 1


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


class TestPinHeaders:
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

    def test_adding_pin_header_based_on_another(self, clean_falcon, pins,
                                                ports):
        origen.dut.add_pin_header("header", "p0", "portb")
        h = origen.dut.add_pin_header(
            "header2", *origen.dut.pin_headers["header"].pin_names)
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


def test_pin_loader_api(clean_eagle):
    assert origen.dut.pins.keys() == [
        "porta0", "porta1", "porta", "portb0", "portb1", "portb2", "portb3",
        "portb", "portc0", "portc1", "portc", "clk", "swd_clk", "tclk"
    ]
    assert origen.dut.pin("portc").reset_data == 0x3
    assert origen.dut.pin("clk").reset_actions == "D"
    assert origen.dut.pin_headers.keys() == [
        "ports", "clk", "all", "pins-for-toggle", "pins-for-toggle-rev"
    ]
    assert origen.dut.pin_headers["ports"].pin_names == [
        "porta", "portb", "portc"
    ]
    assert origen.dut.pin_headers["clk"].pin_names == ["clk"]
