import origen
from types import ModuleType


# This defines the methods for defining sub-blocks in Python and then handles serializing
# the definitions and handing them over to the Rust model for instantiation.
class Loader:
    def __init__(self, controller):
        self.controller = controller

    def sub_block(self,
                  name,
                  block_path=None,
                  class_name="Controller",
                  *,
                  offset=None,
                  sb_options={}):
        if self.controller.__class__.is_currently_loading(name):
            return None
        else:
            self.controller.__class__.currently_loading(name)
        b = self.controller.app.instantiate_block(
            block_path,
            base_path=self.controller.block_path,
            class_name=class_name,
            sb_options=sb_options)

        b.name = name
        b.path = f"{self.controller.path}.{name}"
        # Add the python representation of this block to its parent
        b.parent = self.controller
        self.controller.sub_blocks[name] = b
        # Create a new representation of it in the internal database
        b.model_id = origen.dut.db.create_model(self.controller.model_id, name,
                                                offset)
        if hasattr(b, "model_init"):
            b.model_init(b, block_options=sb_options)
        else:
            for base in b.__class__.__bases__:
                if hasattr(base, "model_init"):
                    base.model_init(b, block_options=sb_options)
        self.controller.__class__.done_loading(name)
        return b

    # Defines the methods that are accessible within blocks/<block>/sub_blocks.py
    def api(self):
        return {
            "SubBlock": self.sub_block,
        }
