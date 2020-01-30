import origen

# A middleman between the Python controller and the associated Rust model and
# which implements the application/user API for working with (sub-)blocks.
# An instance of this class is returned by <my_controller>.sub_blocks
class Proxy:
    def __init__(self, controller):
        self.controller = controller
        self._dict = {}

    def __getitem__(self, key):
        return self._dict[key]

    def __add_block__(self, name, obj):
        self._dict[name] = obj

    def __len__(self):
        return len(self._dict)

    def len(self):
        return len(self._dict)

    def keys(self):
        return self._dict.keys()

    def values(self):
        return self._dict.values()

    def items(self):
        return self._dict.items()

    def __cmp__(self, dict_):
        return self.__cmp__(self._dict, dict_)

    def __contains__(self, item):
        return item in self._dict

    def __iter__(self):
        return iter(self._dict)

# This defines the methods for defining sub-blocks in Python and then handles serializing
# the definitions and handing them over to the Rust model for instantiation.
class Loader:
    def __init__(self, controller):
        self.controller = controller

    def sub_block(self, name, block_path=None):
        b = self.controller.app.instantiate_block(block_path)
        b.name = name
        b.path = f"{self.controller.path}.{name}"
        # Add the python representation of this block to its parent
        self.controller.sub_blocks.__add_block__(name, b)
        # Create a new representation of it in the internal database
        b.model_id = origen.dut.db.create_model(self.controller.model_id, name)
        return b

    # Defines the methods that are accessible within blocks/<block>/sub_blocks.py
    def api(self):
        return {
            "SubBlock": self.sub_block, 
        }