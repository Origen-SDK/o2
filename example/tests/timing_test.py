import pytest, abc
import origen, _origen # pylint: disable=import-error

@pytest.fixture
def clean_eagle():
  origen.app.instantiate_dut("dut.eagle")
  assert origen.dut
  return origen.dut

@pytest.fixture
def clean_falcon():
  origen.app.instantiate_dut("dut.falcon")
  assert origen.dut
  return origen.dut

def test_empty_timesets(clean_falcon):
  assert len(origen.dut.timesets) == 0
  # assert origen.tester.timeset is None
  # assert origen.tester.current_period is None

def test_adding_a_simple_timeset(clean_falcon):
  t = origen.dut.add_timeset("t0", default_period=1)
  assert isinstance(t, _origen.dut.timesets.Timeset)
  assert t.name == "t0"
  assert t.default_period == 1
  assert t.__eval_str__ == "period"
  assert t.period == 1.0

  # Check the DUT
  assert len(origen.dut.timesets) == 1
  assert origen.dut.timesets.keys() == ["t0"]

  # # Ensure this doesn't to anything to the tester
  # assert origen.tester.timeset is None
  # assert origen.tester.current_period == 0

def test_adding_another_simple_timeset():
  t = origen.dut.add_timeset("t1")
  assert isinstance(t, _origen.dut.timesets.Timeset)
  assert t.name == "t1"

  # No default period set, so attempts to resolve the numerical value will fail.
  assert t.default_period == None
  assert t.__eval_str__ == "period"
  with pytest.raises(OSError):
    t.period

  # Check the DUT
  assert len(origen.dut.timesets) == 2
  assert origen.dut.timesets.keys() == ["t0", "t1"]

  # # Ensure this doesn't to anything to the tester
  # assert origen.tester.timeset is None
  # assert origen.tester.current_period == 0

def test_retrieving_timesets():
  t = origen.dut.timeset("t0")
  assert isinstance(t, _origen.dut.timesets.Timeset)
  assert t.name == "t0"

def test_none_on_retrieving_nonexistant_timesets():
  t = origen.dut.timeset("t")
  assert t is None

def test_exception_on_duplicate_timesets():
  with pytest.raises(OSError):
    origen.dut.add_timeset("t0")
  assert len(origen.dut.timesets) == 2
  assert origen.dut.timesets.keys() == ["t0", "t1"]

def test_adding_timeset_with_equation(clean_falcon):
  t = origen.dut.add_timeset("t0", "period/2", default_period=1)
  assert t.default_period == 1.0
  assert t.__eval_str__ == "period/2"
  assert t.period == 0.5

# def test_adding_simple_timeset_with_context_manager(clean_falcon):
#   with origen.dut.new_timeset("t0") as _t:
#     _t.default_period = 1
#     _t.period = "period/4 + 0.5"
#   assert isinstance(t, _origen.Timeset)
#   assert t.name == "t0"
#   assert t.default_period == 1
#   assert t.__eval_str__ == "period/4 + 0.5"
#   assert t.period == 0.75
#   assert len(origen.dut.timesets) == 1
#   assert origen.dut.timesets.keys == ["t0"]

# def test_adding_complex_timesets(clean_falcon):
#   with origen.dut.new_timeset("t0") as _t:
#     _t.default_period = 1
#     with _t.drive_wave("swd_clk") as w:
#       w.drive(1, "period/4")
#       w.drive(0, "period/2")
#       w.drive(1, "3*period/4")
#       w.drive(0, "period")
#     with _t.verify_wave("swd_io") as w:
#       w.verify("data", "period/10")
#   assert isinstance(t, _origen.Timeset)
#   assert t.name == "t0"
#   assert t.default_period == 1

#   assert len(t.drive_waves) == 4
#   assert t.drive_waves[0].__eval_str__ == "period/4"
#   assert t.drive_waves[0].data == 1
#   assert t.drive_waves[0].at == "0.25"
#   for i, wave in enumerate(t.drive_waves):
#     assert wave.at == 0.25*(i+1)
  
#   assert len(t.verify_waves) == 1
#   assert t.verify_waves[0].__expr_str__ == "data"
#   assert t.verify_waves[0].data == 0
#   assert t.verify_waves[0].at == 0.1

# ### With Tester ####

# Test Dict-Like API
class Fixture_DictLikeAPI(abc.ABC):
  class Expected:
    def __init__(self, parameters):
      self.keys = parameters["keys"]
      self.klass = parameters["klass"]
      self.not_in_dut = parameters["not_in_dut"]

    @property
    def len(self):
      return len(self.keys)
    
    @property
    def in_dut(self):
      return self.keys[0]

  @abc.abstractmethod
  def boot_dict_under_test(self):
    pass

  @abc.abstractmethod
  def parameterize(self):
    pass

  # Pytest skips classes which have an __init__ constructor (http://doc.pytest.org/en/latest/goodpractices.html#conventions-for-python-test-discovery)
  # get around this by using a fixture on the first test to boot the object.
  @pytest.fixture
  def boot(self):
    self.expected = self.Expected(self.parameterize())
    self.dut = self.boot_dict_under_test()
    print("Booted!")

  def test_keys(self, boot):
    assert self.dut.keys
    assert isinstance(self.dut.keys(), list)
    assert self.dut.keys() == self.expected.keys

  def test_values(self, boot):
    assert self.dut.values
    assert isinstance(self.dut.values(), list)
    assert len(self.dut.values()) == self.expected.len
    assert isinstance(self.dut.values()[0], self.expected.klass)

  def test_items(self, boot):
    items = self.dut.items()
    assert items
    assert isinstance(items, list)
    assert isinstance(items[0], tuple)
    assert len(items) == self.expected.len
    assert list(i[0] for i in items) == self.expected.keys

  def test_contains(self, boot):
    assert self.expected.in_dut in self.dut
    assert self.expected.not_in_dut not in self.dut

  def test_len(self, boot):
    assert self.dut.__len__
    assert len(self.dut) == self.expected.len

  def test_getitem(self, boot):
    assert self.dut.get
    assert self.dut.get(self.expected.in_dut)
    assert self.dut.get(self.expected.not_in_dut) is None

  def test_index_notation(self, boot):
    assert self.dut[self.expected.in_dut]
    with pytest.raises(KeyError):
      self.dut[self.expected.not_in_dut]

  def test_iterating(self, boot):
    i = 0
    for _i, k in enumerate(self.dut):
      assert k == self.expected.keys[_i]
      i += 1
    assert i == self.expected.len

  def test_iterating_through_items(self, boot):
    i = 0
    for _i, (k, v) in enumerate(self.dut.items()):
      assert k == self.expected.keys[_i]
      assert isinstance(v, self.expected.klass)
      i += 1
    assert i == self.expected.len

  def test_dict_conversion(self, boot):
    d = dict(self.dut)
    assert isinstance(d, dict)
    assert list(d.keys()) == self.expected.keys

  def test_list_converstion(self, boot):
    l = list(self.dut)
    assert isinstance(l, list)
    assert l == self.expected.keys

# class TestDummy:
#   def test_dummy_class(self):
#     assert False

class TestTimesetsDictLike(Fixture_DictLikeAPI):
  def parameterize(self):
    return {
      "keys": ["t0", "t1", "t2"],
      "klass": _origen.dut.timesets.Timeset,
      "not_in_dut": "Blah"
    }

  def boot_dict_under_test(self):
    origen.app.instantiate_dut("dut.eagle")
    dut = origen.dut
    dut.add_timeset("t0")
    dut.add_timeset("t1")
    dut.add_timeset("t2")
    return dut.timesets