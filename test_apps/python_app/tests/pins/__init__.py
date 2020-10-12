import origen, _origen, pytest # pylint: disable=import-error

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
  #assert grp.data == 0
  assert grp.actions == "ZZZ"

@pytest.fixture
def clk_pins():
  origen.dut.add_pin("clk")
