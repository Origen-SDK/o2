import importlib.util, pathlib

def try_method(obj, m, args):
  if has_method(obj, m):
    getattr(obj, m)(*args)

def has_method(obj, m):
  return hasattr(obj, m) and callable(getattr(obj, m))

# Import a module from a file, as described here: https://stackoverflow.com/questions/67631/how-to-import-a-module-given-the-full-path
# The name can be whatever. If no name is given, then the file name is used
def mod_from_file(path, name=None):
  path = pathlib.Path(path)
  if not name:
    name = path.stem
  spec = importlib.util.spec_from_file_location(name, str(path))
  mod = importlib.util.module_from_spec(spec)
  spec.loader.exec_module(mod)
  return mod