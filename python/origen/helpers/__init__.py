def try_method(obj, m, args):
  if has_method(obj, m):
    getattr(obj, m)(*args)

def has_method(obj, m):
  return hasattr(obj, m) and callable(getattr(obj, m))