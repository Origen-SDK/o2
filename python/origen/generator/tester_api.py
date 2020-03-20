import origen # pylint: disable=import-error
from . import processor # pylint: disable=import-error,relative-beyond-top-level
import pickle

class TesterAPI(processor.Processor):
  def __init__(self):
    processor.Processor.__init__(self)
  
  def generate(self):
    return self.process(origen.test_ast())
  
  def __origen__issue_callback__(self, func, node_bytes):
    if hasattr(self, func):
      node = pickle.loads(bytes(node_bytes))
      return getattr(self, func)(node)