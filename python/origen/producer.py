import _origen
from contextlib import contextmanager, ContextDecorator
import origen.producer
import origen.helpers
from pathlib import Path

class Producer(_origen.producer.PyProducer):
  def issue_callback(self, c, kwargs):
      if origen.helpers.has_method(origen.dut, c) and not kwargs.get("skip_all_callbacks") and not kwargs.get(f"skip_callback_{c}"):
          getattr(origen.dut, c)(**kwargs)
          return True # Callback ran or raised an exception
      return False # Callback didn't run

  @contextmanager
  def produce_pattern(self, job, **kwargs):
      name = Path(job.command).stem
      pat = Pattern(name, **kwargs)

      origen.tester.reset()
      origen.target.reload()
      origen.tester.clear_dut_dependencies(ast_name=pat.name)

      origen.logger.info(f"Producing Pattern {pat.name} with job ID {job.id}")
      origen.producer.issue_callback('startup', kwargs)
      yield pat
      origen.producer.issue_callback('shutdown', kwargs)

      origen.tester.end_pattern()
      origen.tester.render()

class Pattern:
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
