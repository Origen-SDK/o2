from pathlib import Path

tests_root = Path(__file__).parent
working_dir = Path(__file__).parent.parent
working_dir_config = working_dir.joinpath("origen.toml")

def tmp_dir(offset=None):
    t = Path(__file__).parent.parent.parent.joinpath('tmp/pytest')
    if offset:
        t = t.joinpath(offset)
    if not t.exists():
        t.mkdir(parents=True, exist_ok=True)
    return t
