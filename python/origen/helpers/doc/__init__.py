import inspect
''' Grabs all the non-variable members whose source is from the given module'''


def internal_members(mod):
    return list(member[0] for member in filter(
        lambda m: (hasattr(m[1], '__module__') and m[1].__module__ == mod.
                   __name__), inspect.getmembers(mod)))
