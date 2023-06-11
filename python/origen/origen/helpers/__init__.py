import importlib.util, pathlib, inspect
import origen
import origen.helpers.num


def try_method(obj, m, args):
    if has_method(obj, m):
        getattr(obj, m)(*args)


def has_method(obj, m):
    return hasattr(obj, m) and callable(getattr(obj, m))

def calling_filename(frame=1):
    return pathlib.Path(inspect.stack()[frame].filename)

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


def continue_on_exception(logger=None, *, method='error'):
    '''
    Decorator to run a given function but catch any exceptions, output them to the logger, but continue onward.
    **This should not be used in place of a ``finally`` block. This is only a helper for methods which may fail
    but which we don't want to entirely stop execution.**

    Use sphinx-arg ``-W`` to turn these back into errors

    This will let system errors or process-killing exception (such as ``SystemExit``) through:

    See Also
    --------
    
    * :python_exception_hierarchy:`See the exception heirachy <>`
    * Examples
      * :func:origen.web.clean
  '''
    def decorator(func):
        def try_func(*args):
            try:
                func(*args)
            except SystemError:
                # Don't catch anything trying to kill the process, especially if multithreading is used, or any gross failure
                # Exits/interrupts are outdie of the Exception class, so only need to let SystemError through
                raise
            except Exception as err:
                # Catch anything else
                l = logger or origen.logger
                getattr(l, method)(str(err))

        return try_func

    return decorator
