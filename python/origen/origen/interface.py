import origen
import _origen


# Base class for all test program flow interfaces
class BaseInterface(_origen.interface.PyInterface):
    def __init__(self):
        self._options = []
        self.bypass_sub_flows = False
        self.add_flow_enable = None
        pass

    def include(self, path, **kwargs):
        origen.log.trace(f"Resolving include reference '{path}'")
        file = self.resolve_file_reference(path)
        origen.log.trace(f"Found include file '{file}'")
        origen.producer.current_job.add_file(file)
        context = origen.producer.api()
        origen.interface._push_options(kwargs)
        origen.load_file(file, locals=context)
        origen.interface._pop_options()
        origen.producer.current_job.pop_file()

    def _push_options(self, kwargs):
        self._options.append(kwargs)

    def _pop_options(self):
        self._options.pop()

    @property
    def options(self):
        return self._options[-1]

    def render(self, template, **kwargs):
        # TODO: Handle a non-template correctly, i.e. if file doesn't have a compiler extension
        # then simply render it verbatim
        compiler = origen.app.compiler.renderer_for(template)({})
        contents = open(template).read()
        text = compiler.render_str(contents, kwargs)
        self.render_str(text)


def dut():
    return origen.dut


def tester():
    return origen.tester


# This interface will be used by Origen when generating a test program flow unless:
# 1) The application defines <app>.interface.default
# 2) An interface argument is given to with Flow()
class BasicInterface(BaseInterface):
    pass
