import pytest, copy
from origen_metal._origen_metal import __test__
from origen_metal.utils import revision_control
from origen_metal.framework import Outcome
from origen_metal._origen_metal import frontend
from origen_metal.frontend import DataStoreAPI
from tests.shared.python_like_apis import Fixture_DictLikeAPI
from tests.shared import fresh_frontend

def init_frontend():
    frontend.reset()
    assert fe() is None
    assert frontend.initialize() is True
    assert isinstance(fe(), frontend.PyFrontend)


def fe():
    return frontend.frontend()


@pytest.fixture
def frontend_init():
    init_frontend()


def test_frontend_is_accessible():
    init_frontend()


def test_multiple_frontend_initializes_are_benign(frontend_init):
    assert frontend.initialize() is False


# TODO clean up a bunch of the hardcoding


class Common:
    class MinimumDataStore(DataStoreAPI):
        def __init__(self, *args, **kwargs):
            super().__init__(*args, **kwargs)
            self.test_param = "abc!"
            self.args = args
            self.kwargs = kwargs

        def update_test_param(self, new_param):
            self.test_param = new_param

        def get(self, key):
            return self.not_supported()

        def store(self, key):
            return self.not_supported()

        def remove(self, key, default=...):
            return self.not_supported()

        def items(self):
            return self.not_supported()

        @property
        def arg1(self):
            return self.args[0] if len(self.args) > 0 else None

        @property
        def arg2(self):
            return self.args[1] if len(self.args) > 1 else None

        @property
        def kwarg1(self):
            return self.kwargs.get("kwarg1", None)

        @property
        def kwarg2(self):
            return self.kwargs.get("kwarg2", None)

    class DummyGetStoreDataStore(DataStoreAPI):
        def __init__(self, *args, **kwargs):
            super().__init__(*args, **kwargs)
            self.data = {}

        def get(self, key):
            return self.data.get(key, None)

        def store(self, key, val):
            overwrote = key in self.data
            self.data[key] = val
            return overwrote

        def remove(self, key, default=...):
            if default is ...:
                if key in self.data:
                    return self.data.pop(key)
                else:
                    raise RuntimeError(
                        f"Cannot remove non-existent item '{key}' from data store '{self.name}' (category '{self._category_name}')"
                    )
            else:
                return self.data.pop(key, default)

        def get_data_store_class(self, *, arg=None):
            if arg == 1:
                return "CustomValue"
            else:
                return super().get_data_store_class()

        def say_hi(self):
            return "hi!"

        def say_hello(self):
            return Outcome(positional_results=["hello!"],
                           succeeded=True,
                           metadata={"working?": True})

        def echo(self, *to_echo, **kwargs):
            kwargs["__update__"] = True
            assert isinstance(to_echo, tuple)
            return to_echo

        def __contains__(self, key):
            return key in self.data

        def __len__(self):
            return len(self.data)

        def items(self):
            return [(k, v) for k, v in self.data.items()]

        def keys(self):
            return list(self.data.keys())

        def values(self):
            return list(self.data.values())

        def __iter__(self):
            return self.keys().__iter__()

    @property
    def ds(self):
        return fe().data_stores

    @property
    def test_cat_name(self):
        return "test_cat"

    @property
    def test_min_ds_name(self):
        return "min_ds"

    @property
    def test_ds_name(self):
        return "test_ds"

    @property
    def test_cat(self):
        return self.ds[self.test_cat_name]

    @property
    def test_min_ds(self):
        return self.ds[self.test_cat_name][self.test_min_ds_name]

    @property
    def test_ds(self):
        return self.ds[self.test_cat_name][self.test_ds_name]

    @property
    def data_stores_class(self):
        return frontend.PyDataStores

    @property
    def cat_class(self):
        return frontend.PyDataStoreCategory


