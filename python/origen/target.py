import origen
import _origen

global current_targets
current_targets = []

# Load the target if one is currently set by the application
def load(targets=None):
    global current_targets
    if targets == None:
        targets = _origen.app_config()["target"]

    if targets != None:
        for t in targets:
            origen.load_file(_origen.target_file(t, "targets"))
    current_targets = targets

def reload():
    for t in current_targets:
        origen.load_file(_origen.target_file(t, "targets"))