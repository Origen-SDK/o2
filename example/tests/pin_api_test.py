import origen # pylint: disable=import-error
import _origen # pylint: disable=import-error
import pytest

def test_pin_api():
  origen.app.instantiate_dut("dut.falcon")

  # Ensure the DUT was set
  assert origen.dut

  # Check the initial pin states
  assert isinstance(origen.dut.pins, _origen.dut.pins.PinContainer)

  # Add a single pin and check its available on the model
  # Side note: the pin class should be cached, so all retrievals
  #  of the same pin should point to the same cached instance.
  assert origen.dut.pin("test_pin") is None
  p = origen.dut.add_pin("test_pin")
  assert isinstance(p, _origen.dut.pins.Pin)
  assert len(origen.dut.pins) == 1
  p2 = origen.dut.pins['test_pin']
  assert isinstance(p2, _origen.dut.pins.Pin)
  p3 = origen.dut.pin('test_pin')
  assert isinstance(p3, _origen.dut.pins.Pin)
  #assert 'test_pin' in origen.dut.pins # origen.dut.has_pin('test_pin')

  # Check the state of the pin itself
  # This should be the default state
  assert p.id == 'test_pin'
  assert p._path == ''
  assert p.data == 0
  assert p.action == "HighZ"

  # Add another pin
  p = origen.dut.add_pin("other_pin")
  assert isinstance(p, _origen.dut.pins.Pin)
  assert len(origen.dut.pins) == 2
  assert p.id == origen.dut.pins['other_pin'].id
  assert p.id == origen.dut.pin('other_pin').id
  #assert origen.dut.has_pin('other_pin')

  # Attempt to posture the pin.
  assert p.data == 0
  p.data = 1
  assert p.data == 1
  p.set(0)
  assert p.data == 0

  # Drive/Verify/Capture Pins
  assert p.action == "HighZ"
  p.drive()
  assert p.data == 0
  assert p.action == "Drive"
  p.verify()
  assert p.data == 0
  assert p.action == "Verify"
  p.capture()
  assert p.data == 0
  assert p.action == "Capture"
  p.highz()
  assert p.data == 0
  assert p.action == "HighZ"

  p.drive(1)
  assert p.data == 1
  assert p.action == "Drive"
  p.verify(0)
  assert p.data == 0
  assert p.action == "Verify"
  p.capture()
  assert p.data == 0
  assert p.action == "Capture"
  p.highz()
  assert p.action == "HighZ"

  # Some cases to ensure errors and bad input are handled.
  # Access an unknown pin
  with pytest.raises(KeyError):
    origen.dut.pins['blah']
  with pytest.raises(OSError):
    # Ensure we cannot add the same pin over again.
    origen.dut.add_pin("test_pin")
  with pytest.raises(OSError):
    p.data = 2
  with pytest.raises(OSError):
    p.drive(2)
  with pytest.raises(OSError):
    p.verify(2)
  with pytest.raises(OSError):
    p.set(2)
  assert p.action == "HighZ"

  # Check that pins are available on subblocks
  assert len(origen.dut.pins) == 2
  assert origen.dut.sub_blocks["core1"]
  assert len(origen.dut.sub_blocks["core1"].pins) == 0
  assert origen.dut.sub_blocks["core1"].add_pin("test_pin_core1")
  assert len(origen.dut.sub_blocks["core1"].pins) == 1
  p = origen.dut.sub_blocks["core1"].pin("test_pin_core1")
  assert p._path == "core1"
  assert len(origen.dut.pins) == 2

  # Add a pin alias
  p = origen.dut.pin('test_pin')
  #p.add_alias('test_alias')
  origen.dut.add_pin_alias('test_pin', 'test_alias1')
  alias_p = origen.dut.pin('test_alias1')
  assert alias_p.id == p.id
  assert len(origen.dut.pins) == 2

  # Add multiple aliases at once.
  origen.dut.add_pin_alias('test_pin', 'test_alias2', 'test_alias3')
  assert len(origen.dut.pins) == 2
  assert origen.dut.pin('test_alias2').id == 'test_pin'
  assert origen.dut.pin('test_alias3').id == 'test_pin'

  # Error conditions related to aliasing
  with pytest.raises(OSError):
    # Pin doesn't exists
    origen.dut.add_pin_alias("blah", "test_blah")
  with pytest.raises(OSError):
    # alias already exists as a pin
    origen.dut.add_pin_alias("test_pin", "test_pin")
  with pytest.raises(OSError):
    # Alias already exists
    origen.dut.add_pin_alias("test_pin", "test_alias1")
  with pytest.raises(OSError):
    # Ensure we cannot add a pin whose id conflicts with an existing alias.
    origen.dut.add_pin("test_alias1")

  # Check some of the dict-like API
  assert "test_pin" in origen.dut.pins
  assert "test_alias1" in origen.dut.pins
  keys = origen.dut.pins.keys()
  values = origen.dut.pins.values()
  assert set(keys) == set(['test_pin', 'other_pin'])
  assert len(values) == 2
  assert isinstance(values[0], _origen.dut.pins.Pin)
  for id in origen.dut.pins:
    assert id in keys
  d = dict(origen.dut.pins)
  assert isinstance(d, dict)
  assert isinstance(d["test_pin"], _origen.dut.pins.Pin)
  assert isinstance(d["other_pin"], _origen.dut.pins.Pin)
  assert len(d) == 2
  for n, p in origen.dut.pins.items():
    assert isinstance(n, str)
    assert isinstance(p, _origen.dut.pins.Pin)

  # Check initial state of pin groups
  assert isinstance(origen.dut.pin_groups, _origen.dut.pins.PinGroupContainer)
  assert len(origen.dut.pin_groups) == 0
  assert origen.dut.pin_group('test_grp') is None
  with pytest.raises(KeyError):
    origen.dut.pin_groups['test_grp']

  # Add a pin group.
  grp = origen.dut.group_pins("test_grp", "test_pin", "other_pin")
  assert isinstance(grp, _origen.dut.pins.PinGroup)

  assert len(origen.dut.pin_groups) == 1
  assert 'test_grp' in origen.dut.pin_groups
  assert isinstance(origen.dut.pin_group('test_grp'), _origen.dut.pins.PinGroup)
  assert isinstance(origen.dut.pin_groups['test_grp'], _origen.dut.pins.PinGroup)

  origen.dut.group_pins("test_grp2", "test_alias", "other_pin")
  assert len(origen.dut.pin_groups) == 2
  assert 'test_grp' in origen.dut.pin_groups
  assert isinstance(origen.dut.pin_group('test_grp2'), _origen.dut.pins.PinGroup)

  # Add a pin group alias
  # origen.dut.add_pin_group_alias("test_grp", "alias")
  # assert len(origen.dut.pin_groups) == 2
  # assert 'alias' in origen.dut.pin_groups
  # assert isinstance(origen.dut.pin_group('alias'), _origen.dut.pins.PinGroup)
  # assert isinstance(origen.dut.pin_groups['alias'], _origen.dut.pins.PinGroup)

  # Check the pin group
  assert grp.id == "test_grp"
  assert grp._path == ""
  assert len(grp) == 2
  assert grp.pin_ids == ['test_pin', 'other_pin']
  assert grp.big_endian == False
  assert grp.little_endian == True
  assert 'test_pin' in grp
  assert 'blah' not in grp
  ids = ['test_pin', 'other_pin']
  # for i, pin in enumerate(grp):
  #   assert isinstance(pin, _origen.dut.pins.Pin)
  #   assert pin.id == id[i]
  assert grp.data == 0
  grp.data = 1
  assert grp.data == 1

  # Test setting actions
  assert grp.pin_actions == "ZZ"
  assert origen.dut.pins["test_pin"].action == "HighZ"
  assert origen.dut.pins["other_pin"].action == "HighZ"

  grp.drive()
  assert grp.pin_actions == "DD"
  assert origen.dut.pins["test_pin"].action == "Drive"
  assert origen.dut.pins["other_pin"].action == "Drive"

  grp.verify()
  assert grp.pin_actions == "VV"
  assert origen.dut.pins["test_pin"].action == "Verify"
  assert origen.dut.pins["other_pin"].action == "Verify"

  grp.capture()
  assert grp.pin_actions == "CC"
  assert origen.dut.pins["test_pin"].action == "Capture"
  assert origen.dut.pins["other_pin"].action == "Capture"

  grp.highz()
  assert grp.pin_actions == "ZZ"
  assert origen.dut.pins["test_pin"].action == "HighZ"
  assert origen.dut.pins["other_pin"].action == "HighZ"

  origen.dut.pins["test_pin"].capture()
  origen.dut.pins["other_pin"].verify()
  assert grp.pin_actions == "CV"

  # Create an anonymous pin group (pin collection)
  collection = origen.dut.pins.collect("test_pin", "other_pin")
  assert isinstance(collection, _origen.dut.pins.PinCollection)
  assert len(collection) == 2
  assert collection.ids == ["test_pin", "other_pin"]
  collection.data = 0b10
  assert collection.data == 0b10
  collection.drive()
  assert collection.pin_actions == "DD"
  collection.verify()
  assert collection.pin_actions == "VV"
  collection.capture()
  assert collection.pin_actions == "CC"
  collection.highz()
  assert collection.pin_actions == "ZZ"

  # with pytest.raises(OSError):
  #   # Pin group with missing pin
  #   origen.dut.group_pins['error', 'test_pin', 'blah']
  # with pytest.raises(OSError):
  #   # Pin group where a pin and its alias are used.
  #   origen.dut.group_pins['error', 'test_alias1', 'test_alias2']

  # Error: Pin group where an alias is used.

  # Add pin group. This should add porta0 - porta7
  #porta = origen.dut.add_pins("porta", 8)
  
  # Add pin group, with an offset. This should add portb1 - portb5
  #portb = origen.dut.add_pins("portb", 4, offset=1)

  origen.dut.add_pin("p0")
  origen.dut.add_pin("p1")
  origen.dut.add_pin("p2")
  origen.dut.add_pin("p3")
  origen.dut.group_pins("p", "p0", "p1", "p2", "p3")
  grp = origen.dut.pin_groups["p"]
  assert grp[0] == "p0"
  assert grp[1] == "p1"
  assert grp[0:1] == ["p0", "p1"]
  assert grp[1:3:2] == ["p1", "p3"]