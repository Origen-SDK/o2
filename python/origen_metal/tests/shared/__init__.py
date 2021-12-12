import pathlib

def tmp_dir():
    t = pathlib.Path(__file__).parent.parent.parent.joinpath('tmp/pytest')
    if not t.exists():
        t.mkdir(parents=True, exist_ok=True)
    return t
