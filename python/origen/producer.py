import _origen
from contextlib import contextmanager, ContextDecorator
import origen.producer
from pathlib import Path

class Producer(_origen.producer.PyProducer):
  def issue_callback(self, c, kwargs):
      if hasattr(origen.dut, c) and callable(getattr(origen.dut, c)) and not kwargs.get("skip_all_callbacks") and not kwargs.get(f"skip_callback_{c}"):
          getattr(origen.dut, c)(**kwargs)
          return True # Callback ran or raised an exception
      return False # Callback didn't run

  def standard_callbacks(self):
    return [(origen.dut, "startup"), (origen.dut, "shutdown")]

  @contextmanager
  def produce_pattern(self, job, **kwargs):
      name = Path(job.command).stem
      pat = Pattern(name, **kwargs)

      origen.tester.reset()
      origen.target.reload()
      origen.tester.clear_dut_dependencies(ast_name=pat.name)

      def callback(m):
          if hasattr(origen.dut, m) and callable(getattr(origen.dut, m)) and not kwargs.get(f"skip_{m}"):
              getattr(origen.dut, m)(**kwargs)

      origen.logger.info(f"  Producing Pattern with job ID {job.id}")
      callback('startup')
      yield pat
      callback('shutdown')

      origen.tester.render()

class Pattern(_origen.producer.PyPattern):
  def __init__(self, name, **kwargs):
    if name in kwargs:
      # User overwrote the pattern name, or provided one for a sourceless generation
      name = kwargs['name']
    
    if "prefix" in kwargs:
      name = f"{kwargs['prefix']}_{name}"
    
    if "postfix" in kwargs:
      name = f"{name}_{kwargs['postfix']}"

    self.name = name
