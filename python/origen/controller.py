import origen
import _origen
from origen import pins
from origen import timesets
from origen.registers import Loader as RegLoader
from origen.sub_blocks import Loader as SubBlockLoader
from contextlib import contextmanager

class Proxies:
    def __init__(self, controller):
        self.controller = controller
        self.proxies = {}
    
    def __getitem__(self, name):
        p = self.proxies.get(name)
        if (p):
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
    # block was globally available as "dut.ana.adc0", then its name attribute would be "adc0"
    name = None
    # Returns the path to this block, e.g. "dut.ana.adc0"
    path = None
    # Returns the block path that defined this block, e.g. the block defined in
    # blocks/adc/derivatives/16_bit will have block_path = "adc.16_bit"
    block_path = None
    # Returns the application instance that defines this block
    app = None

    model_id = None

    _default_default_address_block = None

    is_top = False

    def __init__(self):
        self.__proxies__ = Proxies(self)
        self.regs_loaded = False
        self.sub_blocks_loaded = False
        self.pins_loaded = False
        self.timesets_loaded= False

    def __repr__(self):
        self._load_regs()
        self._load_sub_blocks()
        return origen.dut.db.model_console_display(self.model_id)

    # This lazy-loads the block's files the first time a given resource is referenced
    def __getattr__(self, name):
        #print(f"Looking for attribute {name}")
        # regs called directly on the controller means only the regs in the default
        # memory map and address block
        if name == "regs":
            self._load_regs()
            if self._default_default_address_block:
                return self._default_default_address_block.regs
            else:
                return origen.dut.db.empty_regs()

        elif name == "sub_blocks":
            self._load_sub_blocks()
            return self.sub_blocks

        elif name in pins.Proxy.api():
            from origen.pins import Proxy
            proxy = pins.Proxy(self)
            self.__proxies__["pins"] = proxy
            for method in pins.Proxy.api():
                self.__setattr__(method, getattr(proxy, method))
            self._load_pins()
            return eval(f"self.{name}")
        
        elif name == "memory_maps":
            self._load_regs()
            return origen.dut.db.memory_maps(self.model_id)

        elif name in timesets.Proxy.api():
            from origen.timesets import Proxy
            proxy = timesets.Proxy(self)
            self.__proxies__["timesets"] = proxy
            for method in timesets.Proxy.api():
                self.__setattr__(method, getattr(proxy, method))
            self._load_timesets()
            return eval(f"self.{name}")

        else:
            self._load_sub_blocks()

            if name in self.sub_blocks:
                return self.sub_blocks[name]

            self._load_regs()

            if name in self.memory_maps:
                return self.memory_maps[name]

            # Finally see if this is a reference to a reg in the default address block
            if self._default_default_address_block:
                r = self._default_default_address_block.reg(name)
                if r:
                    return r

            try:
                return getattr(self.model(), name)
            except AttributeError:
                raise AttributeError(f"The block '{self.block_path}' has no attribute '{name}'")

    def tree(self):
        print(self.tree_as_str())

    def tree_as_str(self, leader='', include_header=True):
        if include_header:
            t = self.path
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

    def memory_map(self, name):
        self._load_regs()
        return origen.dut.db.memory_map(self.model_id, name)

    def reg(self, name):
        self._load_regs()
        if self._default_default_address_block:
            return self._default_default_address_block.reg(name)
        else:
            raise AttributeError(f"The block '{self.block_path}' has no reg called '{name}' (at least within its default address block)")

    def add_simple_reg(self, *args, **kwargs):
        kwargs["_called_from_controller"] = True
        RegLoader(self).SimpleReg(*args, **kwargs)

    def model(self):
        return origen.dut.db.model(self.model_id)

    @contextmanager
    def add_reg(self, *args, **kwargs):
        self._load_regs()
        kwargs["_called_from_controller"] = True
        with RegLoader(self).Reg(*args, **kwargs) as reg:
            yield reg

    def add_sub_block(self, *args, **kwargs):
        self._load_sub_blocks()
        return SubBlockLoader(self).sub_block(*args, **kwargs)

    def _load_regs(self):
        if not self.regs_loaded:
            self.app.load_block_files(self, "registers.py")
            self.regs_loaded = True

    def _load_sub_blocks(self):
        if not self.sub_blocks_loaded:
            from origen.sub_blocks import Proxy
            self.sub_blocks = Proxy(self)
            self.app.load_block_files(self, "sub_blocks.py")
            self.sub_blocks_loaded = True
    
    def _load_pins(self):
        if not self.pins_loaded:
            self.app.load_block_files(self, "pins.py")
            self.pins_loaded = True

    def _load_timesets(self):
        if not self.timesets_loaded:
            self.app.load_block_files(self, "timing.py")
            self.timesets_loaded = True

# The base class of all Origen controller objects which are also
# the top-level (DUT)
class TopLevel(Base):
    # Returns the internal Origen data base for this DUT
    db = None
    is_top = True

    def __init__(self):
        self.name = "dut"
        self.path = "dut"
        self.model_id = 0
        origen.dut = self
        # TODO: Probably pass the name of the target in here to act as the DUT name/ID
        self.db = _origen.dut.PyDUT("tbd")
        Base.__init__(self)
