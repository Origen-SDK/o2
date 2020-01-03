import origen # pylint: disable=import-error
import _origen # pylint: disable=import-error
import pytest

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

def pins():
  origen.dut.pins

def p1():
  origen.dut.pin("p1")

def grp():
  origen.dut.pin("grp")

def test_pin_api_boots():
  origen.app.instantiate_dut("dut.falcon")
  # Ensure the DUT was set and the pin container is available.
  assert origen.dut
  assert isinstance(origen.dut.pins, _origen.dut.pins.PinContainer)

def test_empty_pins():
  assert len(origen.dut.pins) == 0
  assert len(origen.dut.physical_pins) == 0

def test_add_and_retrieving_pins():
  assert len(origen.dut.pins) == 0
  p1 = origen.dut.add_pin("p1")
  is_pin_group(p1)
  assert len(origen.dut.pins) == 1

def test_retrieving_pins_using_pin_method():
  p1 = origen.dut.pin("p1")
  is_pin_group(p1)
  assert p1.name == "p1"

def test_exception_on_duplicate_name():
  with pytest.raises(OSError):
    origen.dut.add_pin("p1")

def test_pin_group_default_state():
  p1 = origen.dut.pin("p1")
  assert p1.name == "p1"
  assert len(p1) == 1
  assert p1._path == ""
  assert p1.data == 0
  assert p1.pin_actions == "Z"
  assert p1.pin_names == ["p1"]
  assert p1.physical_names == ["p1"]
  assert p1.little_endian
  assert not p1.big_endian

def test_retrieving_pins_using_indexing():
  p1 = origen.dut.pins["p1"]
  is_pin_group(p1)
  assert p1.name == "p1"

def test_retrieving_missing_pins():
  assert origen.dut.pin("blah") is None
  with pytest.raises(KeyError):
    origen.dut.pins['blah']

def test_adding_more_pins():
  p2 = origen.dut.add_pin("p2")
  p3 = origen.dut.add_pin("p3")
  is_pin_group(p2)
  is_pin_group(p3)
  assert len(origen.dut.pins) == 3
  assert "p2" in origen.dut.pins
  assert "p3" in origen.dut.pins
  _p2 = origen.dut.pin("p2")
  _p3 = origen.dut.pin("p3")
  is_pin_group(_p2)
  is_pin_group(_p3)
  assert _p2.name == "p2"
  assert _p3.name == "p3"

def test_grouping_pins():
  grp = origen.dut.group_pins("grp", "p1", "p2", "p3")
  is_pin_group(grp)
  assert grp.name == "grp"
  assert grp._path == ""
  assert len(origen.dut.pins) == 4
  assert len(grp) == 3
  assert len(origen.dut.physical_pins) == 3
  assert grp.pin_names == ["p1", "p2", "p3"]
  assert grp.data == 0
  assert grp.pin_actions == "ZZZ"

def test_exception_on_missing_pins():
  assert len(origen.dut.pins) == 4
  with pytest.raises(OSError):
    origen.dut.group_pins("fail", "p2", "p3", "blah")
  assert len(origen.dut.pins) == 4

def test_retrieving_groups():
  grp = origen.dut.pin("grp")
  is_pin_group(grp)
  _grp = origen.dut.pins["grp"]
  is_pin_group(_grp)
  assert grp.name == "grp"
  assert _grp.name == "grp"

def test_checking_names_within_group():
  grp = origen.dut.pins["grp"]
  is_pin_group(grp)
  assert "p1" in grp
  assert "p2" in grp
  assert "p3" in grp

def test_setting_pin_data():
  # Observe that the underlying pin state has changed, therefore changing ALL references to that/those pin(s)
  grp = origen.dut.pin("grp")
  assert grp.data == 0
  grp.data = 0x3
  assert grp.data == 3
  assert origen.dut.pins["p1"].data == 1
  assert origen.dut.pins["p2"].data == 1
  assert origen.dut.pins["p3"].data == 0

  origen.dut.pins["p3"].data = 1
  assert origen.dut.pins["p3"].data == 1
  assert grp.data == 7

def test_exception_on_overflow():
  grp = origen.dut.pin("grp")
  assert grp.data == 7
  with pytest.raises(OSError):
    grp.data = 8
  with pytest.raises(OSError):
    grp.drive(8)
  with pytest.raises(OSError):
    grp.verify(8)
  with pytest.raises(OSError):
    grp.set(8)
  assert grp.data == 7

