import origen
import _origen

# The base class of all Origen controller objects
class Base:
    model = None

    def __init__(self):
        self.model = _origen.model.ModelDB("tbd")


# The base class of all Origen controller objects which are also
# the top-level (DUT)
class TopLevel(Base):

    def __init__(self):
        Base.__init__(self)
        origen.dut = self
