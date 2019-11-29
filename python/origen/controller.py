import origen
import _origen

# The base class of all Origen controller objects
class Base:
    model = None
    __regs_loaded = False

    # Returns the application instance that defines this controller type
    app = None
    # Returns the block path that was used to load this controller,
    # e.g. "dut.falcon"
    block_path = None

    def __init__(self):
        self.model = _origen.model.ModelDB("tbd")

    # Force the registers to be loaded
    def load_regs(self):
        if not self.__regs_loaded:
            self.app.load_block_files(self, "registers.py")
            self.__regs_loaded = True

# The base class of all Origen controller objects which are also
# the top-level (DUT)
class TopLevel(Base):

    def __init__(self):
        origen.dut = self
        Base.__init__(self)
