__version__ = '0.1.0'

import _origen;

from pathlib import Path

root = Path(_origen.root())
version = _origen.version()
