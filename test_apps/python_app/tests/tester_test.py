import pytest
import origen, _origen  # pylint: disable=import-error
from tests.shared import *
from tests.shared.python_like_apis import Fixture_ListLikeAPI  # pylint: disable=import-error
from origen.generator.tester_api import TesterAPI  # pylint: disable=import-error
from origen.generator.processor import Return  # pylint: disable=import-error


# Test tester used to test the frontend <-> backend tester hooks
class PyTestRenderer(TesterAPI):
    def __init__(self):
        TesterAPI.__init__(self)
        self.i = 0

    def on_test(self, node):
        print("Printing StubPyAST to console...")
        return Return.process_children

    def on_comment(self, node):
        node_str = f"Comment - Content: {node['attrs'][1][1]}"
        print(f"  PyTestRenderer: Node {self.i}: {node_str}")
        self.i += 1
        return Return.unmodified

    def on_cycle(self, node):
        node_str = f"Vector - Repeat: {node['attrs'][1][0]}"
        print(f"  PyTestRenderer: Node {self.i}: {node_str}")
        self.i += 1
        return Return.unmodified


# Test tester used to test the frontend <-> backend interceptor hooks
class PyTestRendererWithInterceptor(TesterAPI):
    def __init__(self):
        TesterAPI.__init__(self)
        self.i = 0

    def on_test(self, node):
        print("Printing StubPyAST to console...")
        return Return.process_children

    def on_comment(self, node):
        node_str = f"Comment - Content: {node['attrs'][1][1]}"
        print(f"  PyTestRendererWithInterceptor: Node {self.i}: {node_str}")
        self.i += 1
        return Return.unmodified

    def on_cycle(self, node):
        node_str = f"Vector - Repeat: {node['attrs'][1][0]}"
        print(f"  PyTestRendererWithInterceptor: Node {self.i}: {node_str}")
        self.i += 1
        return Return.unmodified

    def cc(self, node):
        node_str = f"Comment - Content: {node['attrs'][1][1]}"
        print(f"Intercepted By PyTestRendererWithInterceptor: {node_str}")


# Test tester which ignores everything and uses the meta interface only.
# Not overly practical, but this should work nonetheless.
class PyTestMetaRenderer(TesterAPI):
    def __init__(self):
        TesterAPI.__init__(self)
        self.nodes = []

    def on_test(self, _node):
        print("Printing StubPyAST to console...")
        for i, n in enumerate(self.nodes):
            print(f"  PyTestMetaRenderer: {i}: {n}")
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
    assert origen.tester.testers == backend_testers
    assert origen.tester.timeset is None


def test_setting_a_timeset(clean_eagle, clean_dummy):
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


def test_exception_on_unknown_timeset(clean_eagle, clean_dummy):
    with pytest.raises(RuntimeError):
        origen.tester.set_timeset("blah")


def test_setting_timeset_with_instance(clean_eagle, clean_dummy):
    assert origen.tester.timeset is None
    origen.tester.set_timeset(origen.dut.timesets["simple"])
    assert origen.tester.timeset.name == "simple"


def test_setting_targets(clean_eagle, clean_tester):
    assert origen.tester.targets == []
    origen.tester.target("DummyRenderer")
    assert origen.tester.targets == ["DUMMYRENDERER"]


def test_resetting_targets():
    assert origen.tester.targets == ["DUMMYRENDERER"]
    origen.tester.reset()
    assert origen.tester.targets == []


def test_exception_on_duplicate_targets(clean_eagle, clean_tester):
    origen.tester.target("DummyRenderer")
    with pytest.raises(RuntimeError):
        origen.tester.target("DummyRenderer")


def test_exception_on_unknown_target(clean_eagle, clean_tester):
    with pytest.raises(RuntimeError):
        origen.tester.target("blah")


class TestTesterAPI:
    def test_cycle(self, clean_eagle, clean_dummy):
        assert len(origen.test_ast()["children"]) == 0
        origen.tester.set_timeset("simple")
        check_last_node_type("SetTimeset")
        origen.tester.cycle()
        check_last_node_type("Cycle")

    def test_multiple_cycles(self, clean_eagle, clean_dummy):
        origen.tester.set_timeset("simple")
        origen.tester.cycle()
        origen.tester.cycle()
        origen.tester.cycle()
        assert len(origen.test_ast()["children"]) == 4

    def test_cc(self, clean_eagle, clean_dummy):
        origen.tester.set_timeset("simple")
        origen.tester.cc("Hello Tester!")
        check_last_node_type("Comment")

    def test_repeat(self, clean_eagle, clean_dummy):
        origen.tester.set_timeset("simple")
        origen.tester.repeat(10)
        n = get_last_node()
        assert n["attrs"][0] == "Cycle"
        assert n["attrs"][1][0] == 10