# Basically make sure we don't get a Rust panic when garbage input is given.
# Would prefer to pass the error up through Python and cleanly fail.
def test_exception_on_bad_input():
  grp = origen.dut.pin("grp")
  with pytest.raises(TypeError):
     grp.data = "hi"
  with pytest.raises(TypeError):
    grp.drive({})
  with pytest.raises(TypeError):
    grp.verify([])
  with pytest.raises(TypeError):
    grp.set(origen)

def test_driving_pins():
  # Same as above: changing the pin action sets it in the physical pin.
  # Note that you cannot set pin states directly using strings.
  grp = origen.dut.pin("grp")
  assert grp.pin_actions == "ZZZ"
  assert grp.drive() is None
  assert grp.pin_actions == "DDD"
  assert origen.dut.pins["p1"].pin_actions == "D"
  assert origen.dut.pins["p2"].pin_actions == "D"
  assert origen.dut.pins["p3"].pin_actions == "D"

def test_capturing_and_highzing_pins():
  grp = origen.dut.pin("grp")
  assert grp.pin_actions == "DDD"
  assert grp.highz() is None
  assert grp.pin_actions == "ZZZ"
  assert origen.dut.pins["p1"].pin_actions == "Z"
  assert origen.dut.pins["p2"].pin_actions == "Z"
  assert origen.dut.pins["p3"].pin_actions == "Z"

  assert grp.capture() is None
  assert grp.pin_actions == "CCC"
  assert origen.dut.pins["p1"].pin_actions == "C"
  assert origen.dut.pins["p2"].pin_actions == "C"
  assert origen.dut.pins["p3"].pin_actions == "C"

def test_driving_pins_with_data():
  grp = origen.dut.pin("grp")
  assert grp.data == 7
  assert grp.pin_actions == "CCC"
  assert grp.drive(5) is None
  assert grp.data == 5
  assert grp.pin_actions == "DDD"

def test_verifying_pins():
  grp = origen.dut.pin("grp")
  assert grp.data == 5
  assert grp.pin_actions == "DDD"
  assert grp.verify() is None
  assert grp.pin_actions == "VVV"

  assert grp.verify(0) is None
  assert grp.data == 0
  assert grp.pin_actions == "VVV"


def test_pins_in_subblocks():
  # We should have pins at the DUT level, but not in any subblocks.
  assert len(origen.dut.pins) == 4
  assert len(origen.dut.sub_blocks["core1"].pins) == 0

  # Add a pin at the subblock. Check it was added and has the correct path.
  assert origen.dut.sub_blocks["core1"].add_pin("p1")
  assert len(origen.dut.sub_blocks["core1"].pins) == 1
  p = origen.dut.sub_blocks["core1"].pin("p1")
  is_pin_group(p)
  #assert p._path == "core1"

  # Add another pin
  assert origen.dut.sub_blocks["core1"].add_pin("_p1")
  assert len(origen.dut.sub_blocks["core1"].pins) == 2
  _p = origen.dut.sub_blocks["core1"].pin("_p1")
  is_pin_group(_p)
  #assert _p._path == "core1"

  # Verify the pins at origen.dut are unchanged.
  assert len(origen.dut.pins) == 4
  assert origen.dut.pin("p1")._path == ""
  assert origen.dut.pin("_p1") is None

def test_adding_aliases():
  origen.dut.add_pin_alias("p1", "a1")
  assert len(origen.dut.pins) == 5
  assert len(origen.dut.physical_pins) == 3
  assert "a1" in origen.dut.pins
  a1 = origen.dut.pin("a1")
  is_pin_group(a1)
  assert len(a1) == 1
  assert a1.pin_names == ["p1"]

def test_driving_an_alias():
  a1 = origen.dut.pin("a1")
  p1 = origen.dut.pin("p1")
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

def test_adding_multiple_aliases():
  origen.dut.add_pin_alias("p1", "a1_a", "a1_b", "a1_c")
  check_alias("p1", "a1_a")
  check_alias("p1", "a1_b")
  check_alias("p1", "a1_c")

def test_exception_on_duplicate_aliases():
  with pytest.raises(OSError):
    origen.dut.add_pin_alias("p1", "a1")

