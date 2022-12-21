import inspect
from origen_metal._origen_metal import frontend
from abc import ABC, ABCMeta, abstractclassmethod


class DataStoreAPIMeta(ABCMeta):
    def __new__(cls, name, *args, **kwargs):
        # TODO likely don't need metaclass anymore
        return ABCMeta(name, *args, **kwargs)


class DataStoreAPI(ABC, metaclass=DataStoreAPIMeta):
    __origen_supported_features__ = {}

    @classmethod
    def __register_origen_supported_feature__(cls, feature, func):
        kls = (func.__module__, func.__qualname__.rsplit(".", 1)[0])
        if kls not in cls.__origen_supported_features__:
            cls.__origen_supported_features__[kls] = {}
        cls.__origen_supported_features__[kls][feature] = func

    @classmethod
    def populate_user(cls, func):
        # TODO need to make these enums/common
        cls.__register_origen_supported_feature__("populate_user", func)
        return func

    # TEST_NEEDED
    @classmethod
    def validate_password(cls, func):
        cls.__register_origen_supported_feature__("validate_password", func)
        return func

    @staticmethod
    def __new__(cls, *args, **kwargs):
        return ABC.__new__(cls)

    class OperationNotSupported(RuntimeError):
        def __init__(self, ds, op):
            self.ds_class = ds.__class__
            self.op = op
            super().__init__(
                f"Data store '{ds.name}' (of class '{ds.__class__}' does not support '{op}' operations"
            )

    def __init__(self, *args, **kwargs):
        self._stale = False
        self._orphaned = False
        self._supported_features_ = {}

    def _set_name_(self, name):
        self._name = name

    def _set_category_(self, category_name):
        self._category_name = category_name

    def __getattribute__(self, __name: str):
        if super().__getattribute__("_stale"):
            n = super().__getattribute__("_name")
            c = super().__getattribute__("_category_name")
            raise RuntimeError(
                f"Stale data store '{n}' in category '{c}' encountered")
        elif super().__getattribute__("_orphaned"):
            n = super().__getattribute__("_name")
            c = super().__getattribute__("_category_name")
            raise RuntimeError(
                f"Data store '{n}' appears orphaned from stale category '{c}'")
        else:
            return super().__getattribute__(__name)

    def _mark_stale_(self):
        super().__setattr__("_stale", True)

    def _mark_orphaned_(self):
        super().__setattr__("_orphaned", True)

    @property
    def name(self):
        return self._name

    @property
    def category(self):
        return frontend.frontend().data_stores[self._category_name]

    @abstractclassmethod
    def get(self, key):
        pass

    @abstractclassmethod
    def store(self, key, value):
        pass

    @abstractclassmethod
    def remove(self, key, default=...):
        pass

    def get_data_store_class(self, **opts):
        return f"{self.__class__.__module__}.{self.__class__.__qualname__}"

    # The below methods will ensure the "dict-like" test fixtures will pass
    # Depending on the underlying implementation though, these may be very slow,
    # (e.g., if the data store is just a dictionary)

    @abstractclassmethod
    def items(self):
        pass

    def keys(self):
        return [k for (k, v) in self.items()]

    def values(self):
        return [v for (k, v) in self.items()]

    def __contains__(self, item):
        return item in self.keys()

    def __setitem__(self, key, value):
        return self.store(key, value)

    def __getitem__(self, key):
        if self.__contains__(key):
            return self.get(key)
        else:
            raise KeyError(key)

    def __len__(self):
        return len(self.keys())

    def __iter__(self):
        return self.keys().__iter__()

    def not_supported(self, caller=None):
        caller = caller or inspect.stack()[1][3]
        raise self.OperationNotSupported(self, caller)

    def __lookup_origen_feature__(self, f):
        features = DataStoreAPI.__origen_supported_features__.get(
            (self.__module__, self.__class__.__qualname__), None)
        if features is not None and f in features:
            return features[f]
        else:
            for b in self.__class__.__bases__:
                features = DataStoreAPI.__origen_supported_features__.get(
                    (b.__module__, b.__qualname__), None)
                if features is not None and f in features:
                    return features[f]
        return None
