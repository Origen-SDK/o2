"""
Launches an interactive Python console with origen_metal loaded, it is aliased
to 'om' for less typing
"""

import pathlib
import origen_metal
om = origen_metal

from origen_metal._helpers import interactive
history_file = pathlib.Path(__file__).parent.joinpath("tmp").joinpath(
    "console_history")
interactive.prep_shell(history_file)
interactive.interact(
    banner=
    "\nOrigen Metal Interactive (type Ctrl-D or exit() to close)\n\nType 'om' to access origen_metal\n\n"
)