class TestFrontendDataStore(Common):
    def test_data_stores_start_empty(self):
        assert isinstance(fe().data_stores, self.data_stores_class)
        assert len(self.ds.categories) == 0
        assert self.ds.categories == {}

    def test_categories_can_be_added(self):
        assert self.test_cat_name not in self.ds
        added = self.ds.add_category(self.test_cat_name)
        assert isinstance(added, self.cat_class)
        assert added.name == self.test_cat_name
        assert self.test_cat_name in self.ds
        assert self.ds.categories == {self.test_cat_name: self.test_cat}

    def test_data_stores_can_be_added(self):
        test_cat = self.test_cat
        assert self.test_cat_name in self.ds
        assert self.test_min_ds_name not in test_cat
        ds = test_cat.add(self.test_min_ds_name, self.MinimumDataStore)
        assert isinstance(ds, self.MinimumDataStore)
        assert ds.name == self.test_min_ds_name
        assert ds.category == self.test_cat

    def test_error_on_adding_duplicate_categories(self):
        n = self.test_cat_name
        with pytest.raises(
                RuntimeError,
                match=f"Data store category '{n}' is already present"):
            self.ds.add_category(n)

    def test_removing_categories(self):
        t = "test_cat_removal"
        assert t not in self.ds.categories
        cat = self.ds.add_category(t)
        assert t in self.ds.categories
        assert cat.name == t
        assert self.ds.remove_category(t) is None
        assert t not in self.ds.categories
        with pytest.raises(RuntimeError,
                           match=f"Stale category '{t}' encountered"):
            cat.name

        # Adding another with the same name should still show the previous as stale
        cat2 = self.ds.add_category(t)
        assert t in self.ds.categories
        assert cat2 != cat
        with pytest.raises(RuntimeError,
                           match=f"Stale category '{t}' encountered"):
            cat.name
        assert self.ds.remove_category(t) is None

    def test_error_removing_nonexistent_categories(self):
        t = "test_cat_removal"
        with pytest.raises(
                RuntimeError,
                match=f"Cannot remove non-existent data store category '{t}'"):
            self.ds.remove_category(t)

    def test_data_stores_can_be_retrieved(self):
        assert self.test_min_ds_name in self.test_cat
        dummy_store = self.test_min_ds
        assert isinstance(dummy_store, self.MinimumDataStore)
        assert dummy_store.test_param == "abc!"

        dummy_store.update_test_param("123!")
        assert dummy_store.test_param == "123!"
        # Try with a new fetch
        self.test_min_ds.test_param == "123!"

    def test_data_stores_can_be_added_with_opts(self):
        test_cat = self.test_cat
        arg_ds_name = "test_ds_args"
        kwarg_ds_name = "test_ds_kwargs"
        both_ds_name = "test_ds_args_and_kwargs"

        test_args = ["arg1", 12345]
        test_kwargs = {"kwarg1": "kw1", "kwarg2": 2}

        ds = test_cat.add(arg_ds_name, self.MinimumDataStore, test_args)
        assert ds.name == arg_ds_name
        assert ds.arg1 == test_args[0]
        assert ds.arg2 == test_args[1]
        assert ds.kwarg1 == None
        assert ds.kwarg2 == None

        ds = test_cat.add(kwarg_ds_name, self.MinimumDataStore, None,
                          test_kwargs)
        assert ds.name == kwarg_ds_name
        assert ds.arg1 == None
        assert ds.arg2 == None
        assert ds.kwarg1 == test_kwargs["kwarg1"]
        assert ds.kwarg2 == test_kwargs["kwarg2"]

        ds = test_cat.add(both_ds_name, self.MinimumDataStore, test_args,
                          test_kwargs)
        assert ds.name == both_ds_name
        assert ds.arg1 == test_args[0]
        assert ds.arg2 == test_args[1]
        assert ds.kwarg1 == test_kwargs["kwarg1"]
        assert ds.kwarg2 == test_kwargs["kwarg2"]

    def test_adding_data_stores_passing_name_and_category(self):
        n = "test_nc1"
        assert n not in self.test_cat
        added = self.test_cat.add(n,
                                  self.MinimumDataStore,
                                  provide_name=True,
                                  provide_category=True)
        assert added.args == ()
        assert added.kwargs == {"name": n, "category": self.test_cat_name}

        n = "test_nc2"
        args = ["abc", 123]
        kwargs = {"kwarg1": "kw1", "kwarg2": 2}
        added = self.test_cat.add(n,
                                  self.MinimumDataStore,
                                  args,
                                  provide_name=0,
                                  provide_category=3)
        assert args == ["abc", 123]
        assert added.args == (n, "abc", 123, self.test_cat_name)
        assert kwargs == {"kwarg1": "kw1", "kwarg2": 2}
        assert added.kwargs == {}

        n = "test_nc3"
        added = self.test_cat.add(n,
                                  self.MinimumDataStore,
                                  args,
                                  kwargs,
                                  provide_name=0,
                                  provide_category=True)
        assert args == ["abc", 123]
        assert added.args == (n, "abc", 123)
        assert kwargs == {"kwarg1": "kw1", "kwarg2": 2}
        assert added.kwargs == {
            "kwarg1": "kw1",
            "kwarg2": 2,
            "category": self.test_cat_name
        }

        n = "fail_conditions"
        assert n not in self.test_cat
        with pytest.raises(
                RuntimeError,
                match=
                "'provide_name' insert index 1 exceeds argument list size 0"):
            self.test_cat.add(n, self.MinimumDataStore, provide_name=1)
        assert n not in self.test_cat

        with pytest.raises(
                RuntimeError,
                match=
                "'provide_category' insert index 4 exceeds argument list size 2"
        ):
            self.test_cat.add(n,
                              self.MinimumDataStore,
                              args,
                              provide_category=4)
        assert args == ["abc", 123]
        assert n not in self.test_cat

        kwargs = {"kwarg1": "kw1", "kwarg2": 2, "name": "hi"}
        with pytest.raises(
                RuntimeError,
                match="'name' key is already present in keyword arguments"):
            self.test_cat.add(n,
                              self.MinimumDataStore,
                              None,
                              kwargs,
                              provide_name=True)
        assert kwargs == {"kwarg1": "kw1", "kwarg2": 2, "name": "hi"}
        assert n not in self.test_cat

        kwargs = {"kwarg1": "kw1", "kwarg2": 2, "category": "hi"}
        with pytest.raises(
                RuntimeError,
                match="'category' key is already present in keyword arguments"
        ):
            self.test_cat.add(n,
                              self.MinimumDataStore,
                              None,
                              kwargs,
                              provide_category=True)
        assert kwargs == {"kwarg1": "kw1", "kwarg2": 2, "category": "hi"}
        assert n not in self.test_cat

    def test_data_store_get_and_store_operations(self):
        assert self.test_ds_name not in self.test_cat
        added = self.test_cat.add(self.test_ds_name,
                                  self.DummyGetStoreDataStore)
        assert isinstance(added, self.DummyGetStoreDataStore)
        assert self.test_ds_name in self.test_cat

        ds = self.test_ds
        assert "t1" not in ds
        assert ds.get("t1") == None
        assert __test__.backend_store_contains(self.test_cat_name,
                                               self.test_ds_name,
                                               "t1") == False
        assert __test__.backend_get_stored(self.test_cat_name,
                                           self.test_ds_name, "t1") == None

        s = "hi data store!"
        ds["t1"] = s
        assert "t1" in ds
        assert ds.get("t1") == s
        assert __test__.backend_store_contains(self.test_cat_name,
                                               self.test_ds_name, "t1") == True
        assert __test__.backend_get_stored(self.test_cat_name,
                                           self.test_ds_name, "t1") == s

        n = 321
        assert __test__.backend_store_item(self.test_cat_name,
                                           self.test_ds_name, "t1", n) == True
        assert "t1" in ds
        assert ds.get("t1") == n
        assert __test__.backend_store_contains(self.test_cat_name,
                                               self.test_ds_name, "t1") == True
        assert __test__.backend_get_stored(self.test_cat_name,
                                           self.test_ds_name, "t1") == n

        assert ds.store("t2", "'returns self' test") == False
        assert ds.store("t2", "'returns self' override test") == True
        assert ds.get("t2") == "'returns self' override test"

    def test_data_store_remove_operations(self):
        n = "test_removal"
        ds = self.test_ds
        assert n not in ds
        ds.store(n, 0xabc)
        assert n in ds
        assert ds.remove(n) == 0xabc
        assert n not in ds

    def test_error_removing_non_existent_items(self):
        n = "test_removal"
        ds = self.test_ds
        assert n not in ds
        with pytest.raises(
                RuntimeError,
                match=
                f"Cannot remove non-existent item '{n}' from data store '{self.test_ds_name}' \\(category '{self.test_cat_name}'\\)"
        ):
            ds.remove(n)

    def test_removing_non_existent_item_with_default(self):
        n = "test_removal"
        ds = self.test_ds
        assert n not in ds
        assert ds.remove(n, "abc") == "abc"

    def test_error_on_adding_duplicate_data_store_categories(self):
        n = self.test_cat_name
        assert n in self.ds
        with pytest.raises(
                RuntimeError,
                match=f"Data store category '{n}' is already present"):
            self.ds.add_category(n)

    def test_error_on_adding_duplicate_data_stores(self):
        n = self.test_ds_name
        assert n in self.test_cat
        with pytest.raises(
                RuntimeError,
                match=
                f"Data store '{n}' is already present in category '{self.test_cat_name}'"
        ):
            self.test_cat.add(n, self.MinimumDataStore)

    def test_removing_data_stores(self):
        n = "test_ds_removal"
        assert n not in self.test_cat

        ds = self.test_cat.add(n, self.MinimumDataStore)
        assert n in self.test_cat
        assert ds.name == n

        self.test_cat.remove(n)
        assert n not in self.test_cat
        with pytest.raises(
                RuntimeError,
                match=
                f"Stale data store '{n}' in category '{self.test_cat_name}' encountered"
        ):
            ds.name

    def test_error_removing_nonexistant_data_stores(self):
        n = "test_ds_removal"
        assert n not in self.test_cat
        with pytest.raises(
                RuntimeError,
                match=
                f"Cannot remove data store '{n} as category '{self.test_cat_name}' does not contain this data store"
        ):
            self.test_cat.remove(n)

    def test_stale_ds_after_category_removal(self):
        n_cat = "test_cat_removal"
        n_ds = "test_ds_removal"
        assert n_cat not in self.ds.categories

        cat = self.ds.add_category(n_cat)
        ds = cat.add(n_ds, self.MinimumDataStore)
        assert cat.name == n_cat
        assert ds.name == n_ds

        self.ds.remove_category(n_cat)
        assert n_cat not in self.ds.categories

        with pytest.raises(RuntimeError,
                           match=f"Stale category '{n_cat}' encountered"):
            cat.name

        with pytest.raises(
                RuntimeError,
                match=
                f"Data store '{n_ds}' appears orphaned from stale category '{n_cat}'"
        ):
            ds.name

    def test_adding_categories_from_backend(self):
        t = __test__.backend_test_cat_name
        assert t not in self.ds
        assert __test__.backend_contains_cat(t) is False
        assert __test__.backend_add_cat(t) is None

        assert t in self.ds
        assert __test__.backend_contains_cat(t) is True

    def test_removing_categories_from_backend(self):
        n = "BE_test_cat_removal"
        assert n not in self.ds
        self.ds.add_category(n)
        assert n in self.ds

        __test__.backend_remove_cat(n)
        assert n not in self.ds

    def test_error_removing_nonexistant_data_stores_from_backend(self):
        n = "test_cat_removal"
        assert n not in self.ds
        with pytest.raises(
                RuntimeError,
                match=f"Cannot remove non-existent data store category '{n}'"):
            __test__.backend_remove_cat(n)

    def test_error_on_adding_existing_categories_from_backend(self):
        t = __test__.backend_test_cat_name
        assert t in self.ds
        assert __test__.backend_contains_cat(t) is True
        with pytest.raises(
                RuntimeError,
                match=f"Data store category '{t}' is already present"):
            __test__.backend_add_cat(t)

    def test_adding_data_stores_from_backend(self):
        t_cat = __test__.backend_test_cat_name
        t_ds = __test__.backend_test_store_name
        assert t_ds not in self.ds[t_cat]
        assert not __test__.backend_cat_contains_store(t_cat, t_ds)
        assert __test__.backend_add_store(t_cat, t_ds, {
            "class":
            f"tests.test_frontend.TestFrontendDataStore.MinimumDataStore"
        }) is None

        assert t_ds in self.ds[t_cat]
        ds = self.ds[t_cat][t_ds]
        assert isinstance(ds, self.MinimumDataStore)
        assert ds.name == t_ds
        assert __test__.backend_cat_contains_store(t_cat, t_ds)

    def test_adding_data_stores_from_backend_with_options(self):
        t_cat = __test__.backend_test_cat_name
        t_ds = __test__.backend_test_store_with_opts_name

        test_args = ["be_arg1", 123]
        test_kwargs = {"kwarg1": "be_kw1", "kwarg2": 2}

        assert t_ds not in self.ds[t_cat]
        config = {
            "class":
            f"tests.test_frontend.TestFrontendDataStore.MinimumDataStore",
            "list_args": test_args,
            **test_kwargs
        }
        assert __test__.backend_add_store(t_cat, t_ds, config) is None

        ds = self.ds[t_cat][t_ds]
        assert isinstance(ds, self.MinimumDataStore)
        assert ds.name == t_ds
        assert __test__.backend_cat_contains_store(t_cat, t_ds)
        assert ds.name == t_ds
        assert ds.arg1 == test_args[0]
        assert ds.arg2 == test_args[1]
        assert ds.kwarg1 == test_kwargs["kwarg1"]
        assert ds.kwarg2 == test_kwargs["kwarg2"]

    def test_error_adding_duplicate_data_stores_from_backend(self):
        n = self.test_ds_name
        assert n in self.test_cat
        with pytest.raises(
                RuntimeError,
                match=
                f"Data store '{n}' is already present in category '{self.test_cat_name}'"
        ):
            self.test_cat.add(n, self.MinimumDataStore)

    def test_removing_data_stores_from_backend(self):
        n = "BE_test_ds_removal"
        cat = self.test_cat
        assert n not in cat

        cat.add(n, self.MinimumDataStore)
        assert n in cat

        __test__.backend_remove_store(self.test_cat_name, n)
        assert n not in cat

    def test_error_removing_nonexistant_categories_from_backend(self):
        n = "BE_test_ds_removal"
        cat = self.test_cat
        assert n not in cat
        with pytest.raises(
                RuntimeError,
                match=
                f"Cannot remove data store '{n} as category '{self.test_cat_name}' does not contain this data store"
        ):
            __test__.backend_remove_store(self.test_cat_name, n)

    def test_basic_data_store_operations_from_backend(self):
        assert __test__.backend_get_name(
            self.test_cat_name, self.test_ds_name) == self.test_ds_name
        assert __test__.backend_get_category(
            self.test_cat_name, self.test_ds_name) == self.test_cat_name
        assert __test__.backend_get_class(
            self.test_cat_name, self.test_ds_name
        ) == "tests.test_frontend.Common.DummyGetStoreDataStore"
        assert __test__.backend_get_class(self.test_cat_name,
                                          self.test_ds_name,
                                          arg=1) == "CustomValue"

    def test_data_store_remove_operations_from_backend(self):
        ds = self.test_ds

        n = "BE_test_item_removal"
        t = "BE_removal_test"
        assert n not in ds
        ds.store(n, t)
        assert n in ds

        __test__.backend_remove_item(self.test_cat_name, self.test_ds_name,
                                     n) == [True, t]
        assert n not in ds

        __test__.backend_remove_item(self.test_cat_name, self.test_ds_name,
                                     n) == [False, None]

    def test_data_store_call_from_backend_no_args(self):
        outcome = __test__.backend_call(self.test_cat_name, self.test_ds_name,
                                        "say_hi")
        assert outcome.succeeded == True
        assert outcome.inferred == True
        assert outcome.positional_results == ("hi!", )
        assert outcome.keyword_results == None
        assert outcome.metadata == None

    def test_data_store_call_from_backend_with_args(self):
        args = ["hi", 12345]
        kwargs = {"first": 1, "second": 2, "third": 3}
        _args = copy.deepcopy(args)
        _kwargs = copy.deepcopy(kwargs)
        outcome = __test__.backend_call(self.test_cat_name, self.test_ds_name,
                                        "echo", args, kwargs)
        assert outcome.succeeded == True
        assert outcome.inferred == True
        assert outcome.positional_results == (_args, )
        assert args == _args
        assert outcome.keyword_results == None
        assert kwargs == _kwargs
        assert outcome.metadata == None

    def test_data_store_call_from_backend_non_inferred_outcome(self):
        outcome = __test__.backend_call(self.test_cat_name, self.test_ds_name,
                                        "say_hello")
        assert outcome.succeeded == True
        assert outcome.inferred == False
        assert outcome.positional_results == ("hello!", )
        assert outcome.keyword_results == None
        assert outcome.metadata == {"working?": True}

