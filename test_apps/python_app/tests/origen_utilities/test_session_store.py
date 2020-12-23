import pytest, shutil, pathlib, marshal, inspect, pickle, origen, _origen
from tests.shared import tmp_dir

class Dummy:
    def __init__(self, data):
        self.data = data

class Base:
    @property
    def app_test_session_root(self):
        return pathlib.Path(tmp_dir().joinpath(".session"))

    @property
    def user_test_session_root(self):
        return pathlib.Path(tmp_dir().joinpath(".user.session"))

    @property
    def _plugin_app_session(self):
        return _origen.utility.session_store.app_session(self.pl_name)
    
    @property
    def _plugin_user_session(self):
        return _origen.utility.session_store.user_session(self.pl_name)
    
    @property
    def plugin_app_session_path(self):
        return self.app_test_session_root.joinpath(self.pl_name)

    @property
    def plugin_user_session_path(self):
        return self.user_test_session_root.joinpath(self.pl_name)

    @property
    def pl(self):
        return origen.plugin(self.pl_name)
    
    @property
    def pl_name(self):
        return "python_plugin"

    @property
    def session_store(self):
        return origen.session_store

    def blank_app_session_for(self):
        name = inspect.stack()[1][3]
        s = origen.session_store.app_session(name)
        s.remove_file()
        return s

    def blank_user_session_for(self):
        name = inspect.stack()[1][3]
        s = origen.session_store.user_session(name)
        s.remove_file()
        return s

    @pytest.fixture
    def clear_test_sessions(self):
        if self.app_test_session_root.exists():
            shutil.rmtree(self.app_test_session_root)
        if self.user_test_session_root.exists():
            shutil.rmtree(self.user_test_session_root)
        self.session_store.clear_cache()
        assert self.app_test_session_root.exists() is False
        assert self.user_test_session_root.exists() is False
    
    def update_roots(self):
        self.session_store.clear_cache()
        origen.session_store.set_app_root(self.app_test_session_root)
        origen.session_store.set_user_root(self.user_test_session_root)

class TestSessionStoreDefaults(Base):
    def test_blank_sessions(self, clear_test_sessions):
        ''' Should return a Session class but not actually create any files yet'''
        assert origen.session_store == _origen.utility.session_store
        assert isinstance(origen.session_store.app_session(), _origen.utility.session_store.SessionStore)
        assert isinstance(origen.session_store.user_session(), _origen.utility.session_store.SessionStore)
        assert origen.session_store.app_session().is_app_session
        assert not origen.session_store.app_session().is_user_session
        assert not origen.session_store.user_session().is_app_session
        assert origen.session_store.user_session().is_user_session
        assert not self.app_test_session_root.exists()
        assert not self.user_test_session_root.exists()

    def test_updating_session_roots(self):
        ''' The user path will move around and is more dependent on the 
            `User` class than the session.
            Just make sure its loosly valid. We'll be changing it anyway so
            just ensure the change is observed.
        '''
        assert origen.session_store.app_root() == origen.app.root.joinpath('.session')
        assert origen.session_store.user_root() == origen.current_user().home_dir.joinpath('.o2/.session')
        assert origen.session_store.user_root() != self.user_test_session_root
        self.update_roots()
        assert origen.session_store.app_root() == self.app_test_session_root
        assert origen.session_store.user_root() == self.user_test_session_root

    def test_app_session_aliases(self):
        assert origen.session_store.app_session() == _origen.utility.session_store.app_session()
        assert origen.session_store.app_session("example") == _origen.utility.session_store.app_session()
        assert origen.app.session == _origen.utility.session_store.app_session()
        assert origen.session_store.app_session(origen.app) == _origen.utility.session_store.app_session()

    def test_user_session_aliases(self):
        assert origen.session_store.user_session() == _origen.utility.session_store.user_session()
        assert origen.current_user().session == _origen.utility.session_store.user_session()
        assert origen.session_store.user_session(origen.app) == _origen.utility.session_store.user_session(origen.app)

    def test_user_session_for_app(self):
        assert origen.app.user_session == _origen.utility.session_store.user_session(origen.app)

    def test_plugin_session_aliases(self):
        assert self._plugin_app_session.path == self.plugin_app_session_path
        assert self._plugin_user_session.path == self.plugin_user_session_path

        assert origen.session_store.app_session(self.pl_name) == self._plugin_app_session
        assert origen.plugin(self.pl_name).session == self._plugin_app_session

        assert origen.session_store.user_session(self.pl_name) == self._plugin_user_session
        assert origen.plugin(self.pl_name).user_session == self._plugin_user_session

