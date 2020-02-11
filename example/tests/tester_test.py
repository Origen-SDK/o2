import pytest
import origen, _origen # pylint: disable=import-error
from shared import clean_eagle, clean_tester
from shared.python_like_apis import Fixture_ListLikeAPI

# Test generator used to test the frontend <-> backend generator hooks
class PyTestGenerator:
  def __init__(self):
    self.nodes = []

  def generate(self, ast):
    print("Printing StubPyAST to console...")
    for i, n in enumerate(ast):
      if n.fields["type"] == "node":
        node_str = "Node"
      elif n.fields["type"] == "comment":
        node_str = f"Comment - Content: {n.fields['content']}"
      elif n.fields["type"] == "vector":
        node_str = f"Vector - Repeat: {n.fields['repeat']}"
      else:
        node_str = f"Error! Unknown type {n.fields['type']}"
      print(f"  PyTestGenerator: Node {i}: {node_str}")

class TestPyAST(Fixture_ListLikeAPI):

  def verify_i0(self, i):
    assert i.fields["type"] == "comment"
    assert i.fields["content"] == "Start!"

  def verify_i1(self, i):
    assert i.fields["type"] == "vector"
    assert i.fields["repeat"] == 4

  def verify_i2(self, i):
    assert i.fields["type"] == "comment"
    assert i.fields["content"] == "End!"

  def boot_list_under_test(self):
    origen.app.instantiate_dut("dut.eagle")
    origen.tester.set_timeset("simple")
    origen.tester.cc("Start!")
    origen.tester.repeat(4)
    origen.tester.cc("End!")
    return origen.tester.ast

def test_init_state(clean_eagle, clean_tester):
  # The 'clean_tester' fixture has a number of asserts itself,
  # but just in case those change unbeknownst to this method, double check the initial state here.
  assert origen.tester
  assert origen.tester.targets == []
  assert len(origen.tester.ast) == 0
  assert origen.tester.generators == ["::DummyGenerator", "::DummyGeneratorWithInterceptors", "::V93K::ST7"]
  assert origen.tester.timeset is None

def test_setting_a_timeset(clean_eagle, clean_tester):
  origen.tester.set_timeset("simple")
  assert isinstance(origen.tester.timeset, _origen.dut.timesets.Timeset)
  assert origen.tester.timeset.name == "simple"

  origen.tester.timeset = "complex"
  assert isinstance(origen.tester.timeset, _origen.dut.timesets.Timeset)
  assert origen.tester.timeset.name == "complex"

def test_resetting_the_timeset():
  assert origen.tester.timeset.name == "complex"
  origen.tester.timeset = None
  assert origen.tester.timeset is None

def test_exception_on_unknown_timeset(clean_eagle, clean_tester):
  with pytest.raises(OSError):
    origen.tester.set_timeset("blah")

def test_setting_timeset_with_instance(clean_eagle, clean_tester):
  assert origen.tester.timeset is None
  origen.tester.set_timeset(origen.dut.timesets["simple"])
  assert origen.tester.timeset.name == "simple"

def test_setting_targets(clean_eagle, clean_tester):
  assert origen.tester.targets == []
  origen.tester.target("::DummyGenerator")
  assert origen.tester.targets == ["::DummyGenerator"]

def test_resetting_targets():
  assert origen.tester.targets == ["::DummyGenerator"]
  origen.tester.clear_targets()
  assert origen.tester.targets == []

def test_exception_on_duplicate_targets(clean_eagle, clean_tester):
  origen.tester.target("::DummyGenerator")
  with pytest.raises(OSError):
    origen.tester.target("::DummyGenerator")

def test_exception_on_unknown_target(clean_eagle, clean_tester):
  with pytest.raises(OSError):
    origen.tester.target("blah")

def test_ast_retrieval(clean_eagle, clean_tester):
  assert origen.tester.ast is not None
  assert isinstance(origen.tester.ast, _origen.tester.StubPyAST)
  assert len(origen.tester.ast) == 0

