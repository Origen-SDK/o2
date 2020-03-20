import pytest
import origen, _origen # pylint: disable=import-error
from tests.shared import clean_eagle, clean_tester, check_last_node_type, get_last_node # pylint: disable=import-error
from tests.shared.python_like_apis import Fixture_ListLikeAPI # pylint: disable=import-error
from origen.generator.tester_api import TesterAPI # pylint: disable=import-error
from origen.generator.processor import Return # pylint: disable=import-error

# Test generator used to test the frontend <-> backend generator hooks
class PyTestGenerator(TesterAPI):
  def __init__(self):
    TesterAPI.__init__(self)
    self.i = 0

  def on_test(self, node):
    print("Printing StubPyAST to console...")
    return Return.process_children

  def on_comment(self, node):
    node_str = f"Comment - Content: {node['attrs'][1][1]}"
    print(f"  PyTestGenerator: Node {self.i}: {node_str}")
    self.i += 1
    return Return.unmodified

  def on_cycle(self, node):
    node_str = f"Vector - Repeat: {node['attrs'][1][0]}"
    print(f"  PyTestGenerator: Node {self.i}: {node_str}")
    self.i += 1
    return Return.unmodified

# Test generator used to test the frontend <-> backend interceptor hooks
class PyTestGeneratorWithInterceptor(TesterAPI):
  def __init__(self):
    TesterAPI.__init__(self)
    self.i = 0

  def on_test(self, node):
    print("Printing StubPyAST to console...")
    return Return.process_children

  def on_comment(self, node):
    node_str = f"Comment - Content: {node['attrs'][1][1]}"
    print(f"  PyTestGeneratorWithInterceptor: Node {self.i}: {node_str}")
    self.i += 1
    return Return.unmodified

  def on_cycle(self, node):
    node_str = f"Vector - Repeat: {node['attrs'][1][0]}"
    print(f"  PyTestGeneratorWithInterceptor: Node {self.i}: {node_str}")
    self.i += 1
    return Return.unmodified

  def cc(self, node):
    node_str = f"Comment - Content: {node['attrs'][1][1]}"
    print(f"Intercepted By PyTestGeneratorWithInterceptor: {node_str}")
    #ast[-1].set('content', f"Intercepted By PyTestGeneratorWithInterceptor: {ast[-1].fields['content']}")

# Test generator which ignores everything and uses the meta interface only.
# Not overly practical, but this should work nonetheless.
class PyTestMetaGenerator(TesterAPI):
  def __init__(self):
    TesterAPI.__init__(self)
    self.nodes = []

  def on_test(self, _node):
    print("Printing StubPyAST to console...")
    for i, n in enumerate(self.nodes):
      print(f"  PyTestMetaGenerator: {i}: {n}")
    return Return.unmodified

  def cycle(self, node):
    self.nodes.append(f"Meta Cycle: {node['attrs'][1][0]}")
    return Return.unmodified

  def cc(self, node):
    self.nodes.append(f"Meta CC: {node['attrs'][1][1]}")
    return Return.unmodified

def test_init_state(clean_eagle, clean_tester):
  # The 'clean_tester' fixture has a number of asserts itself,
  # but just in case those change unbeknownst to this method, double check the initial state here.
  assert origen.tester
  assert origen.tester.targets == []
  assert origen.tester.generators == ["::DummyGenerator", "::DummyGeneratorWithInterceptors", "::V93K::ST7", "::Simulator"]
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