def test_adding_frontend_renderer(clean_eagle, clean_tester):
    t = "CUSTOM::tests.tester_test.PyTestRenderer"
    assert t not in origen.tester.testers
    origen.tester.register_tester(PyTestRenderer)
    assert t in origen.tester.testers


def test_frontend_testers_can_be_targeted():
    origen.tester.reset()
    tname = "tests.tester_test.PyTestRenderer"
    t = f"CUSTOM::{tname}"
    assert t in origen.tester.testers
    assert origen.tester.targets == []
    origen.tester.target(t)
    assert origen.tester.targets == [f'CUSTOM(\"{tname}\")']


def test_frontend_testers_can_be_targeted_as_class():
    origen.tester.reset()
    tname = "tests.tester_test.PyTestRenderer"
    t = f"CUSTOM::{tname}"
    assert t in origen.tester.testers
    assert origen.tester.targets == []
    origen.tester.target(PyTestRenderer)
    assert origen.tester.targets == [f'CUSTOM(\"{tname}\")']


def run_pattern():
    origen.tester.cc("Pattern Start!")
    origen.tester.repeat(5)
    origen.tester.cc("Pattern End!")


@pytest.fixture
def tester_target_backend_dummy():
    origen.tester.target("DummyRenderer")


@pytest.fixture
def tester_target_frontend_dummy():
    try:
        origen.tester.register_tester(PyTestRenderer)
    except RuntimeError:
        # If we get an error back that shows it's already been added, that's fine. Ignore it.
        pass
    origen.tester.target("CUSTOM::tests.tester_test.PyTestRenderer")


class TestBackendRenderer:
    def test_tester(self, capfd, clean_eagle, clean_tester,
                    tester_target_backend_dummy):
        origen.tester.set_timeset("simple")
        run_pattern()
        origen.tester.render_pattern()
        out, err = capfd.readouterr()
        assert out == "\n".join([
            "Printing StubAST to console...",
            "  ::DummyRenderer Node 0: Comment - Content: Pattern Start!",
            "  ::DummyRenderer Node 1: Vector - Repeat: 5, Timeset: 'simple'",
            "  ::DummyRenderer Node 2: Comment - Content: Pattern End!", ""
        ])
        assert err == ""

    def test_interceptors_on_backend(self, capfd, clean_eagle, clean_tester):
        origen.tester.target("DummyRendererWithInterceptors")
        origen.tester.set_timeset("simple")
        run_pattern()
        origen.tester.render_pattern()
        out, err = capfd.readouterr()
        assert out == "\n".join([
            "Comment intercepted by DummyRendererWithInterceptors!",
            "Vector intercepted by DummyRendererWithInterceptors!",
            "Comment intercepted by DummyRendererWithInterceptors!",
            "Printing StubAST to console...",
            "  ::DummyRendererWithInterceptors Node 0: Comment - Content: Pattern Start!",
            "  ::DummyRendererWithInterceptors Node 1: Vector - Repeat: 5, Timeset: 'simple'",
            "  ::DummyRendererWithInterceptors Node 2: Comment - Content: Pattern End!",
            ""
        ])
        assert err == ""