def test_exception_when_aliasing_missing_pin():
  with pytest.raises(OSError):
    origen.dut.add_pin_alias("blah", "alias_blah")

def test_aliasing_an_alias():
  origen.dut.add_pin_alias("a1", "_a1")
  assert "_a1" in origen.dut.pins
  assert origen.dut.pins["_a1"].pin_names == ["p1"]

def test_exception_on_grouping_the_same_pin():
  with pytest.raises(OSError):
    origen.dut.group_pins("test_grouping_the_same_pin", "p1", "p1", "p1")

def test_exception_on_grouping_aliases_of_the_same_pin():
  with pytest.raises(OSError):
    origen.dut.group_pins("test_grouping_aliases_of_same_pin", "p2", "p3", "a1_a", "a1_b", "a1_c")

def test_collecting_pins():
  n = len(origen.dut.pins)
  # Create an anonymous pin group (pin collection)
  c = origen.dut.pins.collect("p1", "p2")
  assert len(origen.dut.pins) == n
  is_pin_collection(c)

def test_exception_on_collecting_missing_pins():
  with pytest.raises(OSError):
    origen.dut.pins.collect("p1", "p2", "blah")

def test_exception_on_collecting_duplicate_pins():
  with pytest.raises(OSError):
    origen.dut.pins.collect("p1", "p1", "p1")
  with pytest.raises(OSError):
    origen.dut.pins.collect("p1", "a1")
  with pytest.raises(OSError):
    origen.dut.pins.collect("a1_a", "a1_b", "a1_c")

def test_pin_collection_initial_state():
  origen.dut.pin("p1").drive(1)
  origen.dut.pin("p2").drive(0)
  origen.dut.pin("p3").drive(1)

  c = origen.dut.pins.collect("p1", "p2", "p3")
  is_pin_collection(c)
  assert c.data == 0x5
  assert c.pin_actions == "DDD"

def test_pin_collection_getting_and_setting_data():
  c = origen.dut.pins.collect("p1", "p2", "p3")
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

  with pytest.raises(OSError):
    c.data = 8
  assert c.data == 0x1

def test_pin_collectino_getting_and_setting_actions():
  c = origen.dut.pins.collect("p1", "p2", "p3")
  c.verify()
  assert c.pin_actions == "VVV"
  assert origen.dut.physical_pin("p1").action == "Verify"
  assert origen.dut.physical_pin("p2").action == "Verify"
  assert origen.dut.physical_pin("p3").action == "Verify"

  c.highz()
  assert c.pin_actions == "ZZZ"
  assert origen.dut.physical_pin("p1").action == "HighZ"
  assert origen.dut.physical_pin("p2").action == "HighZ"
  assert origen.dut.physical_pin("p3").action == "HighZ"

def test_pins_dict_like_api():
  # Check '__contains__'
  assert "p1" in origen.dut.pins

  # Check '__getitem__' (indexing)
  assert origen.dut.pins["p1"].name == "p1"

  # Check 'keys()'
  keys = origen.dut.pins.keys()
  assert isinstance(keys, list)
  assert {'a1_c', 'a1', 'a1_b', 'grp', 'p1', 'p2', '_a1', 'p3', 'a1_a'} == set(keys)

  # Check 'values()'
  values = origen.dut.pins.values()
  assert len(values) == 9
  assert isinstance(values[0], _origen.dut.pins.PinGroup)
  for name in origen.dut.pins:
    assert name in keys

  # Check 'items()'
  for n, p in origen.dut.pins.items():
    assert isinstance(n, str)
    is_pin_group(p)

  # Check __len__
  assert len(origen.dut.pins) == 9

  # Check 'to_dict' conversion
  d = dict(origen.dut.pins)
  assert isinstance(d, dict)
  assert isinstance(d["p1"], _origen.dut.pins.PinGroup)
  assert isinstance(d["p2"], _origen.dut.pins.PinGroup)
  assert len(d) == 9

