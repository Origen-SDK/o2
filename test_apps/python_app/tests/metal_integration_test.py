'''
Some basic tests to verify proper integration between origen & origen_metal,
particularly as it related to the _origen_metal compiled library.
'''

import importlib, pathlib, origen, origen_metal

# Grab the dummy RC from origen_metal's tests
om_rc_tests_spec = importlib.util.spec_from_file_location(
    "om_rc_tests",
    str(pathlib.Path(__file__).parent.joinpath("../../../python/origen_metal/tests/test_frontend.py").resolve())
)
om_rc_tests = importlib.util.module_from_spec(om_rc_tests_spec)
om_rc_tests_spec.loader.exec_module(om_rc_tests)

def test_frontend_is_set():
    assert(origen_metal.frontend.frontend() is not None)

def test_app_rc_can_be_set():
    assert(origen_metal.frontend.frontend().rc is None)
    assert(origen.app.rc is None)

    # The app/frontend should be tied together internally for an origen-app
    rc = om_rc_tests.TestRevisionControlFrontend.DummyRC()
    origen_metal.frontend.frontend().rc = rc
    assert(origen_metal.frontend.frontend().rc == rc)
    assert(origen.app.rc == rc)

    # Try a dummy method to make sure the full "roundtrip" works
    outcome = origen.app.__rc_init__()
    assert outcome.succeeded is True
    assert outcome.message == "From Dummy RC"
