import origen
import _origen

# The base class of all Origen controller objects
class Base:
    # Returns the internal Origen model of this block
    model = None
    # Returns the application instance that defines this block
    app = None
    # Returns the block path that was used to load this block
    # e.g. "dut.falcon"
    block_path = None

    def __init__(self):
        self.model = _origen.model.ModelDB("tbd")

    # This lazy-loads the block's files the first time a given resource is referenced
    def __getattr__(self, name):
        if name == "regs":
            self.app.load_block_files(self, "registers.py")
            from origen.registers import Proxy
            self.regs = Proxy(self)
            return self.regs

        elif name == "sub_blocks":
            from origen.sub_blocks import Proxy
            self.sub_blocks = Proxy(self)
            self.app.load_block_files(self, "sub_blocks.py")
            return self.sub_blocks

        else:
            raise AttributeError(f"The controller for block '{self.block_path}' has no attribute '{name}'")

# The base class of all Origen controller objects which are also
# the top-level (DUT)
class TopLevel(Base):
    def __init__(self):
        origen.dut = self
        Base.__init__(self)