class TestSessionStore(Base):
    
    @pytest.fixture(autouse=True)
    def update_roots(self):
        Base.update_roots(self)

    def test_adding_app_sessions(self):
        s = origen.session_store.app_session("hi")
        assert isinstance(s, _origen.utility.session_store.SessionStore)
        assert s.path == self.app_test_session_root.joinpath("hi")
        assert s == origen.session_store.app_session("hi")

    def test_adding_user_sessions(self):
        s = origen.session_store.user_session("hi")
        assert isinstance(s, _origen.utility.session_store.SessionStore)
        assert s.path == self.user_test_session_root.joinpath("hi")
        assert s == origen.session_store.user_session("hi")

    def test_roundtrip(self):
        s = self.blank_app_session_for()
        assert s.get("test") == None
        assert s.path.exists() is False
        s.store("test", "str")
        s.store("test2", "strA")
        assert s.path.exists() is True

        s.refresh()
        assert s.get("test") == "str"
        assert s.get("test2") == "strA"

        # Overide an item
        s.store("test", "str2")
        s.refresh()
        assert s.get("test") == "str2"

    def test_roundtrip_user_session(self):
        s = self.blank_user_session_for()
        assert s.get("test") == None
        assert s.path.exists() is False

        s.store("test", "user_str")
        s.store("test2", "user_strA")
        assert s.path.exists() is True

        s.refresh()
        assert s.get("test") == "user_str"
        assert s.get("test2") == "user_strA"

    def test_roundtrip_for_fancier_objects(self):
        s = self.blank_app_session_for()
        assert s.get("list test") == None
        s.store("list test", [1, 2, "three", 1.23, -5])
        s.refresh()
        assert s.get("list test") == [1, 2, "three", 1.23, -5]

    def test_roundtrip_for_custom_class(self):
        s = self.blank_app_session_for()
        assert s.get("dummy") == None
        s.store("dummy", Dummy(1))
        d = s.get("dummy")
        assert isinstance(d, Dummy)
        assert d.data == 1

    def test_clearing_session_values(self):
        s = self.blank_app_session_for()
        assert s.store("test_str", "str").refresh().get("test_str") == "str"
        assert s.store("test_str2", "str2").refresh().get("test_str2") == "str2"
        assert s.store("test_str3", "str3").refresh().get("test_str3") == "str3"

        # Set an item to None
        assert s.store("test_str", None).refresh().get("test_str") == None
        assert s.get("test_str2") == "str2"
        assert s.get("test_str3") == "str3"

        # Use delete method
        assert s.delete("test_str2") == "str2"
        s.refresh()
        assert s.get("test_str2") == None
        assert s.get("test_str3") == "str3"

    def test_data_is_returned_by_value(self):
        s = self.blank_app_session_for()
        assert s.get("test") is None
        d = {"i0": 0, "i1": 1}
        s.store("test", d).refresh()
        roundtrip_d = s.get("test")
        assert roundtrip_d == d
        roundtrip_d["i2"] = 2
        assert roundtrip_d != d
        roundtrip_d_2 = s.get("test")
        assert roundtrip_d_2 == d
        assert not roundtrip_d_2 == roundtrip_d

        s.store("test", roundtrip_d).refresh()
        assert roundtrip_d == s.get("test")

    def test_roundtrip_serialized(self):
        ''' The session will automatically serialize data by default. However,
            if the data is pre-serialized, can store it verbatim as a byte array.

            When this is retrieved, it will not be de-serialized by the session.
        '''
        s = self.blank_app_session_for()
        assert s.get("serialized") == None
        s.store_serialized("serialized", marshal.dumps("serialized?"))
        s.refresh()
        serialized = s.get("serialized")
        assert isinstance(serialized, bytes)
        assert marshal.loads(serialized) == "serialized?"

        # Pickle is used under the hood to serialize/de-serialize Python objects.
        # Test that we can get it back using just `get` confirming that:
        #   1. `get` and `get_serialized` are returning the same data, just different formats
        #   2. serialization didn't occur on the Rust side (though, already tested above).
        s.store_serialized("pickle_serialized", pickle.dumps("Pickle!"))
        s.refresh()
        assert pickle.loads(s.get("pickle_serialized")) == "Pickle!"

    @pytest.mark.skip
    def test_dict_like(self):
        ...

    def test_roundtrip_as_string(self):
        s = self.blank_app_session_for()
        assert s.store("test_str", "str").refresh().get("test_str") == "str"

    def test_roundtrip_as_int(self):
        s = self.blank_app_session_for()
        assert s.store("test_int", 1).refresh().get("test_int") == 1
        assert s.store("test_int_neg", -1).refresh().get("test_int_neg") == -1

    def test_roundtrip_as_bool(self):
        s = self.blank_app_session_for()
        assert s.store("test_true", True).refresh().get("test_true") == True
        assert s.store("test_false", False).refresh().get("test_false") == False

    def test_roundtrip_as_float(self):
        s = self.blank_app_session_for()
        assert s.store("test_float", 3.14).refresh().get("test_float") == 3.14
        assert s.store("test_float_neg", -3.14).refresh().get("test_float_neg") == -3.14
