import origen

def test_pin_api():
  origen.app.instantiate_dut("dut.falcon")

  # Ensure the DUT was set
  assert origen.dut

  # Check the initial pin states
  assert origen.dut.pins() == {}

  # Add a single pin and check its available on the model
  # Side note: the pin class should be cached, so all retrievals
  #  of the same pin should point to the same cached instance.
  p = origen.dut.add_pin("test_pin")
  assert isinstance(p, origen.pins.Pin)
  assert len(origen.dut.pins()) == 1
  p2 = origen.dut.pins()['test_pin']
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
  assert len(origen.dut.pins()) == 2
  assert p == origen.dut.pins()['other_pin']
  assert p == origen.dut.pin('other_pin')
  assert origen.dut.has_pin('other_pin')
