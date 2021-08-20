"""
Launches an interactive Python console with origen_metal loaded, it is aliased
to 'om' for less typing
"""
from pathlib import Path
import atexit, readline, os, rlcompleter

historyPath = Path(__file__).parent.joinpath("tmp").joinpath("console_history")


def save_history(historyPath=historyPath):
    import readline

    readline.write_history_file(historyPath)


if os.path.exists(historyPath):
    readline.read_history_file(str(historyPath))

atexit.register(save_history)

del atexit, readline, os, rlcompleter, save_history, historyPath

import code
import origen_metal
om = origen_metal

code.interact(
    banner=
    f"\nOrigen Metal Interactive (type Ctrl-D or exit() to close)\n\nType 'om' to access origen_metal\n\n",
    local=dict(globals()),
    exitmsg="",
)
