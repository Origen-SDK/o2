import origen, _origen
from origen.errors import *

PinActions = _origen.dut.pins.PinActions


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
        return origen.dut.db.add_pin_alias(self.controller.model_id, name,
                                           *aliases)

    def group_pins(self, name, *pin_names, **options):
        return origen.dut.db.group_pins(self.controller.model_id, name,
                                        *pin_names, **options)

    def physical_pin(self, name):
        return origen.dut.db.physical_pin(self.controller.model_id, name)

    @property
    def pin_headers(self):
        return origen.dut.db.pin_headers(self.controller.model_id)

    def add_pin_header(self, name, *pins):
        return origen.dut.db.add_pin_header(self.controller.model_id, name,
                                            *pins)

    def pin_header(self, name):
        return origen.dut.db.get_pin_header(self.controller.model_id, name)

    @classmethod
    def api(cls):
        return [
            'pins', 'add_pin', 'pin', 'add_pin_alias', 'group_pins',
            'physical_pin', 'physical_pins', 'pin_headers', 'add_pin_header',
            'pin_header'
        ]


class Loader:
    def __init__(self, controller):
        self.controller = controller

    def Pin(self, name, **kwargs):
        self.controller.add_pin(name, **kwargs)

    def Alias(self, name, *aliases):
        self.controller.add_pin_alias(name, *aliases)

    def Group(self, name, *pins, **options):
        self.controller.group_pins(name, *pins, **options)

    def PinHeader(self, name, *pins):
        self.controller.add_pin_header(name, *pins)

    def api(self):
        return {
            "Pin": self.Pin,
            "Alias": self.Alias,
            "PinHeader": self.PinHeader,
            "Group": self.Group,
        }