class TestFrontendRenderer:
    def test_tester(self, capfd, clean_eagle, clean_tester,
                    tester_target_frontend_dummy):
        origen.tester.set_timeset("simple")
        run_pattern()
        origen.tester.render_pattern()
        out, err = capfd.readouterr()
        assert out == "\n".join([
            "Printing StubPyAST to console...",
            "  PyTestRenderer: Node 0: Comment - Content: Pattern Start!",
            "  PyTestRenderer: Node 1: Vector - Repeat: 5",
            "  PyTestRenderer: Node 2: Comment - Content: Pattern End!", ""
        ])
        assert err == ""

    def test_renderer_with_interceptors(self, capfd, clean_eagle,
                                        clean_tester):
        origen.tester.register_tester(PyTestRendererWithInterceptor)
        origen.tester.target(PyTestRendererWithInterceptor)
        origen.tester.set_timeset("simple")
        run_pattern()
        origen.tester.render_pattern()
        out, err = capfd.readouterr()
        assert out == "\n".join([
            "Intercepted By PyTestRendererWithInterceptor: Comment - Content: Pattern Start!",
            "Intercepted By PyTestRendererWithInterceptor: Comment - Content: Pattern End!",
            "Printing StubPyAST to console...",
            "  PyTestRendererWithInterceptor: Node 0: Comment - Content: Pattern Start!",
            "  PyTestRendererWithInterceptor: Node 1: Vector - Repeat: 5",
            "  PyTestRendererWithInterceptor: Node 2: Comment - Content: Pattern End!",
            ""
        ])
        assert err == ""

    def test_meta_renderer(self, capfd, clean_eagle, clean_tester):
        origen.tester.register_tester(PyTestMetaRenderer)
        origen.tester.target(PyTestMetaRenderer)
        origen.tester.set_timeset("simple")
        run_pattern()
        origen.tester.render_pattern()
        out, err = capfd.readouterr()
        assert out == "\n".join([
            "Printing StubPyAST to console...",
            "  PyTestMetaRenderer: 0: Meta CC: Pattern Start!",
            "  PyTestMetaRenderer: 1: Meta Cycle: 5",
            "  PyTestMetaRenderer: 2: Meta CC: Pattern End!", ""
        ])
        assert err == ""


def test_targeted_renderer_ordering(capfd, clean_eagle, clean_tester):
    origen.tester.register_tester(PyTestRendererWithInterceptor)
    origen.tester.target(PyTestRendererWithInterceptor)
    origen.tester.target("DummyRendererWithInterceptors")
    origen.tester.set_timeset("simple")
    run_pattern()
    origen.tester.render_pattern()
    out, err = capfd.readouterr()
    assert out == "\n".join([
        "Intercepted By PyTestRendererWithInterceptor: Comment - Content: Pattern Start!",
        "Comment intercepted by DummyRendererWithInterceptors!",
        "Vector intercepted by DummyRendererWithInterceptors!",
        "Intercepted By PyTestRendererWithInterceptor: Comment - Content: Pattern End!",
        "Comment intercepted by DummyRendererWithInterceptors!",
        "Printing StubPyAST to console...",
        "  PyTestRendererWithInterceptor: Node 0: Comment - Content: Pattern Start!",
        "  PyTestRendererWithInterceptor: Node 1: Vector - Repeat: 5",
        "  PyTestRendererWithInterceptor: Node 2: Comment - Content: Pattern End!",
        ""
        "Printing StubAST to console...",
        "  ::DummyRendererWithInterceptors Node 0: Comment - Content: Pattern Start!",
        "  ::DummyRendererWithInterceptors Node 1: Vector - Repeat: 5, Timeset: 'simple'",
        "  ::DummyRendererWithInterceptors Node 2: Comment - Content: Pattern End!",
        ""
    ])
    assert err == ""


def test_targeted_renderer_reverse_ordering(capfd, clean_eagle, clean_tester):
    origen.tester.register_tester(PyTestRendererWithInterceptor)
    origen.tester.target("DummyRendererWithInterceptors")
    origen.tester.target(PyTestRendererWithInterceptor)
    origen.tester.set_timeset("simple")
    run_pattern()
    origen.tester.render_pattern()
    out, err = capfd.readouterr()
    assert out == "\n".join([
        "Comment intercepted by DummyRendererWithInterceptors!",
        "Intercepted By PyTestRendererWithInterceptor: Comment - Content: Pattern Start!",
        "Vector intercepted by DummyRendererWithInterceptors!",
        "Comment intercepted by DummyRendererWithInterceptors!",
        "Intercepted By PyTestRendererWithInterceptor: Comment - Content: Pattern End!",
        "Printing StubAST to console...",
        "  ::DummyRendererWithInterceptors Node 0: Comment - Content: Pattern Start!",
        "  ::DummyRendererWithInterceptors Node 1: Vector - Repeat: 5, Timeset: 'simple'",
        "  ::DummyRendererWithInterceptors Node 2: Comment - Content: Pattern End!",
        ""
        "Printing StubPyAST to console...",
        "  PyTestRendererWithInterceptor: Node 0: Comment - Content: Pattern Start!",
        "  PyTestRendererWithInterceptor: Node 1: Vector - Repeat: 5",
        "  PyTestRendererWithInterceptor: Node 2: Comment - Content: Pattern End!",
        ""
    ])
    assert err == ""
