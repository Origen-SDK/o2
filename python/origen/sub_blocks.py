import origen

# A middleman between the Python controller and the associated Rust model and
# which implements the application/user API for working with (sub-)blocks.
# An instance of this class is returned by <my_controller>.sub_blocks
class Proxy:
    def __init__(self, controller):
        self.controller = controller
        self.__dict__ = {}

    def __getitem__(self, key):
        return self.__dict__[key]

    def __add_block__(self, name, obj):
        self.__dict__[name] = obj

    def __len__(self):
        return len(self.__dict__)

    def len(self):
        return len(self.__dict__)

    def keys(self):
        return self.__dict__.keys()

    def values(self):
        return self.__dict__.values()

    def items(self):
        return self.__dict__.items()

    def __cmp__(self, dict_):
        return self.__cmp__(self.__dict__, dict_)

    def __contains__(self, item):
        return item in self.__dict__

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
            "sub_block": self.sub_block, 
        }
                
