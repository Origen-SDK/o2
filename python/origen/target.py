import origen
import _origen

current_targets = []
first_load_done = False


# Setup the targets to be applied at future calls to target.load, but without
# actually loading them at this time
def setup(targets=None):
    global current_targets
    if targets == None:
        targets = _origen.app_config()["target"]
        if targets == None:
            current_targets = []
            return None
    if not isinstance(targets, list):
        targets = [targets]
    current_targets = targets


# Load the target(s) previously registered by setup or as given
def load(targets=None):
    global first_load_done
    if targets is not None:
        setup(targets)
    first_load_done = True
    origen.dut = None
    origen._target_loading = True
    _origen.prepare_for_target_load()
    if current_targets != None:
        for t in current_targets:
            if callable(t):
                t()
            else:
                origen.load_file(_origen.target_file(t, "targets"))
    origen._target_loading = False


# Load the target(s) previously registered by setup but only if they
# have not been loaded yet
def load_unless_loaded():
    if not first_load_done:
        load()
