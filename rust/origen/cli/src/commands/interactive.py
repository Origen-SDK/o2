import atexit
import os
import readline
import rlcompleter

def save_history(historyPath=historyPath):
    import readline
    readline.write_history_file(historyPath)

if os.path.exists(historyPath):
    readline.read_history_file(historyPath)

atexit.register(save_history)
del os, atexit, readline, rlcompleter, save_history, historyPath

import origen;
import code;
from origen import dut, tester;
code.interact(banner=f"Origen {origen.version}", local=locals(), exitmsg="")