class TestDataStoresDictLike(Fixture_DictLikeAPI, Common):
    def parameterize(self):
        return {
            "keys": ['test_cat', 'BE_cat'],
            "klass": self.cat_class,
            "not_in_dut": "Blah"
        }

    def boot_dict_under_test(self):
        return self.ds


class TestDataStoreCategoryDictLike(Fixture_DictLikeAPI, Common):
    def parameterize(self):
        return {
            "keys": [
                "min_ds", "test_ds_args", "test_ds_kwargs",
                "test_ds_args_and_kwargs", "test_nc1", "test_nc2", "test_nc3",
                "test_ds"
            ],
            "klass": [
                self.MinimumDataStore, self.MinimumDataStore,
                self.MinimumDataStore, self.MinimumDataStore,
                self.MinimumDataStore, self.MinimumDataStore,
                self.MinimumDataStore, self.DummyGetStoreDataStore
            ],
            "not_in_dut":
            "Blah"
        }

    def boot_dict_under_test(self):
        return self.test_cat


class TestDataStoreAPIDictLike(Fixture_DictLikeAPI, Common):
    ''' The data store API should contain all methods for dict-lie behavior'''
    def parameterize(self):
        return {
            "keys": ["i1", "i2", "i3", "i4"],
            "klass": int,
            "not_in_dut": "Blah"
        }

    def init_dict_under_test(self):
        ds = self.test_cat.add("dict_like_api_test",
                               self.DummyGetStoreDataStore)
        ds.store("i1", 1)
        ds.store("i2", 2)
        ds.store("i3", 3)
        ds.store("i4", 4)

    def boot_dict_under_test(self):
        return self.test_cat['dict_like_api_test']


