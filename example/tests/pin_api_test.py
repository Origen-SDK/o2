import origen # pylint: disable=import-error
import pytest

def test_pin_api():
  origen.app.instantiate_dut("dut.falcon")

  # Ensure the DUT was set
  assert origen.dut

  # Check the initial pin states
  assert origen.dut.pins == {}

  # Add a single pin and check its available on the model
  # Side note: the pin class should be cached, so all retrievals
  #  of the same pin should point to the same cached instance.
  p = origen.dut.add_pin("test_pin")
  assert isinstance(p, origen.pins.Pin)
  assert len(origen.dut.pins) == 1
  p2 = origen.dut.pins['test_pin']
  assert isinstance(p2, origen.pins.Pin)
  p3 = origen.dut.pin('test_pin')
  assert isinstance(p3, origen.pins.Pin)
  assert p == p2 == p3
  assert origen.dut.has_pin('test_pin')

  # Check the state of the pin itself
  # This should be the default state
  assert p.name == 'test_pin'
  assert p.postured_state == False
  assert p.action == "HighZ"

  # Add another pin
  p = origen.dut.add_pin("other_pin")
  assert isinstance(p, origen.pins.Pin)
  assert len(origen.dut.pins) == 2
  assert p == origen.dut.pins['other_pin']
  assert p == origen.dut.pin('other_pin')
  assert origen.dut.has_pin('other_pin')

  # Attempt to posture the pin.
  assert p.postured_state == False
  assert p.data == 0
  p.posture(True)
  assert p.postured_state == True
  assert p.data == 1
  p.posture(False)
  assert p.postured_state == False
  assert p.data == 0
  p.posture(1)
  assert p.postured_state == True
  assert p.data == 1
  p.posture(0)
  assert p.postured_state == False
  assert p.data == 0

  # Drive/Verify/Capture Pins
  assert p.action == "HighZ"
  p.drive()
  assert p.action == "Drive"
  p.verify()
  assert p.action == "Verify"
  p.capture()
  assert p.action == "Capture"
  p.highz()
  assert p.action == "HighZ"

  # Some cases to ensure errors and bad input are handled.
  # Access an unknown pin
  assert origen.dut.has_pin('blah') == False
  assert origen.dut.pin('blah') == None
  with pytest.raises(KeyError):
    origen.dut.pins['blah']

  # Some developer-side errors that should return Python exceptions instead of Rust panicking
  with pytest.raises(OSError):
    origen.dut.__proxies__["pins"].__pin__("blah")
  with pytest.raises(OSError):
    origen.dut.__proxies__["pins"].__update_pin__("blah", blah="blah")
  with pytest.raises(OSError):
    origen.dut.__proxies__["pins"].__update_pin__("test_pin", blah="blah")

  # Check that pins are available on subblocks
  assert len(origen.dut.pins) == 2
  assert origen.dut.sub_blocks["core1"]
  assert origen.dut.sub_blocks["core1"].pins == {}
  assert origen.dut.sub_blocks["core1"].add_pin("test_pin_core1")
  assert len(origen.dut.sub_blocks["core1"].pins) == 1
  assert len(origen.dut.pins) == 2

  # Add a pin alias
  p = origen.dut.pin('test_pin')
  p.add_alias('test_alias')
  assert origen.dut.has_pin('test_alias')
  assert p == origen.dut.pin('test_alias')
  assert len(origen.dut.pins) == 3

  ### In Progress ###

  # Check initial state of pin groups
  # assert origen.dut.pin_groups == {}
  # assert len(origen.dut.pin_groups) == 0
  # assert origen.dut.has_pin_group('grp') == False
  # assert origen.dut.pin_group('grp') == None

  # Add a pin group.
  # p = origen.dut.pin('test_pin')
  # grp = origen.dut.group_pins("grp", "test_pin", "other_pin")
  # assert len(p.groups) == 0
  # assert len(origen.dut.pin_groups) == 1
  # assert origen.dut.has_pin_group('grp') == True
  # assert isinstance(origen.dut.pin_group('grp')) == origen.pins.PinCollection
  # assert len(p.groups) == 1
  # assert p.groups['grp'] == grp
  # assert p.group('grp') == grp
  # assert p.group('grp2') == None

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