def test_physical_pins_dict_like_api():
  # check __contains__
  assert "p1" in origen.dut.physical_pins

  # check __len__
  assert len(origen.dut.physical_pins) == 3

  # Check '__getitem__' (indexing)
  pin = origen.dut.physical_pins["p1"]
  is_pin(pin)

  pin_names = ["p1", "p2", "p3"]
  # Check keys()
  assert set(origen.dut.physical_pins.keys()) == set(pin_names)

  # Check values()
  for v in origen.dut.physical_pins.values():
    is_pin(v)
    assert v.name in pin_names

  # Check items()
  for n, p in origen.dut.physical_pins.items():
    assert isinstance(n, str)
    is_pin(p)

  # Check iterating
  for name in origen.dut.physical_pins:
    assert name in pin_names

  # Check 'to_dict' conversion
  d = dict(origen.dut.physical_pins)
  assert isinstance(d, dict)
  assert len(d) == 3
  assert isinstance(d["p1"], _origen.dut.pins.Pin)
  assert isinstance(d["p2"], _origen.dut.pins.Pin)

def test_pin_group_list_like_api():
  grp = origen.dut.pin("grp")

  # Check '__contains__'
  assert "p1" in grp

  # Check '__getitem__' (indexing)
  is_pin_collection(grp[0])
  assert grp[0].pin_names == ["p1"]

  # Check __len__
  assert len(grp) == 3

  # Check iterating
  # Note: Unlike the dictionary, this is ordered.
  names = [["p1"], ["p2"], ["p3"]]
  for i, name in enumerate(grp):
    is_pin_collection(name)
    assert name.pin_names == names[i]

  # Check 'to_list'
  as_list = list(grp)
  assert isinstance(as_list, list)
  for i, item in enumerate(as_list):
    is_pin_collection(item)
    assert item.pin_names == names[i]

def test_pin_collection_from_pin_group():
  origen.dut.add_pin("p0")
  origen.dut.group_pins("p", "p0", "p1", "p2", "p3")
  grp = origen.dut.pins["p"]

  assert grp[0].pin_names == ["p0"]
  assert grp[1].pin_names == ["p1"]
  assert grp[0:1].pin_names == ["p0", "p1"]
  assert grp[1:3:2].pin_names == ["p1", "p3"]

def test_pin_collection_list_like_api():
  c = origen.dut.pins.collect("p0", "p1", "p2", "p3")
  is_pin_collection(c)
  
  # Check __contains__
  assert "p0" in c
  assert "p3" in c

  # Check __getitem__ (indexing)
  assert c[0].pin_names == ["p0"]
  assert c[1].pin_names == ["p1"]

  # Check Slicing
  assert c[0:1].pin_names == ["p0", "p1"]
  assert c[1:3:2].pin_names == ["p1", "p3"]

  # Check __len__
  assert len(c) == 4

  # Check iterating
  names = ["p0", "p1", "p2", "p3"]
  for i, _c in enumerate(c):
    is_pin_collection(_c)
    _c.pin_names == list(names[i])

def test_exception_on_out_of_bounds_indexing():
  # Covers both pin collection and pin groups
  grp = origen.dut.pin("grp")
  with pytest.raises(OSError):
    grp[100]
  with pytest.raises(OSError):
    grp[0:100]

  c = origen.dut.pins.collect("p1", "p2", "p3")
  with pytest.raises(OSError):
    c[100]
  with pytest.raises(OSError):
    c[0:100]

def test_chaining_method_calls_with_nonsticky_mask():
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

# def test_chaining_method_calls_with_sticky_mask():
#   ...

### Get a clean DUT here ###

def test_reseting_DUT():
  origen.app.instantiate_dut("dut.falcon")
  # Ensure the DUT was set and the pin container is available.
  assert origen.dut
  assert isinstance(origen.dut.pins, _origen.dut.pins.PinContainer)
  assert len(origen.dut.pins) == 0
  assert len(origen.dut.physical_pins) == 0

def test_adding_multiple_pins():
  # !!!
  # This should add two physical pins and one pin group, containing both physical pins.
  # !!!
  porta = origen.dut.add_pin("porta", width=2)
  is_pin_group(porta)
  assert porta.name == "porta"
  assert len(porta) == 2
  assert len(origen.dut.physical_pins) == 2
  assert len(origen.dut.pins) == 3
  assert set(origen.dut.physical_pins.pin_names) == {"porta0", "porta1"}
  assert set(origen.dut.pins.pin_names) == {"porta0", "porta1", "porta"}

  # This should add four physical pins and one pin group, but the indexing should start a 1 instead of 0.
  portb = origen.dut.add_pin("portb", width=4, offset=1)
  is_pin_group(portb)
  assert portb.name == "portb"
  assert len(portb) == 4
  assert len(origen.dut.physical_pins) == 6
  assert len(origen.dut.pins) == 8
  assert set(origen.dut.physical_pins.pin_names) == {"porta0", "porta1", "portb1", "portb2", "portb3", "portb4"}
  assert set(origen.dut.pins.pin_names) == {"porta0", "porta1", "porta", "portb1", "portb2", "portb3", "portb4", "portb"}

