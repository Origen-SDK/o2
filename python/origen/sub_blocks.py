from origen import dut

# A middleman between the Python controller and the associated Rust model and
# which implements the application/user API for working with (sub-)blocks.
# An instance of this class is returned by <my_controller>.sub_blocks
class Proxy:
    def __init__(self, controller):
        self.controller = controller
        self.__store = {}

    def __getitem__(self, key):
        return self.__store[key]

    def __add_block__(self, id, obj):
        self.__store[id] = obj

# This defines the methods for defining sub-blocks in Python and then handles serializing
# the definitions and handing them over to the Rust model for instantiation.
class Loader:
    def __init__(self, controller):
        self.controller = controller

    def sub_block(self, id, block_path=None):
        b = self.controller.app.instantiate_block(block_path)
        b.id = id
        b.parent = f"{self.controller.parent}.{self.controller.id}"
        # If a direct child of the top-level
        if b.parent == ".":
            b.parent = ''
        # Add the python representation of this block to its parent
        self.controller.sub_blocks.__add_block__(id, b)
        # Create a new representation of it in the internal store
        dut.store.create_model(b.parent, b.id)
        pass

    # Defines the methods that are accessible within blocks/<block>/sub_blocks.py
    def api(self):
        return {
            "sub_block": self.sub_block, 
        }
                
