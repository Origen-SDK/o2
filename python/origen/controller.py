import origen
import _origen

# The base class of all Origen controller objects
class Base:

    # This is the ID given to this block instance by its parent. For example, if this
    # block was globally available as "dut.ana.adc0", then its id attribute would be "adc0"
    id = None
    # Returns the path to this block's parent relative to the top-level DUT. For example,
    # if this block was globally available as "dut.core0.ana.adc0", then its parent attribute
    # would return "core0.ana"
    parent_path = None
    # Returns the path to this block relative to the top-level DUT, essentially the
    # concatenation of parent_path and id
    path = None
    # Returns the application instance that defines this block
    app = None
    # Returns the block path that was used to load this block, e.g. "dut.falcon"
    block_path = None

    is_top = False

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

        elif name == "memory_maps":
            self.regs  # Ensure the memory maps for this block have been loaded
            return origen.dut.db.memory_maps(self.path)

        else:
            raise AttributeError(f"The block '{self.block_path}' has no attribute '{name}'")

    def tree(self):
        print(self.tree_as_str())

    def tree_as_str(self):
        if self.is_top:
            t = "dut"
        elif self.parent == '':
            t = f"dut.{self.id}"
        else:
            t = f"dut.{self.parent}.{self.id}"
        return t

    def memory_map(self, id):
        self.regs  # Ensure the memory maps for this block have been loaded
        return origen.dut.db.memory_map(self.path, id)


# The base class of all Origen controller objects which are also
# the top-level (DUT)
class TopLevel(Base):
    # Returns the internal Origen data base for this DUT
    db = None
    is_top = True

    def __init__(self):
        self.id = ""
        self.parent_path = ""
        self.path = ""
        origen.dut = self
        # TODO: Probably pass the name of the target in here to act as the DUT name/ID
        self.db = _origen.dut.PyDUT("tbd")
        Base.__init__(self)