def test_adding_single_pins_with_width_and_offset():
  # Corner case for adding single pins. Normally, you'd not use these options for a single one, but there's no reason why
  # it should work any differently.
  origen.dut.add_pin("portc", width=1, offset=1)
  assert len(origen.dut.physical_pins) == 7
  assert len(origen.dut.pins) == 10
  assert "portc1" in origen.dut.physical_pins
  assert "portc" in origen.dut.pins
  assert "portc1" in origen.dut.pins

  origen.dut.add_pin("portd", width=1)
  assert len(origen.dut.physical_pins) == 8
  assert len(origen.dut.pins) == 12
  assert "portd0" in origen.dut.physical_pins
  assert "portd" in origen.dut.pins
  assert "portd0" in origen.dut.pins

### Note: pyo3 will throw an overflow error on negative numbers.

def test_exception_on_invalid_width():
  assert len(origen.dut.physical_pins) == 8
  assert len(origen.dut.pins) == 12
  with pytest.raises(OSError):
    origen.dut.add_pin("porte", width=0)
  with pytest.raises(OverflowError):
    origen.dut.add_pin("porte", width=-1)
  assert len(origen.dut.physical_pins) == 8
  assert len(origen.dut.pins) == 12

def test_exception_on_invalid_offset():
  assert len(origen.dut.physical_pins) == 8
  assert len(origen.dut.pins) == 12
  with pytest.raises(OverflowError):
    origen.dut.add_pin("porte", width=1, offset=-1)
  assert len(origen.dut.physical_pins) == 8
  assert len(origen.dut.pins) == 12

def test_exception_on_offset_without_width():
  assert len(origen.dut.physical_pins) == 8
  assert len(origen.dut.pins) == 12
  with pytest.raises(OSError):
    origen.dut.add_pin("porte", offset=1)
  assert len(origen.dut.physical_pins) == 8
  assert len(origen.dut.pins) == 12

# !!!
# Reseting DUT
# !!!

def test_length_vs_width():
  origen.app.instantiate_dut("dut.falcon")
  assert origen.dut
  origen.dut.add_pin("porta", width=2)
  origen.dut.add_pin("portb", width=4)
  assert len(origen.dut.physical_pins) == 6
  assert len(origen.dut.pins) == 8
  grp = origen.dut.group_pins("ports", "porta", "portb")
  assert len(grp) == 6
  assert grp.width == 6

  c = origen.dut.pins.collect("porta", "portb")
  assert len(c) == 6
  assert c.width == 6

def test_getting_nested_pin_data():
  grp = origen.dut.pin("ports")
  c = origen.dut.pins.collect("porta", "portb")

  assert grp.data == 0
  assert c.data == 0

def test_getting_nested_pin_actions():
  grp = origen.dut.pin("ports")
  c = origen.dut.pins.collect("porta", "portb")

  assert grp.pin_actions == "ZZZZZZ"
  assert c.pin_actions == "ZZZZZZ"

def test_setting_nested_pin_data():
  grp = origen.dut.pin("ports")
  c = origen.dut.pins.collect("porta", "portb")

  grp.data = 0x2A
  assert grp.data == 0x2A
  assert c.data == 0x2A
  c.data = 0x15
  assert c.data == 0x15
  assert grp.data == 0x15

def test_setting_nested_pin_actions():
  grp = origen.dut.pin("ports")
  c = origen.dut.pins.collect("porta", "portb")

  grp.capture()
  assert grp.pin_actions == "CCCCCC"
  assert c.pin_actions == "CCCCCC"
  c.verify()
  assert c.pin_actions == "VVVVVV"
  assert grp.pin_actions == "VVVVVV"

