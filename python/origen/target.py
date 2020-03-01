import origen
import _origen

target = None

# Load the target if one is currently set by the application
def load(target=None, environment=None):
    app = origen.app
    if target == None:
        target = _origen.app_config()["target"]

    if environment == None:
        environment = _origen.app_config()["environment"]

    if target != None:
        origen.load_file(_origen.target_file(target, "targets"))
        origen.target.target = target

    if environment != None:
        origen.load_file(_origen.target_file(target, "environments"))

def reload():
    if target != None:
        origen.load_file(_origen.target_file(target, "targets"))
