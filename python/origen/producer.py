import _origen
from contextlib import contextmanager, ContextDecorator
import origen.producer

class Job(_origen.producer.PyJob):
  @contextmanager
  def produce_pattern(self, **kwargs):
      origen.tester.reset()
      origen.target.reload()
      origen.tester.target("::V93K::ST7")

      def callback(m):
          if hasattr(origen.dut, m) and callable(getattr(origen.dut, m)) and not kwargs.get(f"skip_{m}"):
              getattr(origen.dut, m)(**kwargs)

      origen.logger.info(f"  Producing Pattern")
      pat = Pattern(**kwargs)
      callback('startup')
      yield pat
      callback('shutdown')

      origen.tester.generate()

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
      origen.tester.reset()
      origen.target.reload()
      origen.tester.target("::V93K::ST7")

      def callback(m):
          if hasattr(origen.dut, m) and callable(getattr(origen.dut, m)) and not kwargs.get(f"skip_{m}"):
              getattr(origen.dut, m)(**kwargs)

      origen.logger.info(f"  Producing Pattern with job ID {job.id}")
      pat = Pattern(**kwargs)
      callback('startup')
      yield pat
      callback('shutdown')

      origen.tester.generate()

class Pattern(_origen.producer.PyPattern):
  def __init__(self, **kwargs):
    ...
