import origen  # pylint: disable=import-error
from . import processor  # pylint: disable=import-error,relative-beyond-top-level
import pickle


class TesterAPI(processor.Processor):
    # Needed so pytest doesn't attempt to collect this as a test structure (since it starts with 'Test')
    __test__ = False

    def __init__(self):
        processor.Processor.__init__(self)

    def render_pattern(self):
        return self.process(origen.test_ast())

    def __origen__issue_callback__(self, func, node_bytes):
        if hasattr(self, func):
            node = pickle.loads(bytes(node_bytes))
            return getattr(self, func)(node)