class TestRevisionControlFrontend:
    class DummyRC(revision_control.RevisionControlAPI):
        def init(self):
            return Outcome(succeeded=True, message="From Dummy RC")

        def is_initialized(self):
            return True

        def checkout(self):
            raise NotImplemented("checkout not available for DummyRC")

        def populate(self):
            raise NotImplemented("populate not available for DummyRC")

        def revert(self):
            raise NotImplemented("revert not available for DummyRC")

        def status(self):
            raise NotImplemented("status not available for DummyRC")

        def system(self):
            self.__class__.name

        def tag(self):
            raise NotImplemented("tag not available for DummyRC")

    @pytest.fixture
    def dummy_rc(self):
        fe().rc = TestRevisionControlFrontend.DummyRC()

    def test_frontend_rc_driver(frontend_init):
        assert fe().rc is None
        assert fe().revision_control is None

    def test_frontend_rc_driver_can_be_set(frontend_init, dummy_rc):
        assert isinstance(fe().rc, TestRevisionControlFrontend.DummyRC)
        assert isinstance(fe().revision_control,
                          TestRevisionControlFrontend.DummyRC)

    def test_frontend_rc_can_be_called_by_the_backend(frontend_init, dummy_rc):
        outcome = __test__.rc_init_from_metal()
        assert outcome.succeeded is True
        assert outcome.message == "From Dummy RC"