def test_exception_on_grouping_duplicates_when_nested():
  assert len(origen.dut.physical_pins) == 6
  assert len(origen.dut.pins) == 9
  assert "porta" in origen.dut.pins
  assert "porta0" in origen.dut.pins
  assert "porta1" in origen.dut.pins
  with pytest.raises(OSError):
    origen.dut.group_pins("grouping_porta", "porta", "porta0", "porta1")
  assert len(origen.dut.physical_pins) == 6
  assert len(origen.dut.pins) == 9

def test_exception_on_collecting_duplicates_when_nested():
  assert "porta" in origen.dut.pins
  assert "porta0" in origen.dut.pins
  assert "porta1" in origen.dut.pins
  with pytest.raises(OSError):
    origen.dut.pins.collect("collecting_porta", "porta", "porta0", "porta1")

def test_exception_on_collecting_duplicates_when_nested_2():
  origen.dut.group_pins("index_0s", "porta0", "portb0")
  origen.dut.group_pins("index_0s_rev", "portb0", "porta0")
  assert "error" not in origen.dut.pins
  with pytest.raises(OSError):
    origen.dut.pins.collect("error", "index_0s", "index_0s_rev")
  assert "error" not in origen.dut.pins

def test_big_endian():
  pins = origen.dut.add_pin("portc", width=4, little_endian=False)
  assert pins.pin_names == ["portc3", "portc2", "portc1", "portc0"]
  assert not pins.little_endian
  assert pins.big_endian

def test_little_endian():
  pins = origen.dut.add_pin("portd", width=4, little_endian=True)
  assert pins.pin_names == ["portd0", "portd1", "portd2", "portd3"]
  assert pins.little_endian
  assert not pins.big_endian

def test_big_endian_with_offset():
  pins = origen.dut.add_pin("porte", width=2, little_endian=False, offset=2)
  assert pins.pin_names == ["porte3", "porte2"]
  assert not pins.little_endian
  assert pins.big_endian

def test_grouping_mixed_endianness():
  grp = origen.dut.group_pins("mixed_endianness", "portc", "portd")
  assert grp.pin_names == ["portc3", "portc2", "portc1", "portc0", "portd0", "portd1", "portd2", "portd3"]
  assert grp.little_endian == True
  assert grp.big_endian == False

def test_grouping_big_endian():
  grp = origen.dut.group_pins("big_endian", "portc", "portd", little_endian=False)
  assert grp.pin_names == ["portd3", "portd2", "portd1", "portd0", "portc0", "portc1", "portc2", "portc3"]
  assert grp.little_endian == False
  assert grp.big_endian == True

def test_collecting_mixed_endianness():
  c = origen.dut.pins.collect("portc", "portd")
  assert c.pin_names == ["portc3", "portc2", "portc1", "portc0", "portd0", "portd1", "portd2", "portd3"]
  assert c.little_endian == True
  assert c.big_endian == False

def test_collecting_big_endian():
  c = origen.dut.pins.collect("portc", "portd", little_endian=False)
  assert c.pin_names == ["portd3", "portd2", "portd1", "portd0", "portc0", "portc1", "portc2", "portc3"]
  assert c.little_endian == False
  assert c.big_endian == True

# # !!!
# # RESET DUT
# # !!!

def test_default_reset_values():
  origen.app.instantiate_dut("dut.falcon")
  assert origen.dut
  origen.dut.add_pin("p0")
  assert origen.dut.physical_pin("p0").reset_data == None
  assert origen.dut.physical_pin("p0").reset_action == None
  assert origen.dut.pin("p0").reset_data == 0
  assert origen.dut.pin("p0").reset_actions == 'Z'

def test_default_reset_values_with_width():
  origen.dut.add_pin("porta", width=2)
  assert origen.dut.physical_pin("porta0").reset_data == None
  assert origen.dut.physical_pin("porta0").reset_action == None
  assert origen.dut.physical_pin("porta1").reset_data == None
  assert origen.dut.physical_pin("porta1").reset_action == None
  assert origen.dut.pin("porta").reset_data == 0
  assert origen.dut.pin("porta").reset_actions == "ZZ"

def test_adding_with_reset_values():
  origen.dut.add_pin("p1", reset_data=1, reset_action="D")
  assert origen.dut.physical_pin("p1").reset_data == 1
  assert origen.dut.physical_pin("p1").reset_action == "D"
  assert origen.dut.pin("p1").reset_data == 1
  assert origen.dut.pin("p1").reset_actions == "D"

