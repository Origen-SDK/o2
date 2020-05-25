import origen
import _origen

# Base class for all test program flow interfaces
class BaseInterface(_origen.interface.PyInterface):
    def include(self, path):
        origen.log.trace(f"Resolving include reference '{path}'")
        file = self.resolve_file_reference(path)
        origen.log.trace(f"Found include file '{file}'")
        origen.producer.current_job.add_file(file)
        context = origen.producer.api()
        origen.load_file(file, locals=context)
        origen.producer.current_job.pop_file()

# This interface will be used by Origen when generating a test program flow unless:
# 1) The application defines <app>.interface.default
# 2) An interface argument is given to with Flow()
class BasicInterface(BaseInterface):
    pass