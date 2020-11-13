import _origen
from origen.controller import Base


class Generic(Base):
    def __init__(self, *, offset=0, word_size=4, length=None):
        if length is None:
            raise ValueError(
                "Argument 'length' when defining a memory cannot be None")
        self.length_in_bytes = length

        Base.__init__(self)
        self.type = "Generic"
        self.word_size = word_size

    @property
    def size(self):
        return self.length_in_bytes / self.word_size

    @property
    def start_addr(self):
        return self.address()

    start_address = start_addr
    start = start_addr
    first_addr = start_addr
    first_address = start_addr
    first = start_addr

    @property
    def last_addr(self):
        return self.address() + self.length - self.word_size

    last_address = last_addr
    last = last_addr
    end_addr = last_addr
    end_address = last_addr
    end = last_addr

    @property
    def length(self):
        return self.length_in_bytes

    @property
    def is_generic(self):
        return self.type == "Generic"

    @property
    def is_ram(self):
        return self.type == "RAM"

    @property
    def is_rom(self):
        return self.type == "ROM"

    @property
    def is_flash(self):
        return self.type == "Flash"

    def __getitem__(self, i):
        return self.parent.read_register(i)

    def __setitem__(self, i, value):
        return self.parent.write_register(i, value)

    def __len__(self):
        return self.length


class RAM(Generic):
    def __init__(self, **kwargs):
        Generic.__init__(self, **kwargs)
        self.type = "RAM"


class ROM(Generic):
    class WriteROMError(TypeError):
        def __init__(self):
            self.message = "Cannot write to ROM memory type"

    def __init__(self, **kwargs):
        Generic.__init__(self, **kwargs)
        self.type = "ROM"

    def write_register(self, *args, **kwargs):
        raise ROM.WriteROMError()


class Flash(Generic):
    def __init__(self, **kwargs):
        Generic.__init__(self, **kwargs)
        self.type = "Flash"
