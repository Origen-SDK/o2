import origen
import _origen
from origen import pins
from origen.registers import Loader as RegLoader
from contextlib import contextmanager

class Proxies:
    def __init__(self, controller):
        self.controller = controller
        self.proxies = {}
    
    def __getitem__(self, name):
        if (p := self.proxies.get(name)):
            return p
        else:
            origen.logger.error(f"No proxy for '{name}' has been set!")
            exit()
    
    def __setitem__(self, name, proxy):
        if proxy in self.proxies:
            origen.logger.error(f"A proxy for '{proxy}' has already been set! Cannot set the same proxy again!")
            exit()
        else:
            self.proxies[name] = proxy
            return proxy

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
        self.__proxies__ = Proxies(self)
        self.regs_loaded = False

    # This lazy-loads the block's files the first time a given resource is referenced
    def __getattr__(self, name):
        # regs called directly on the controller means only the regs in the default
        # memory map and address block
        if name == "regs":
            self._load_regs()
            return origen.dut.db.regs(self.path, None, None)

        elif name == "sub_blocks":
            from origen.sub_blocks import Proxy
            self.sub_blocks = Proxy(self)
            self.app.load_block_files(self, "sub_blocks.py")
            return self.sub_blocks

        elif name in pins.Proxy.api():
            from origen.pins import Proxy
            proxy = pins.Proxy(self)
            self.__proxies__["pins"] = proxy
            for method in pins.Proxy.api():
                self.__setattr__(method, getattr(proxy, method))
            return eval(f"self.{name}")
        
        elif name == "memory_maps":
            self._load_regs()
            return origen.dut.db.memory_maps(self.path)

        elif name in self.sub_blocks:
            return self.sub_blocks[name]

        else:
            self._load_regs()

            if name in self.memory_maps:
                return self.memory_maps[name]

            else:
                raise AttributeError(f"The block '{self.block_path}' has no attribute '{name}'")

    def tree(self):
        print(self.tree_as_str())

    def tree_as_str(self, leader='', include_header=True):
        if include_header:
            if self.is_top:
                t = "dut"
            elif self.parent_path == '':
                t = f"dut.{self.id}"
            else:
                t = f"dut.{self.parent_path}.{self.id}"
            names = t.split('.')
            names.pop()
            if not names:
                leader = ' '
            else:
                leader = ' ' * (2 + len('.'.join(names)))
        else:
            t = ''
        last = len(self.sub_blocks) - 1
        for i, key in enumerate(sorted(self.sub_blocks.keys())):
            if i != last:
                t += "\n" + leader + f"├── {key}"
            else:
                t += "\n" + leader + f"└── {key}"
            if self.sub_blocks[key].sub_blocks.len() > 0:
                if i != last:
                    l = leader + '│    '
                else:
                    l = leader + '     '
                t += self.sub_blocks[key].tree_as_str(l, False)
        return t

    def memory_map(self, id):
        self.regs  # Ensure the memory maps for this block have been loaded
        return origen.dut.db.memory_map(self.path, id)

    def add_simple_reg(self, *args, **kwargs):
        RegLoader(self).SimpleReg(*args, **kwargs)

    @contextmanager
    def add_reg(self, *args, **kwargs):
        with RegLoader(self).Reg(*args, **kwargs) as reg:
            yield reg

    def _load_regs(self):
        if not self.regs_loaded:
            self.app.load_block_files(self, "registers.py")
            self.regs_loaded = True

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
