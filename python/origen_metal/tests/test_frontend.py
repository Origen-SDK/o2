import pytest
from origen_metal._origen_metal import __test__
from origen_metal.utils import revision_control
from origen_metal.framework import Outcome
from origen_metal._origen_metal import frontend


def init_frontend():
    frontend.reset()
    assert fe() is None
    assert frontend.initialize() is True
    assert isinstance(fe(), frontend.PyFrontend)


def fe():
    return frontend.frontend()


@pytest.fixture
def frontend_init():
    init_frontend()


def test_frontend_is_accessible():
    assert fe() is None
    init_frontend()


def test_multiple_frontend_initializes_are_benign(frontend_init):
    assert frontend.initialize() is False


class TestRevisionControlFrontend:
    class DummyRC(revision_control.RevisionControl):
        def init(self):
            return Outcome(succeeded=True, message="From Dummy RC")

    @pytest.fixture
    def dummy_rc(self):
        fe().rc = TestRevisionControlFrontend.DummyRC()

    def test_frontend_rc_driver(frontend_init):
        assert fe().rc is None
        assert fe().revision_control is None

    def test_frontend_rc_driver_can_be_set(frontend_init, dummy_rc):
        assert isinstance(fe().rc, TestRevisionControlFrontend.DummyRC)
        assert isinstance(fe().revision_control,
                          TestRevisionControlFrontend.DummyRC)

    def test_frontend_rc_can_be_called_by_the_backend(frontend_init, dummy_rc):
        outcome = __test__.rc_init_from_metal()
        assert outcome.succeeded is True
        assert outcome.message == "From Dummy RC"
