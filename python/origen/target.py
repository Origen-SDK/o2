import origen
import _origen

target = None

# Load the target if one is currently set by the application
def load(targets=None):
    if targets == None:
        targets = _origen.app_config()["target"]

    if targets != None:
        for t in targets:
            origen.load_file(_origen.target_file(t, "targets"))
