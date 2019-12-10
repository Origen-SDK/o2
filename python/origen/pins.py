import _origen

class Proxy:
  def __init__(self, controller):
    self.controller = controller
    self.__pin_container__ = _origen.model.pins.PinContainer()
    self.__cache__ = {}

  # Add a pin to the current model
  def add_pin(self, name, **kwargs):
    self.__pin_container__.add_pin(name, **kwargs)
    self.__cache__[name] = Pin(name, self)
    return self.pin(name)

  # Retrieve a pin, returning a origen.pins.Pin object.
  def pin(self, name):
    if name not in self.__cache__:
      self.__cache__[name] = self.__pin_container__.pin_fields_for(name)
    return self.__cache__[name]
  
  def __pin__(self, name):
    return self.__pin_container__.pin_fields_for(name)

  # Return a dictionary of all pin names and their respective Pin object.
  #def pins(self, *filters): <- eventually include some ways to filter the result pins,
  # such as by role, type, or regex matching the pin name/alias.
  def pins(self):
    #_pins = self.__pin_container__.unique_pins()
    #return dict([(_name, Pin(_name, self)) for _name in self.__pin_container__.unique_pins()])
    #return dict([(_name, Pin(_name, self)) for _name in self.__cache__.items()])
    return self.__cache__

  # Return all the available pins, including aliases.
  # Essentially, anything here will return a valid pin(n) result.
  def available_pins(self):
    return self.__pin_container__.available_pins()

  # Return a boolean indicating whether or not the pin exists.
  # If a role is given, the result will based on that as well.
  #def has_pin(self, name, *, role=None):
  def has_pin(self, name):
    return (True if name in self.__cache__ else False)

  @classmethod
  def api(cls):
    return [
      'add_pin',
      'pin',
      'pins',
      'available_pins',
      'has_pin',
    ]

class Loader:
  def api(self):
    return Proxy.api()

# Pin Class
# This is really just a controller for a backend model that lives in Rust
# During intialization, the pin will be looked up and linked to the given
class Pin:
  def __init__(self, name, pin_container):
    self.__pin_container = pin_container
    self.__name = name

  def __pin_field(self, field):
    print(self.__pin_container.__pin__(self.__name))
    return self.__pin_container.__pin__(self.__name)[field]

  def __set_pin_field(self, field, value):
    m = f"self.__pin_container.pin.set_field({field}, {value})"
    return eval(m)

  @property
  def name(self):
    return self.__name

  @property
  def postured_state(self):
    return self.__pin_field("postured_state")

  @property
  def data(self):
    return self.__pin_field("postured_state")

  @property
  def action(self):
    return self.__pin_field("action")

  @property
  def role(self):
    return self.__pin_field("role")

  @property
  def aliases(self):
    return self.__pin_field("aliases")

  @property
  def meta(self):
    return self.__pin_field("meta")

  def posture(self, value):
    return self.__set_pin_field("state", value)
  
  def drive(self, value=None):
    return self.__set_pin_field("action", "drive")

  def verify(self, value=None):
    return self.__set_pin_field("action", "verify")

  def capture(self, value=None):
    return self.__set_pin_field("action", "capture")
  