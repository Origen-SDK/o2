import inspect
import multiprocessing as mp

# TODO swap out assert?

def in_new_proc(func=None, mod=None, func_kwargs=None, expect_fail=False):
    if func is None:
        func = getattr(mod, inspect.stack()[1].function)
    context = mp.get_context("spawn")
    q = context.Queue()

    args=(q, func_kwargs)
    proc = context.Process(target=func, args=args)
    proc.start()
    proc.join()
    results = {}
    while not q.empty():
        # Convert the populated Queue to a dictionary
        obj = q.get()
        results[obj[0]] = obj[1]
    if expect_fail:
        assert proc.exitcode == 1
    else:
        assert proc.exitcode == 0
    return results
