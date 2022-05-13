import pathlib, pytest
from origen_metal._origen_metal import frontend

def tmp_dir():
    t = pathlib.Path(__file__).parent.parent.parent.joinpath('tmp/pytest')
    if not t.exists():
        t.mkdir(parents=True, exist_ok=True)
    return t

@pytest.fixture
def needs_frontend():
    if frontend.frontend() is None:
        frontend.initialize()
        yield(frontend.frontend())
        frontend.reset()
    else:
        yield(frontend.frontend())

@pytest.fixture
def fresh_frontend():
    frontend.reset()
    frontend.initialize()
    return frontend.frontend()
