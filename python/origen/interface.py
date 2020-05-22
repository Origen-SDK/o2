import origen
import _origen

# Base class for all test program flow interfaces
class Interface(_origen.interface.PyInterface):
    pass


# This interface will be used by Origen when generating a test program flow unless:
# 1) The application defines <app>.interface.default
# 2) An interface argument is given to with Flow()
class BasicInterface(Interface):
    pass