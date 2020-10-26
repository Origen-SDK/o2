import pytest
import origen, _origen  # pylint: disable=import-error
from origen.pins import PinActions  # pylint: disable=import-error
from tests.shared import instantiate_dut, clean_falcon  # pylint: disable=import-error
from tests.shared.python_like_apis import Fixture_ListLikeAPI  # pylint: disable=import-error
from tests.pins import is_pin_collection, pins, ports, grp  # pylint: disable=import-error


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
        assert c.actions == "101"

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

    def test_driving_pin_collection(self, clean_falcon, pins):
        c = origen.dut.pins.collect("p1", "p2", "p3")
        c.drive(0x2)
        assert c.actions == "010"
        assert origen.dut.physical_pin("p1").action == "0"
        assert origen.dut.physical_pin("p2").action == "1"
        assert origen.dut.physical_pin("p3").action == "0"

    def test_verifying_pin_collection(self, clean_falcon, pins):
        c = origen.dut.pins.collect("p1", "p2", "p3")
        c.verify(0x5)
        assert c.actions == "HLH"
        assert origen.dut.physical_pin("p1").action == "H"
        assert origen.dut.physical_pin("p2").action == "L"
        assert origen.dut.physical_pin("p3").action == "H"

    def test_tristating_pin_collection(self, clean_falcon, pins):
        c = origen.dut.pins.collect("p1", "p2", "p3")
        c.drive(0x0)
        assert c.actions == "000"
        c.highz()
        assert c.actions == "ZZZ"
        assert origen.dut.physical_pin("p1").action == "Z"
        assert origen.dut.physical_pin("p2").action == "Z"
        assert origen.dut.physical_pin("p3").action == "Z"

    def test_capturing_pin_collection(self, clean_falcon, pins):
        c = origen.dut.pins.collect("p1", "p2", "p3")
        c.capture()
        assert c.actions == "CCC"
        assert origen.dut.physical_pin("p1").action == "C"
        assert origen.dut.physical_pin("p2").action == "C"
        assert origen.dut.physical_pin("p3").action == "C"

    def test_reset_values_persist_in_collections(self, clean_falcon, pins):
        origen.dut.add_pin("port", width=2, reset_data=0x3, reset_action="DD")
        c = origen.dut.pins.collect("p0", "p1", "port")
        assert c.reset_actions == "DDZZ"
        assert c.actions == "DDZZ"

    def test_resetting_collection(self, clean_falcon, pins):
        origen.dut.add_pin("port", width=2, reset_data=0x3, reset_action="DD")
        c = origen.dut.pins.collect("p0", "p1", "port")
        c.verify(0x3)
        assert c.actions == "LLHH"
        c.reset()
        assert c.actions == "DDZZ"

    def test_setting_actions_with_mask(self, clean_falcon, pins):
        c = origen.dut.pins.collect("p0", "p1")
        c.highz()
        assert c.actions == "ZZ"
        c.set_actions("CC", mask=0x1)
        assert c.actions == "ZC"

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

    def test_getting_nested_actions(self, clean_falcon, ports):
        grp = origen.dut.group_pins("ports", "porta", "portb")
        c = origen.dut.pins.collect("porta", "portb")
        assert grp.actions == "ZZZZZZ"
        assert c.actions == "ZZZZZZ"

    def test_setting_nested_actions(self, clean_falcon, ports):
        grp = origen.dut.group_pins("ports", "porta", "portb")
        c = origen.dut.pins.collect("porta", "portb")
        grp.capture()
        assert grp.actions == "CCCCCC"
        assert c.actions == "CCCCCC"
        c.verify(0x0)
        assert c.actions == "LLLLLL"
        assert grp.actions == "LLLLLL"

    def test_directly_setting_nested_actions(self, clean_falcon, ports):
        grp = origen.dut.group_pins("ports", "porta", "portb")
        c = origen.dut.pins.collect("porta", "portb")
        assert isinstance(c.actions, PinActions)
        c.actions = "CCCZZZ"
        assert c.actions == "CCCZZZ"
        assert grp.actions == "CCCZZZ"
        c.set_actions("111HHH")
        assert c.actions == "111HHH"
        assert grp.actions == "111HHH"
        c.actions = PinActions("CCCZZZ")
        assert c.actions == "CCCZZZ"
        assert grp.actions == "CCCZZZ"
        c.set_actions(PinActions("111HHH"))
        assert c.actions == "111HHH"
        assert grp.actions == "111HHH"

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
        assert c.actions == "ZZZ"
        with pytest.raises(OSError):
            c.drive(0xF)
        with pytest.raises(OverflowError):
            c.drive(-1)
        assert c.actions == "ZZZ"

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

    def test_exception_on_invalid_actions(self, clean_falcon, ports):
        c = origen.dut.pins.collect("porta", "portb")
        with pytest.raises(OSError):
            c.actions = "Z"
        with pytest.raises(OSError):
            c.set_actions(PinActions("ZZZZZZZZZZ"))
