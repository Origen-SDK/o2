import origen


# This defines the methods for defining sub-blocks in Python and then handles serializing
# the definitions and handing them over to the Rust model for instantiation.
class Loader:
    def __init__(self, controller):
        self.controller = controller

    def sub_block(self, name, block_path=None, offset=None):
        b = self.controller.app.instantiate_block(block_path)
        b.name = name
        b.path = f"{self.controller.path}.{name}"
        # Add the python representation of this block to its parent
        b.parent = self.controller
        self.controller.sub_blocks[name] = b
        # Create a new representation of it in the internal database
        b.model_id = origen.dut.db.create_model(self.controller.model_id, name,
                                                offset)
        return b

    # Defines the methods that are accessible within blocks/<block>/sub_blocks.py
    def api(self):
        return {
            "SubBlock": self.sub_block,
        }