class TestDataStoreCategories(Common):
    def load_method(self, cat):
        cat.add("ds_method1", self.MinimumDataStore)
        cat.add("ds_method2", self.MinimumDataStore)
        cat.add("ds_method3", self.MinimumDataStore)

    @staticmethod
    def load_func_from_str(cat):
        cat.add("ds_func_str1", TestDataStoreCategories.MinimumDataStore)

    @staticmethod
    def load_func_from_str_updated(cat):
        cat.add("ds_func_str_updated", TestDataStoreCategories.MinimumDataStore)

    def load_method_from_str(self, cat):
        cat.add("ds_method_str1", self.MinimumDataStore)

    @property
    def load_func_str(self):
        return f"{TestDataStoreCategories.load_func_from_str.__module__}.{TestDataStoreCategories.load_func_from_str.__qualname__}"

    @property
    def load_method_str(self):
        return f"{TestDataStoreCategories.load_func_from_str.__module__}.{TestDataStoreCategories.load_method_from_str.__qualname__}"

    def test_category_without_load_method(self, fresh_frontend):
        assert len(self.ds.categories) == 0
        assert self.ds.unloaded_categories == []

        added = self.ds.add_category(self.test_cat_name)
        assert len(self.ds.categories) == 1
        assert self.ds.unloaded_categories == [self.test_cat_name]

        # TODO these should be autoloaded?
        # Or an autoload vs. a lazy-load?
        assert added is not None
        assert added.loaded == False
        assert added.unloaded == True
        assert added.autoload == True
        assert added.load_function is None

        cat = self.ds.get(self.test_cat_name)
        assert cat.loaded == True
        assert cat.unloaded == False
        assert list(cat.keys()) == []
        assert list(added.keys()) == []
        assert added.loaded == True
        assert added.unloaded == False
        assert added.autoload == True
        assert added.load_function is None
        assert self.ds.unloaded_categories == []

    def test_loading_with_function(self, fresh_frontend):
        def load_function(cat):
            cat.add("ds_func1", self.MinimumDataStore)
            cat.add("ds_func2", self.MinimumDataStore)

        added = self.ds.add_category(self.test_cat_name, load_function=load_function)
        cat = self.ds.get(self.test_cat_name)
        assert cat.loaded == True
        assert cat.unloaded == False
        assert added.loaded == True
        assert added.unloaded == False
        assert added.autoload == True
        assert added.load_function == load_function
        assert list(cat.keys()) == ["ds_func1", "ds_func2"]

    def test_loading_with_method(self, fresh_frontend):
        added = self.ds.add_category(self.test_cat_name, load_function=self.load_method)
        cat = self.ds.get(self.test_cat_name)
        assert cat.loaded == True
        assert cat.autoload == True
        assert cat.load_function == self.load_method
        assert list(cat.keys()) == ["ds_method1", "ds_method2", "ds_method3"]

    def test_loading_with_function_from_string(self, fresh_frontend):
        added = self.ds.add_category(self.test_cat_name, load_function=self.load_func_str)
        cat = self.ds.get(self.test_cat_name)
        assert cat.loaded == True
        assert cat.autoload == True
        assert cat.load_function == self.load_func_str
        assert added.loaded == True
        assert added.unloaded == False
        assert added.autoload == True
        assert added.load_function == self.load_func_str
        assert list(cat.keys()) == ["ds_func_str1"]

    def test_error_loading_with_method_from_string(self, fresh_frontend):
        ''' This is a known shortcoming - looking up an attribute by name will not have an associated instance,
            and therefore method calls are not appropriate.

            Since the instance will need to be known/given, this can be worked around by giving the method directly or otherwise
            caching/storing the instance in some global location, and giving the function which will access this and re-call as
            a method.

            Since the purpose of giving function names is to facilitate booting, this limitation should be acceptable.

            FEATURE possibly see if some generic solution of "cache an object then call this method on it" can be supported
        '''
        added = self.ds.add_category(self.test_cat_name, load_function=self.load_method_str)
        with pytest.raises(TypeError, match=fr"load_method_from_str\(\) missing 1 required positional argument: 'cat'"):
            self.ds.get(self.test_cat_name)
        assert added.loaded == False
        assert added.unloaded == True
        assert added.autoload == True
        assert added.load_function == self.load_method_str
        assert list(added.keys()) == []

    def test_loading_with_string_is_lazily_evaluated(self, fresh_frontend):
        self.ds.add_category(self.test_cat_name, load_function=self.load_func_str)
        self.ds.add_category("second", load_function=self.load_func_str)
        old_name = self.load_func_str
        old_m = self.load_func_from_str
        setattr(self.__class__, "load_func_from_str", self.load_func_from_str_updated)

        cat = self.ds.get(self.test_cat_name)
        assert cat.load_function == old_name
        assert list(cat.keys()) == ["ds_func_str_updated"]
        setattr(self.__class__, "load_func_from_str", old_m)

        cat = self.ds.get("second")
        assert cat.load_function == self.load_func_str
        assert list(cat.keys()) == ["ds_func_str1"]

    def test_loading_with_none(self, fresh_frontend):
        added = self.ds.add_category(self.test_cat_name, load_function=None)
        cat = self.ds.get(self.test_cat_name)
        assert cat.loaded == True
        assert cat.autoload == True
        assert cat.load_function is None
        assert added.loaded == True
        assert added.unloaded == False
        assert added.autoload == True
        assert added.load_function is None
        assert list(cat.keys()) == []

    def test_invalid_load_type(self, fresh_frontend):
        with pytest.raises(RuntimeError, match=f"Load function for category '{self.test_cat_name}' should either be a fully-qualified function name or a callable object"):
            self.ds.add_category(self.test_cat_name, load_function=0)

    def test_invalid_load_func_str(self, fresh_frontend):
        self.ds.add_category("invalid1", load_function="unknown_function")
        with pytest.raises(AttributeError, match="module 'builtins' has no attribute 'unknown_function'"):
            self.ds.get("invalid1")

        self.ds.add_category("invalid2", load_function="unknown.function")
        with pytest.raises(ModuleNotFoundError, match="No module named 'unknown'"):
            self.ds.get("invalid2")

    def test_manual_loading(self, fresh_frontend):
        def manual_load_func(cat):
            cat.add("ds_manual1", self.MinimumDataStore)
            cat.add("ds_manual2", self.MinimumDataStore)
            return None

        added = self.ds.add_category(self.test_cat_name, load_function=manual_load_func, autoload=False)
        cat = self.ds.get(self.test_cat_name)
        assert cat.loaded == False
        assert cat.autoload == False
        assert added.loaded == False
        assert added.autoload == False
        assert list(added.keys()) == []
        assert self.ds.unloaded_categories == [self.test_cat_name]

        r = cat.load()
        assert cat.loaded == True
        assert cat.autoload == False
        assert added.loaded == True
        assert added.autoload == False
        assert list(added.keys()) == ["ds_manual1", "ds_manual2"]
        assert self.ds.unloaded_categories == []

        assert isinstance(r, Outcome)
        assert r.succeeded == True

    @pytest.mark.xfail
    def test_loading_function_returns_bool(self):
        raise NotImplementedError

    @pytest.mark.xfail
    def test_loading_function_returns_error(self):
        raise NotImplementedError

    @pytest.mark.xfail
    def test_loading_function_returns_string(self):
        raise NotImplementedError

    @pytest.mark.xfail
    def test_loading_function_bad_return_type(self):
        raise NotImplementedError

    @pytest.mark.xfail
    def test_loading_categories_accessing_categories(self):
        raise NotImplementedError

    @pytest.mark.xfail
    def test_loading_categories_adding_categories(self):
        raise NotImplementedError

    @pytest.mark.xfail
    def test_error_loading_categories(self):
        raise NotImplementedError
        # Failed outcome
        # Error outcome
        # Exception during execution
