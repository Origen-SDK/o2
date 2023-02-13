import origen
import _origen

current_targets = None
first_load_done = False
setup_pending = False

def __getattr__(name: str):
    if name == "current":
        return current_targets
    else:
        raise AttributeError(f"module {__name__!r} has no attribute {name!r}")

# Setup the targets to be applied at future calls to target.load, but without
# actually loading them at this time
def setup(targets=None):
    global current_targets, setup_pending

    if targets is None:
        targets = _origen.app_config()["target"]
        if targets is None:
            current_targets = []
            return None
    if isinstance(targets, str):
        targets = [targets]
    elif not isinstance(targets, list):
        targets = list(targets)
    setup_pending = True
    current_targets = targets

# Load the target(s) previously registered by setup or as given
def load(*targets):
    global first_load_done, setup_pending, current_targets

    if len(targets) > 0:
        setup(targets)
    if not first_load_done and not setup_pending:
        setup(targets or None)
    origen.dut = None
    origen._target_loading = True
    _origen.prepare_for_target_load()
    for t in current_targets:
        if callable(t):
            t()
        else:
            if t is not None:
                origen.load_file(_origen.target_file(t, "targets"))
    origen._target_loading = False
    first_load_done = True
    setup_pending = False


# Load the target(s) previously registered by setup but only if they
# have not been loaded yet
def load_unless_loaded():
    if not first_load_done:
        load()
