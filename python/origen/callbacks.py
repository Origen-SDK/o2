import origen, inspect, enum


class UnloadOn(enum.Enum):
    DUT_CHANGE = "dut_change"


def register_new(name, source=None):
    _callbacks.register_new(name, source)


def register_callback(name, *, source=None):
    _callbacks.register_new(name, source)


def listen_for(name, *, source=None, unload_on=None):
    def inner(func):
        _callbacks.register_listener(name, source, func)

    return inner


def emit(name, source=None, args=None, kwargs=None):
    return _callbacks.emit(name, source, args, kwargs)


def unload(phase):
    _callbacks.unload(phase)


def unload_on_dut_change():
    _callbacks.unload(UnloadOn.DUT_CHANGE)


class ProxyFunctions:
    def __init__(self):
        self.proxy_functions = {}

    def append(self, callback, func, *, unload_on=None):
        klass = (inspect.getmodule(func), func.__qualname__.rsplit('.', 1)[0])
        if klass not in self.proxy_functions:
            self.proxy_functions[klass] = {'callbacks': {}}
        if callback not in self.proxy_functions[klass]['callbacks']:
            self.proxy_functions[klass]['callbacks'][callback] = []
        self.proxy_functions[klass]['callbacks'][callback].append({
            "func":
            func,
            "unload_on":
            unload_on
        })

    def apply(self, slf, *, force=False):
        handlers = []
        for k in slf.__class__.mro():
            klass_tuple = (inspect.getmodule(k), k.__qualname__.rsplit('.',
                                                                       1)[0])
            origen.logger.debug(f"Applying callbacks for class {klass_tuple}")
            if klass_tuple in self.proxy_functions:
                for cb, func_list in self.proxy_functions[klass_tuple][
                        'callbacks'].items():
                    for func in func_list:
                        n = func["func"].__name__
                        if n not in handlers:
                            _callbacks.register_listener(
                                cb,
                                None,
                                getattr(slf, n),
                                unload_on=func["unload_on"])
                            handlers.append(n)


proxies = ProxyFunctions()


class Callback:
    class Listener:
        def __init__(self, source, func, *, unload_on=None):
            self.source = source
            self.func = func
            if unload_on:
                if isinstance(unload_on, list):
                    self.unload_on = unload_on
                else:
                    self.unload_on = [unload_on]
            else:
                self.unload_on = []

        def run(self, args, kwargs):
            if args is None:
                args = []
            if kwargs is None:
                kwargs = {}
            return self.func(*args, **kwargs)

    def __init__(self, name, registered_at):
        self.name = name
        self.registered_at = registered_at
        self.emitted_at = []
        self.listeners = []

    def emit(self, caller, args, kwargs):
        retn = list()
        for listener in self.listeners:
            retn.append(listener.run(args, kwargs))
            self.emitted_at.append(caller)
        return retn

    def register_listener(self, source, func, *, unload_on=None):
        self.listeners.append(
            Callback.Listener(source, func, unload_on=unload_on))

    def unload(self, phase):
        self.listeners = list(
            filter(lambda f: phase not in f.unload_on, self.listeners))


class Callbacks:
    def __init__(self):
        self.callbacks = {}

        # Temporary - move these somewhere else probably
        # self.register_new("on_app_init", None)
        self.register_new("toplevel__startup", None)
        self.register_new("toplevel__shutdown", None)
        self.register_new("toplevel__initialized", None)
        self.register_new("controller__startup", None)
        self.register_new("controller__shutdown", None)

    def register_new(self, name, source):
        self.callbacks[name] = Callback(name, source)

    def emit(self, name, caller, args, kwargs):
        return self.callbacks[name].emit(caller, args, kwargs)

    # def validate_callback(self):
    #     ...

    def register_listener(self, name, source, func, *, unload_on=None):
        origen.logger.debug(
            f"Registering listener function '{func.__qualname__}' for callback '{name}'"
        )
        self.callbacks[name].register_listener(source,
                                               func,
                                               unload_on=unload_on)

    def unload(self, phase):
        origen.logger.trace(f"Unloading listeners for phase '{phase}'")
        for n, cb in self.callbacks.items():
            cb.unload(phase)


_callbacks = Callbacks()
