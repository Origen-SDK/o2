import origen
import _origen

# Load the target if one is currently set by the application
def load(target=None, environment=None):
    app = origen.app
    if target == None:
        target = app.config["target"]
    if environment == None:
        environment = app.config["environment"]

    if target != None:
        # Also see:
        #https://docs.python.org/2/library/imp.html#imp.load_source
        tfile = _origen.target_file(target, "targets")
        global_vars = {}
        local_vars = {}
        with open(tfile) as f:
            code = compile(f.read(), tfile, 'exec')
            exec(code, global_vars, local_vars)

    if environment != None:
        # Also see:
        #https://docs.python.org/2/library/imp.html#imp.load_source
        tfile = _origen.target_file(environment, "environments")
        global_vars = {}
        local_vars = {}
        with open(tfile) as f:
            code = compile(f.read(), tfile, 'exec')
            exec(code, global_vars, local_vars)
