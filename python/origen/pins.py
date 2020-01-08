import origen
from origen.errors import *

class Proxy:
  def __init__(self, controller):
    self.controller = controller

  @property
  def pins(self):
    return origen.dut.db.pins(self.controller.model_id)
  
  @property
  def physical_pins(self):
    return origen.dut.db.physical_pins(self.controller.model_id)

  def add_pin(self, name, **kwargs):
    return origen.dut.db.add_pin(self.controller.model_id, name, **kwargs)
  
  def pin(self, name):
    return origen.dut.db.pin(self.controller.model_id, name)
  
  def add_pin_alias(self, name, *aliases):
    return origen.dut.db.add_pin_alias(self.controller.model_id, name, *aliases)
  
  def group_pins(self, name, *pin_names, **options):
    return origen.dut.db.group_pins(self.controller.model_id, name, *pin_names, **options)
  
  def physical_pin(self, name):
    return origen.dut.db.physical_pin(self.controller.model_id, name)

  @classmethod
  def api(cls):
    return [
      'pins',
      'add_pin',
      'pin',
      'add_pin_alias',
      'group_pins',
      'physical_pin',
      'physical_pins',
    ]
  
class Loader:
  def __init__(self, controller):
    self.controller = controller

  def Pin(self, name, **kwargs):
    self.controller.add_pin(name, **kwargs)

  def Alias(self, name, *aliases):
    self.controller.add_pin_alias(name, *aliases)

  def api(self):
      return {
          "Pin": self.Pin,
          "Alias": self.Alias,
      }