def test_adding_with_reset_values_with_width():
  origen.dut.add_pin("portb", width=3, reset_data=0b101, reset_action="VDV")
  assert origen.dut.pin("portb").reset_data == 0b101
  assert origen.dut.pin("portb").reset_actions == "VDV"

def test_reset_values_persist_in_groups():
  grp = origen.dut.group_pins("grp1", "p1", "p0", "porta", "portb")
  assert grp.data == 0b101_00_0_1
  assert grp.pin_actions == "DZZZVDV"
  assert grp.reset_data == 0b101_00_0_1
  assert grp.reset_actions == "DZZZVDV"

def test_reset_values_persist_in_collections():
  c = origen.dut.pins.collect("p1", "p0", "porta", "portb")
  assert c.data == 0b101_00_0_1
  assert c.pin_actions == "DZZZVDV"
  assert c.reset_data == 0b101_00_0_1
  assert c.reset_actions == "DZZZVDV"

def test_resetting_pins():
  pins = origen.dut.pin("portb")
  pins.set(0b111)
  pins.drive()
  assert pins.data == 0b111
  assert pins.pin_actions == "DDD"
  pins.reset()
  assert pins.data == 0b101
  assert pins.pin_actions == "VDV"

def test_resetting_collection():
  c = origen.dut.pins.collect("portb", "p1")
  c.set(0b0000)
  c.capture()
  assert c.data == 0
  assert c.pin_actions == "CCCC"
  c.reset()
  assert c.data == 0b1101
  assert c.pin_actions == "VDVD"

def test_exception_on_invalid_reset_data():
  with pytest.raises(OSError):
    origen.dut.add_pin("error", width=2, reset_data=0b111)

def test_exception_on_invalid_reset_actions():
  with pytest.raises(OSError):
    origen.dut.add_pin("error", width=2, reset_action="ZZZ")
  with pytest.raises(OSError):
    origen.dut.add_pin("error", width=2, reset_action="Z")

def test_collecting_with_single_regex():
  import re
  r = re.compile("port.0")
  c = origen.dut.pins.collect(r)
  assert c.pin_names == ["porta0", "portb0"]

def test_collecting_using_ruby_like_syntax():
  c = origen.dut.pins.collect("/port.0/")
  assert c.pin_names == ["porta0", "portb0"]

def test_collecting_with_mixed_inputs():
  import re
  r = re.compile("port.0")
  c = origen.dut.pins.collect("/port.1/", "p1", r)
  assert c.pin_names == ["porta1", "portb1", "p1", "porta0", "portb0"]

def test_physical_pin_has_empty_metadata():
  assert origen.dut.physical_pin("porta0").added_metadata == []

def test_adding_metadata_to_physical_pin():
  # Essentially just check that nothing here throws an exception
  origen.dut.physical_pin("porta0").add_metadata("meta1", 1)
  origen.dut.physical_pin("porta0").add_metadata("meta2", "meta2!")
  origen.dut.physical_pin("porta0").add_metadata("meta3", {})
  origen.dut.physical_pin("porta0").add_metadata("meta4", MyRandomClass())

def test_getting_all_metadata_keys():
  assert origen.dut.physical_pin("porta0").added_metadata == ["meta1", "meta2", "meta3", "meta4"]

def test_getting_metadata_from_physical_pin():
  assert origen.dut.physical_pin("porta0").get_metadata("meta1") == 1
  assert origen.dut.physical_pin("porta0").get_metadata("meta2") == "meta2!"
  assert isinstance(origen.dut.physical_pin("porta0").get_metadata("meta3"), dict)
  assert isinstance(origen.dut.physical_pin("porta0").get_metadata("meta4"), MyRandomClass)

def test_setting_existing_metadata_on_physical_pin():
  assert origen.dut.physical_pin("porta0").set_metadata("meta1", "hi!")
  assert origen.dut.physical_pin("porta0").set_metadata("meta2", "meta2 updated!")
  assert origen.dut.physical_pin("porta0").get_metadata("meta1") == "hi!"
  assert origen.dut.physical_pin("porta0").get_metadata("meta2") == "meta2 updated!"

def test_setting_nonexistant_metadata_adds_it():
  assert origen.dut.physical_pin('porta0').get_metadata("meta5") is None
  assert origen.dut.physical_pin("porta0").set_metadata("meta5", 5.0) == False
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