class TestTesterAPI:
  def test_cycle(self, clean_eagle, clean_tester):
    origen.tester.set_timeset("simple")
    assert len(origen.tester.ast) == 0
    assert origen.tester.ast.cycle_count == 0
    assert origen.tester.ast.vector_count == 0
    origen.tester.cycle()
    assert len(origen.tester.ast) == 1
    assert origen.tester.ast[0].fields["type"] == "vector"
    assert origen.tester.ast[0].fields["repeat"] == 1
    assert origen.tester.ast.cycle_count == 1
    assert origen.tester.ast.vector_count == 1

  def test_multiple_cycles(self, clean_eagle, clean_tester):
    origen.tester.set_timeset("simple")
    assert len(origen.tester.ast) == 0
    assert origen.tester.ast.cycle_count == 0
    assert origen.tester.ast.vector_count == 0
    origen.tester.cycle()
    origen.tester.cycle()
    origen.tester.cycle()
    assert len(origen.tester.ast) == 3
    assert origen.tester.ast.cycle_count == 3
    assert origen.tester.ast.vector_count == 3

  def test_cc(self, clean_eagle, clean_tester):
    origen.tester.set_timeset("simple")
    assert len(origen.tester.ast) == 0
    origen.tester.cc("Hello Tester!")
    assert len(origen.tester.ast) == 1
    assert origen.tester.ast[0].fields["type"] == "comment"
    assert origen.tester.ast[0].fields["content"] == "Hello Tester!"
  
  def test_repeat(self, clean_eagle, clean_tester):
    origen.tester.set_timeset("simple")
    assert len(origen.tester.ast) == 0
    assert origen.tester.ast.cycle_count == 0
    assert origen.tester.ast.vector_count == 0
    origen.tester.repeat(10)
    assert len(origen.tester.ast) == 1
    assert origen.tester.ast[0].fields["type"] == "vector"
    assert origen.tester.ast[0].fields["repeat"] == 10
    assert origen.tester.ast.cycle_count == 10
    assert origen.tester.ast.vector_count == 1

  def test_multiple_cycles_and_repeat(self, clean_eagle, clean_tester):
    origen.tester.set_timeset("simple")
    assert len(origen.tester.ast) == 0
    assert origen.tester.ast.cycle_count == 0
    assert origen.tester.ast.vector_count == 0
    origen.tester.cycle()
    origen.tester.cycle(repeat=11)
    origen.tester.cycle()
    assert len(origen.tester.ast) == 3
    assert origen.tester.ast.cycle_count == 13
    assert origen.tester.ast.vector_count == 3

def test_adding_frontend_generator(clean_eagle, clean_tester):
  assert "tester_test.PyTestGenerator" not in origen.tester.generators
  origen.tester.register_generator(PyTestGenerator)
  assert "tester_test.PyTestGenerator" in origen.tester.generators

def test_frontend_generators_can_be_targeted():
  origen.tester.clear_targets()
  assert "tester_test.PyTestGenerator" in origen.tester.generators
  assert origen.tester.targets == []
  origen.tester.target("tester_test.PyTestGenerator")
  assert origen.tester.targets == ["tester_test.PyTestGenerator"]

def test_frontend_generators_can_be_targeted_as_class():
  origen.tester.clear_targets()
  assert "tester_test.PyTestGenerator" in origen.tester.generators
  assert origen.tester.targets == []
  origen.tester.target(PyTestGenerator)
  assert origen.tester.targets == ["tester_test.PyTestGenerator"]

def run_pattern():
  origen.tester.cc("Pattern Start!")
  origen.tester.repeat(5)
  origen.tester.cc("Pattern End!")

@pytest.fixture
def tester_target_backend_dummy():
  origen.tester.target("::DummyGenerator")

@pytest.fixture
def tester_target_frontend_dummy():
  try:
    origen.tester.register_generator(PyTestGenerator)
  except OSError:
    # If we get an error back that shows it's already been added, that's fine. Ignore it.
    pass
  origen.tester.target("tester_test.PyTestGenerator")

class TestBackendGenerator:
  def test_generator(self, capfd, clean_eagle, clean_tester, tester_target_backend_dummy):
    origen.tester.set_timeset("simple")
    run_pattern()
    origen.tester.generate()
    out, err = capfd.readouterr()
    assert out == "\n".join([
      "Printing StubAST to console...",
      "  ::DummyGenerator Node 0: Comment - Content: Pattern Start!",
      "  ::DummyGenerator Node 1: Vector - Repeat: 5",
      "  ::DummyGenerator Node 2: Comment - Content: Pattern End!",
      ""
    ])
    assert err == ""

class TestFrontendGenerator:
  def test_generator(self, capfd, clean_eagle, clean_tester, tester_target_frontend_dummy):
    origen.tester.set_timeset("simple")
    run_pattern()
    origen.tester.generate()
    out, err = capfd.readouterr()
    assert out == "\n".join([
      "Printing StubPyAST to console...",
      "  PyTestGenerator: Node 0: Comment - Content: Pattern Start!",
      "  PyTestGenerator: Node 1: Vector - Repeat: 5",
      "  PyTestGenerator: Node 2: Comment - Content: Pattern End!",
      ""
    ])
    assert err == ""
