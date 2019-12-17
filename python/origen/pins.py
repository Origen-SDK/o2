import origen

class Proxy:
  def __init__(self, controller):
    self.controller = controller

  @property
  def pins(self):
    return origen.dut.db.pins(self.controller.path)
  
  def add_pin(self, name):
    return origen.dut.db.add_pin(self.controller.path, name)
  
  def pin(self, name):
    return origen.dut.db.pin(self.controller.path, name)
  
  def add_pin_alias(self, name, *aliases):
    return origen.dut.db.add_pin_alias(self.controller.path, name, *aliases)
  
  @property
  def pin_groups(self):
    return origen.dut.db.pin_groups(self.controller.path)

  def group_pins(self, name, *pin_names):
    return origen.dut.db.group_pins(self.controller.path, name, *pin_names)

  def pin_group(self, name):
    return origen.dut.db.pin_group(self.controller.path, name)

  @classmethod
  def api(cls):
    return [
      'pins',
      'add_pin',
      'pin',
      'add_pin_alias',
      'pin_groups',
      'group_pins',
      'pin_group',
      #'add_pin_group_alias',
    ]
  
class Loader:
  def __init__(self, controller):
    self.controller = controller