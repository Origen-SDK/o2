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
