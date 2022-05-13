'''
Some basic tests to verify proper integration between origen & origen_metal,
particularly as it related to the _origen_metal compiled library.
'''

import origen, origen_metal
from tests import om_shared

# Grab the dummy RC from origen_metal's tests
with om_shared():
    from om_tests import test_frontend


def test_frontend_is_set():
    assert (origen_metal.frontend.frontend() is not None)


def test_app_rc_can_be_set():
    assert (origen_metal.frontend.frontend().rc is None)
    assert (origen.app.rc is None)

    # The app/frontend should be tied together internally for an origen-app
    rc = test_frontend.TestRevisionControlFrontend.DummyRC()
    origen_metal.frontend.frontend().rc = rc
    assert (origen_metal.frontend.frontend().rc == rc)
    assert (origen.app.rc == rc)

    # Try a dummy method to make sure the full "roundtrip" works
    outcome = origen.app.__rc_init__()
    assert outcome.succeeded is True
    assert outcome.message == "From Dummy RC"