class TestTesterAPI:
  def test_cycle(self, clean_eagle, clean_tester):
    assert len(origen.test_ast()["children"]) == 0
    origen.tester.set_timeset("simple")
    check_last_node_type("SetTimeset")
    origen.tester.cycle()
    check_last_node_type("Cycle")

  def test_multiple_cycles(self, clean_eagle, clean_tester):
    origen.tester.set_timeset("simple")
    origen.tester.cycle()
    origen.tester.cycle()
    origen.tester.cycle()
    assert len(origen.test_ast()["children"]) == 4

  def test_cc(self, clean_eagle, clean_tester):
    origen.tester.set_timeset("simple")
    origen.tester.cc("Hello Tester!")
    check_last_node_type("Comment")
  
  def test_repeat(self, clean_eagle, clean_tester):
    origen.tester.set_timeset("simple")
    origen.tester.repeat(10)
    n = get_last_node()
    assert n["attrs"][0] == "Cycle"
    assert n["attrs"][1][0] == 10

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
      "  ::DummyGenerator Node 1: Vector - Repeat: 5, Timeset: 'simple'",
      "  ::DummyGenerator Node 2: Comment - Content: Pattern End!",
      ""
    ])
    assert err == ""
  
  def test_interceptors_on_backend(self, capfd, clean_eagle, clean_tester):
    origen.tester.target("::DummyGeneratorWithInterceptors")
    origen.tester.set_timeset("simple")
    run_pattern()
    origen.tester.generate()
    out, err = capfd.readouterr()
    assert out == "\n".join([
      "Comment intercepted by DummyGeneratorWithInterceptors!",
      "Vector intercepted by DummyGeneratorWithInterceptors!",
      "Comment intercepted by DummyGeneratorWithInterceptors!",
      "Printing StubAST to console...",
      "  ::DummyGeneratorWithInterceptors Node 0: Comment - Content: Pattern Start!",
      "  ::DummyGeneratorWithInterceptors Node 1: Vector - Repeat: 5, Timeset: 'simple'",
      "  ::DummyGeneratorWithInterceptors Node 2: Comment - Content: Pattern End!",
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

  def test_generator_with_interceptors(self, capfd, clean_eagle, clean_tester):
    origen.tester.register_generator(PyTestGeneratorWithInterceptor)
    origen.tester.target(PyTestGeneratorWithInterceptor)
    origen.tester.set_timeset("simple")
    run_pattern()
    origen.tester.generate()
    out, err = capfd.readouterr()
    assert out == "\n".join([
      "Intercepted By PyTestGeneratorWithInterceptor: Comment - Content: Pattern Start!",
      "Intercepted By PyTestGeneratorWithInterceptor: Comment - Content: Pattern End!",
      "Printing StubPyAST to console...",
      "  PyTestGeneratorWithInterceptor: Node 0: Comment - Content: Pattern Start!",
      "  PyTestGeneratorWithInterceptor: Node 1: Vector - Repeat: 5",
      "  PyTestGeneratorWithInterceptor: Node 2: Comment - Content: Pattern End!",
      ""
    ])
    assert err == ""

  def test_meta_generator(self, capfd, clean_eagle, clean_tester):
    origen.tester.register_generator(PyTestMetaGenerator)
    origen.tester.target(PyTestMetaGenerator)
    origen.tester.set_timeset("simple")
    run_pattern()
    origen.tester.generate()
    out, err = capfd.readouterr()
    assert out == "\n".join([
      "Printing StubPyAST to console...",
      "  PyTestMetaGenerator: 0: Meta CC: Pattern Start!",
      "  PyTestMetaGenerator: 1: Meta Cycle: 5",
      "  PyTestMetaGenerator: 2: Meta CC: Pattern End!",
      ""
    ])
    assert err == ""

def test_targeted_generator_ordering(capfd, clean_eagle, clean_tester):
    origen.tester.register_generator(PyTestGeneratorWithInterceptor)
    origen.tester.target(PyTestGeneratorWithInterceptor)
    origen.tester.target("::DummyGeneratorWithInterceptors")
    origen.tester.set_timeset("simple")
    run_pattern()
    origen.tester.generate()
    out, err = capfd.readouterr()
    assert out == "\n".join([
      "Intercepted By PyTestGeneratorWithInterceptor: Comment - Content: Pattern Start!",
      "Comment intercepted by DummyGeneratorWithInterceptors!",
      "Vector intercepted by DummyGeneratorWithInterceptors!",
      "Intercepted By PyTestGeneratorWithInterceptor: Comment - Content: Pattern End!",
      "Comment intercepted by DummyGeneratorWithInterceptors!",
      "Printing StubPyAST to console...",
      "  PyTestGeneratorWithInterceptor: Node 0: Comment - Content: Pattern Start!",
      "  PyTestGeneratorWithInterceptor: Node 1: Vector - Repeat: 5",
      "  PyTestGeneratorWithInterceptor: Node 2: Comment - Content: Pattern End!",
      ""
      "Printing StubAST to console...",
      "  ::DummyGeneratorWithInterceptors Node 0: Comment - Content: Pattern Start!",
      "  ::DummyGeneratorWithInterceptors Node 1: Vector - Repeat: 5, Timeset: 'simple'",
      "  ::DummyGeneratorWithInterceptors Node 2: Comment - Content: Pattern End!",
      ""
    ])
    assert err == ""

def test_targeted_generator_reverse_ordering(capfd, clean_eagle, clean_tester):
    origen.tester.register_generator(PyTestGeneratorWithInterceptor)
    origen.tester.target("::DummyGeneratorWithInterceptors")
    origen.tester.target(PyTestGeneratorWithInterceptor)
    origen.tester.set_timeset("simple")
    run_pattern()
    origen.tester.generate()
    out, err = capfd.readouterr()
    assert out == "\n".join([
      "Comment intercepted by DummyGeneratorWithInterceptors!",
      "Intercepted By PyTestGeneratorWithInterceptor: Comment - Content: Pattern Start!",
      "Vector intercepted by DummyGeneratorWithInterceptors!",
      "Comment intercepted by DummyGeneratorWithInterceptors!",
      "Intercepted By PyTestGeneratorWithInterceptor: Comment - Content: Pattern End!",
      "Printing StubAST to console...",
      "  ::DummyGeneratorWithInterceptors Node 0: Comment - Content: Pattern Start!",
      "  ::DummyGeneratorWithInterceptors Node 1: Vector - Repeat: 5, Timeset: 'simple'",
      "  ::DummyGeneratorWithInterceptors Node 2: Comment - Content: Pattern End!",
      ""
      "Printing StubPyAST to console...",
      "  PyTestGeneratorWithInterceptor: Node 0: Comment - Content: Pattern Start!",
      "  PyTestGeneratorWithInterceptor: Node 1: Vector - Repeat: 5",
      "  PyTestGeneratorWithInterceptor: Node 2: Comment - Content: Pattern End!",
      ""
    ])
    assert err == ""
