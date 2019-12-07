import origen
import _origen

# The base class of all Origen controller objects
class Base:

    # This is the ID given to this block instance by its parent. For example, if this
    # block was globally available as "dut.ana.adc0", then its id attribute would be "adc0"
    id = None
    # Returns the path to this block instance relative to the top-level DUT. For example
    # if this block was globally available as "dut.core0.ana.adc0", then its parent attribute
    # would return "core0.ana"
    parent = None
    # Returns the application instance that defines this block
    app = None
    # Returns the block path that was used to load this block
    # e.g. "dut.falcon"
    block_path = None

    def __init__(self):
        pass

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
            raise AttributeError(f"The block '{self.block_path}' has no attribute '{name}'")

# The base class of all Origen controller objects which are also
# the top-level (DUT)
class TopLevel(Base):
    # Returns the internal Origen data store for this DUT
    store = None

    def __init__(self):
        self.id = ""
        self.parent = ""
        origen.dut = self
        # TODO: Probably pass the name of the target in here to act as the DUT name/ID
        self.store = _origen.dut.PyDUT("tbd")
        Base.__init__(self)
