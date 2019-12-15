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
  assert p.name == 'test_pin'
  assert p._path == ''
  assert p.data == 0
  assert p.action == "HighZ"

  # Add another pin
  p = origen.dut.add_pin("other_pin")
  assert isinstance(p, _origen.dut.pins.Pin)
  assert len(origen.dut.pins) == 2
  assert p.name == origen.dut.pins['other_pin'].name
  assert p.name == origen.dut.pin('other_pin').name
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
  assert alias_p.name == p.name
  assert len(origen.dut.pins) == 2

  # Add multiple aliases at once.
  origen.dut.add_pin_alias('test_pin', 'test_alias2', 'test_alias3')
  assert len(origen.dut.pins) == 2
  assert origen.dut.pin('test_alias2').name == 'test_pin'
  assert origen.dut.pin('test_alias3').name == 'test_pin'

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
    # Ensure we cannot add a pin whose name conflicts with an existing alias.
    origen.dut.add_pin("test_alias1")

  # Check some of the dict-like API
  assert "test_pin" in origen.dut.pins
  assert "test_alias1" in origen.dut.pins
  keys = origen.dut.pins.keys()
  values = origen.dut.pins.values()
  assert set(keys) == set(['test_pin', 'other_pin'])
  assert len(values) == 2
  assert isinstance(values[0], _origen.dut.pins.Pin)
  for name in origen.dut.pins:
    assert name in keys
  d = dict(origen.dut.pins)
  assert isinstance(d, dict)
  assert isinstance(d["test_pin"], _origen.dut.pins.Pin)
  assert isinstance(d["other_pin"], _origen.dut.pins.Pin)
  assert len(d) == 2
  for n, p in origen.dut.pins.items():
    assert isinstance(n, str)
    assert isinstance(p, _origen.dut.pins.Pin)
  #for (n, )

  ### In Progress ###

  # Check initial state of pin groups
  assert isinstance(origen.dut.pin_groups, _origen.dut.pins.PinGroupContainer)
  assert len(origen.dut.pin_groups) == 0
  assert origen.dut.pin_group('test_grp') is None
  with pytest.raises(KeyError):
    origen.dut.pin_groups['test_grp']

  # Add a pin group.
  grp = origen.dut.group_pins("test_grp", "test_pin", "other_pin")
  assert isinstance(grp, _origen.dut.pins.PinGroup)
  assert len(grp) == 2

  assert len(origen.dut.pin_groups) == 1
  # assert origen.dut.has_pin_group('grp') == True
  assert isinstance(origen.dut.pin_group('test_grp'), _origen.dut.pins.PinGroup)
  assert isinstance(origen.dut.pin_groups['test_grp'], _origen.dut.pins.PinGroup)

  # Check the initial state of the pin collection.
  # This should match the pins current state.

  #grp2 = origen.dut.group_pins("grp2", "test_alias", "other_pin")
  #assert len(origen.dut.pin_groups) == 2
  #assert origen.dut.has_pin_group('grp2') == True
  #assert isinstance(origen.dut.pin_group('grp2')) == origen.pins.PinCollection

  # Error: Pin group with missing pin
  # Error: Pin group where an alias is used.

  # Add pin group. This should add porta0 - porta7
  #porta = origen.dut.add_pins("porta", 8)
  
  # Add pin group, with an offset. This should add portb1 - portb5
  #portb = origen.dut.add_pins("portb", 4, offset=1)