import origen


# This defines the methods for defining block attributes in Python
class Loader:
    def __init__(self, controller):
        self.controller = controller

    def attribute(self, key, value):
        self.controller.attributes[key] = value

    # Defines the methods that are accessible within blocks/<block>/attributes.py
    def api(self):
        return {
            "Attr": self.attribute,
        }
