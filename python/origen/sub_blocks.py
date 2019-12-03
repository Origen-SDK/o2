# A middleman between the Python controller and the associated Rust model and
# which implements the application/user API for working with (sub-)blocks.
# An instance of this class is returned by <my_controller>.sub_blocks
class Proxy:
    def __init__(self, model):
        self.model = model

# This defines the methods for defining sub-blocks in Python and then handles serializing
# the definitions and handing them over to the Rust model for instantiation.
class Loader:
    def __init__(self, model):
        self.model = model

    def sub_block(self, name, block_path=None):
        pass

    # Defines the methods that are accessible within blocks/<block>/sub_blocks.py
    def api(self):
        return {
            "sub_block": self.sub_block, 
        }
                
