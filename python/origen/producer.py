import _origen
from contextlib import contextmanager, ContextDecorator
import origen.producer
import origen.helpers
from pathlib import Path
import importlib


class Producer(_origen.producer.PyProducer):
  def issue_callback(self, c, kwargs):
      if origen.helpers.has_method(origen.dut, c) and not kwargs.get("skip_all_callbacks") and not kwargs.get(f"skip_callback_{c}"):
          getattr(origen.dut, c)(**kwargs)
          return True # Callback ran or raised an exception
      return False # Callback didn't run

  @contextmanager
  def Pattern(self, job, **kwargs):
      name = Path(job.command).stem
      pat = PatternClass(name, **kwargs)

      origen.tester.reset()
      origen.target.reload()
      origen.tester.clear_dut_dependencies(ast_name=pat.name)
      origen.tester.generate_pattern_header(pat.header_comments)

      origen.logger.debug(f"Producing pattern {pat.name} with job ID {job.id}")
      origen.producer.issue_callback('startup', kwargs)
      yield pat
      origen.producer.issue_callback('shutdown', kwargs)

      origen.tester.end_pattern()
      origen.tester.render_pattern()

  @contextmanager
  def Flow(self, job, **kwargs):
      name = Path(job.command).stem
      flow = FlowClass(name, **kwargs)

      origen.tester.reset()
      origen.target.reload()
      #origen.tester.clear_dut_dependencies(ast_name=flow.name)
      #origen.tester.generate_pattern_header(flow.header_comments)

      origen.logger.info(f"Generating flow {flow.name} with job ID {job.id}")
      origen.producer.issue_callback('startup', kwargs)
      yield flow.interface
      origen.producer.issue_callback('shutdown', kwargs)

      origen.tester.end_pattern()
      origen.tester.render()

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


    # Instantiate the app interface
    #path = f'{_origen.app_config()["name"]}.interface.default'
    #origen.logger.trace(f"Looking for application default interface at {path}")
    #try:
    #    m = importlib.import_module(path)
    #except ModuleNotFoundError:
    origen.logger.trace(f"Not found")
    origen.logger.trace(f"Instantiating Origen's basic interface instead")
    m = importlib.import_module("origen.interface")
    self.interface = m.BasicInterface()