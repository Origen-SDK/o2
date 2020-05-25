import _origen
from contextlib import contextmanager, ContextDecorator
import origen.producer
import origen.helpers
from pathlib import Path
import importlib
from origen.interface import BasicInterface

top_level_flow_open = False

class Producer(_origen.producer.PyProducer):
  def issue_callback(self, c, kwargs):
      if origen.helpers.has_method(origen.dut, c) and not kwargs.get("skip_all_callbacks") and not kwargs.get(f"skip_callback_{c}"):
          getattr(origen.dut, c)(**kwargs)
          return True # Callback ran or raised an exception
      return False # Callback didn't run
      
  # Defines the methods that are accessible within blocks/<block>/registers.py
  def api(self):
      return {
          "Pattern": self.Pattern, 
          "Flow": self.Flow, 
      }

  @contextmanager
  def Pattern(self, **kwargs):
      # Always freshly load the target when generating a pattern, no matter how much anyone
      # complains about this!
      # It guarantees that produced patterns are always the same regardless of generation
      # order by clearing all DUT state.
      origen.target.load()

      job = origen.producer.current_job
      name = Path(job.source_file).stem
      pat = PatternClass(name, **kwargs)

      # This initializes a new AST for the pattern we are about to generate
      _origen.start_new_test(pat.name)
      origen.tester.generate_pattern_header(pat.header_comments)

      origen.logger.debug(f"Producing pattern {pat.name} in job {job.id}")
      origen.producer.issue_callback('startup', kwargs)
      yield pat
      origen.producer.issue_callback('shutdown', kwargs)

      origen.tester.end_pattern()
      origen.tester.render_pattern()

  @contextmanager
  def Flow(self, **kwargs):




      # Instantiate the app interface
      if origen.interface is None:
          path = f'{_origen.app_config()["name"]}.interface.interface'
          origen.logger.trace(f"Looking for application test program interface at {path}")
          try:
              origen.logger.trace(f"Found application interface module, instantiating the Interface class")
              m = importlib.import_module(path)
              origen.interface = m.Interface()
          except ModuleNotFoundError:
              origen.logger.trace(f"Not found, instantiating Origen's basic interface instead")
              origen.interface = BasicInterface()
          except AttributeError:
              origen.logger.trace(f"Not found, instantiating Origen's basic interface instead")
              origen.interface = BasicInterface()

      global top_level_flow_open

      job = origen.producer.current_job
      name = Path(job.current_file).stem
      flow = FlowClass(name, **kwargs)

      if top_level_flow_open:
          top_level = False
          origen.logger.debug(f"Producing sub-flow '{flow.name}' in job '{job.id}'")
      else:
          origen.logger.debug(f"Producing flow '{flow.name}' in job '{job.id}'")
          top_level = True
          top_level_flow_open = True

      #origen.tester.reset()
      #origen.target.reload()
      #origen.tester.clear_dut_dependencies(ast_name=flow.name)
      #origen.tester.generate_pattern_header(flow.header_comments)

      #origen.producer.issue_callback('startup', kwargs)
      yield origen.interface
      #origen.producer.issue_callback('shutdown', kwargs)

      if top_level:
        top_level_flow_open = False

      #origen.tester.end_pattern()
      #origen.tester.render()

# (_origen.producer.PyPattern)
class PatternClass:
  def __init__(self, name, **kwargs):
    if name in kwargs:
      # User overwrote the pattern name, or provided one for a sourceless generation
      processed_name = kwargs['name']
    else:
      processed_name = name

    if "prefix" in kwargs:
      processed_name = f"{kwargs['prefix']}_{processed_name}"
    
    if "postfix" in kwargs:
      processed_name = f"{processed_name}_{kwargs['postfix']}"

    self.name = processed_name

    # Collect the header comments from:
    #  * The application
    #  * <To-do> Current plugin
    #  * <To-do> Other plugins (if necessary)
    #  * Pattern specifics given in the header
    self.header_comments = {}
    if origen.helpers.has_method(origen.app, "pattern_header"):
      self.header_comments["app"] = origen.app.pattern_header(self)
    
    if "header_comments" in kwargs:
      self.header_comments["pattern"] = kwargs["header_comments"]

# (_origen.producer.PyFlow)
class FlowClass:
  def __init__(self, name, **kwargs):
    if name in kwargs:
      # User overwrote the pattern name, or provided one for a sourceless generation
      processed_name = kwargs['name']
    else:
      processed_name = name

    if "prefix" in kwargs:
      processed_name = f"{kwargs['prefix']}_{processed_name}"
    
    if "postfix" in kwargs:
      processed_name = f"{processed_name}_{kwargs['postfix']}"

    self.name = processed_name

    # Collect the header comments from:
    #  * The application
    #  * <To-do> Current plugin
    #  * <To-do> Other plugins (if necessary)
    #  * Pattern specifics given in the header
    self.header_comments = {}
    if origen.helpers.has_method(origen.app, "flow_header"):
      self.header_comments["app"] = origen.app.flow_header(self)
    
    if "header_comments" in kwargs:
      self.header_comments["flow"] = kwargs["header_comments"]

