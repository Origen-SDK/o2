import pytest, inspect, pickle, marshal, shutil, pathlib
import origen_metal as om
from tests.shared import tmp_dir
from tests.shared.python_like_apis import Fixture_DictLikeAPI

class Dummy:
    def __init__(self, data):
        self.data = data

class Common:
    @property
    def sessions(self):
        return om.sessions

    @property
    def root(self):
        return tmp_dir().joinpath(".session")

    @property
    def prepopulated_root(self):
        return pathlib.Path(__file__).parent.joinpath('prepopulated_sessions')

    @property
    def sessions_class(self):
        return om._origen_metal.framework.sessions.Sessions

    @property
    def group_class(self):
        return om._origen_metal.framework.sessions.SessionGroup

    @property
    def sg_class(self):
        return self.group_class

    @property
    def session_class(self):
        return om._origen_metal.framework.sessions.SessionStore

    @property
    def ss_class(self):
        return self.session_class

    def blank_session_for(self, name=None, remove=True):
        n = name or inspect.stack()[1][3]
        r = self.root
        if remove:
            if r.joinpath(n).exists():
                # 'missing_ok' option isn't support in pre python 3.8. So, need the manual check.
                r.joinpath(n).unlink()
        s = om.sessions.add_standalone(n, r)
        return s
    
    def blank_sg_for(self, name=None, remove=True):
        n = name or inspect.stack()[1][3]
        p = self.root.joinpath(n)
        if remove and p.exists():
            shutil.rmtree(p)

        sg = om.sessions.add_group(n, self.root)
        return sg

class TestSessions(Common):
    def test_sessions_is_accessible(self):
        s = self.sessions
        assert isinstance(s, self.sessions_class)
        assert s.groups == {}
        assert s.standalones == {}
    
    def test_session_groups_can_be_added(self):
        n = "my_group"
        s = self.sessions
        assert n not in s.groups

        grp = s.add_group(n, self.root.joinpath(n))
        assert isinstance(grp, self.group_class)
        assert n in s.groups
        assert n not in s.standalones
        assert grp == s.group(n)
        assert grp == s.groups[n]

    def test_standalone_sessions_can_be_added(self):
        n = "my_standalone"
        s = self.sessions
        assert n not in s.standalones

        st = s.add_standalone(n, self.root)
        assert isinstance(st, self.session_class)
        assert n in s.standalones
        assert n not in s.groups
        assert st == s.standalone(n)
        assert st == s.standalones[n]

    def test_error_on_adding_duplicate_groups(self):
        sg = self.blank_sg_for()
        num_groups = len(self.sessions.groups)
        sg.add_session("s1")
        sg.add_session("s2")
        assert set(sg.keys()) == {"s1", "s2"}

        with pytest.raises(RuntimeError, match=f"Session group '{sg.name}' has already been added"):
            self.sessions.add_group(sg.name, sg.path.parent)

        # Ensure the group wasn't changed
        assert sg.name in self.sessions.groups
        assert num_groups == len(self.sessions.groups)
        assert set(sg.keys()) == {"s1", "s2"}

    def test_error_on_adding_duplicate_sessions(self):
        s = self.blank_session_for()
        num_standalones = len(self.sessions.standalones)
        assert s.name in self.sessions.standalones
        assert "test" not in s
        s["test"] = 123
        assert "test" in s

        with pytest.raises(RuntimeError, match=f"Standalone session '{s.name}' has already been added"):
            self.sessions.add_standalone(s.name, s.path.parent)
 
        assert s.name in self.sessions.standalones
        assert "test" in s
        assert num_standalones == len(self.sessions.standalones)

    def test_groups_can_be_removed(self):
        sg = self.blank_sg_for()
        n = sg.name
        p = sg.path
        assert n in self.sessions.groups
        num_standalones = len(self.sessions.groups)
        assert p.exists() == False

        assert self.sessions.delete_group(sg.name) == True
        assert n not in self.sessions.groups
        assert len(self.sessions.groups) == num_standalones - 1
        assert p.exists() == False

        with pytest.raises(RuntimeError, match=f"Session group {n} has not been created yet!"):
            sg.name
        with pytest.raises(RuntimeError, match=f"Session group {n} has not been created yet!"):
            sg.path

    def test_groups_can_be_removed_and_are_cleaned(self):
        sg = self.blank_sg_for()
        n = sg.name
        sg_path = sg.path
        assert n in self.sessions.groups
        num_standalones = len(self.sessions.groups)
        assert sg_path.exists() == False

        s1 = sg.add_session("s1")
        s2 = sg.add_session("s2")
        s1_path = s1.path
        s2_path = s1.path
        assert s1_path.exists() == False
        assert s2_path.exists() == False

        s1["test1"] = 1
        s2["test2"] = 2
        assert s1_path.exists() == True
        assert s2_path.exists() == True
        assert sg_path.exists() == True

        assert self.sessions.delete_group(sg.name) == True
        assert n not in self.sessions.groups
        assert len(self.sessions.groups) == num_standalones - 1
        assert sg_path.exists() == False
        assert s1_path.exists() == False
        assert s2_path.exists() == False

        with pytest.raises(RuntimeError, match=f"Error encountered retrieving session 's1': Error retrieving group: '{n}'"):
            s1.name
        with pytest.raises(RuntimeError, match=f"Error encountered retrieving session 's2': Error retrieving group: '{n}'"):
            s2.name
        with pytest.raises(RuntimeError, match=f"Session group {n} has not been created yet!"):
            sg.name

    def test_removing_nonexistant_groups_returns_false(self):
        assert "blah" not in self.sessions.groups
        assert self.sessions.delete_group("blah") == False

    def test_standalones_can_be_removed(self):
        s = self.blank_session_for()
        n = s.name
        p = s.path
        assert n in self.sessions.standalones
        num_standalones = len(self.sessions.standalones)
        assert p.exists() == False

        assert self.sessions.delete_standalone(n) == True
        assert n not in self.sessions.standalones
        assert len(self.sessions.standalones) == num_standalones - 1
        assert p.exists() == False

        with pytest.raises(RuntimeError, match=f"Standalone session {n} has not been created yet"):
            s.name
        with pytest.raises(RuntimeError, match=f"Standalone session {n} has not been created yet"):
            s.path

    def test_standalones_can_be_removed_and_are_cleaned(self):
        s = self.blank_session_for()
        n = s.name
        p = s.path
        assert n in self.sessions.standalones
        num_standalones = len(self.sessions.standalones)
        assert p.exists() == False

        assert "test" not in s
        s["test"] = 123
        assert "test" in s
        assert p.exists() == True

        assert self.sessions.delete_standalone(n) == True
        assert n not in self.sessions.groups
        assert len(self.sessions.standalones) == num_standalones - 1
        assert p.exists() == False

        with pytest.raises(RuntimeError, match=f"Standalone session {n} has not been created yet"):
            s.name
        with pytest.raises(RuntimeError, match=f"Standalone session {n} has not been created yet"):
            s.path

    def test_error_removing_nonexistant_standalones(self):
        assert "blah" not in self.sessions.standalones
        assert self.sessions.delete_standalone("blah") == False

class TestIndividualSessions(Common):
    def test_blank_session(self):
        session = self.blank_session_for()
        assert session.path == self.root.joinpath("test_blank_session")
        assert session.name == "test_blank_session"
        assert session.items() == []
    
    def test_simple_data_roundtrip(self):
        s = self.blank_session_for()
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

    def test_roundtrip_for_fancier_objects(self):
        s = self.blank_session_for()
        assert s.get("list test") == None
        s.store("list test", [1, 2, "three", 1.23, -5])
        s.refresh()
        assert s.get("list test") == [1, 2, "three", 1.23, -5]

    def test_roundtrip_for_custom_class(self):
        s = self.blank_session_for()
        assert s.get("dummy") == None
        s.store("dummy", Dummy(1))
        d = s.get("dummy")
        assert isinstance(d, Dummy)
        assert d.data == 1

    def test_clearing_session_values(self):
        s = self.blank_session_for()
        assert s.store("test_str", "str").refresh().get("test_str") == "str"
        assert s.store("test_str2",
                       "str2").refresh().get("test_str2") == "str2"
        assert s.store("test_str3",
                       "str3").refresh().get("test_str3") == "str3"

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
        s = self.blank_session_for()
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
        s = self.blank_session_for()
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

    def test_roundtrip_as_string(self):
        s = self.blank_session_for()
        assert s.store("test_str", "str").refresh().get("test_str") == "str"

    def test_roundtrip_as_int(self):
        s = self.blank_session_for()
        assert s.store("test_int", 1).refresh().get("test_int") == 1
        assert s.store("test_int_neg", -1).refresh().get("test_int_neg") == -1

    def test_roundtrip_as_bool(self):
        s = self.blank_session_for()
        assert s.store("test_true", True).refresh().get("test_true") == True
        assert s.store("test_false",
                       False).refresh().get("test_false") == False

    def test_roundtrip_as_float(self):
        s = self.blank_session_for()
        assert s.store("test_float", 3.14).refresh().get("test_float") == 3.14
        assert s.store("test_float_neg",
                       -3.14).refresh().get("test_float_neg") == -3.14

    def test_session_item_assignment(self):
        s = self.blank_session_for()
        assert "set_test" not in s
        s["set_test"] = True
        assert "set_test" in s
        assert s["set_test"] == True
        s["set_test"] = 1
        assert s["set_test"] == 1

    def test_setting_to_none_deletes_key(self):
        s = self.blank_session_for()

        # Add item
        assert "set_test" not in s
        s["set_test"] = True
        assert "set_test" in s

        # Now, try to remove
        assert "set_test" in s
        s["set_test"] = None
        assert "set_test" not in s

    def test_standalone_session_autoloads(self):
        n = "test_standalone_session_autoloads"
        assert n not in self.sessions.standalones
        s = self.sessions.add_standalone(n, self.prepopulated_root)
        assert s.path.exists()
        assert len(s) == 1
        assert s["test"] == "abc"

    class TestSessionStoreDictLike(Fixture_DictLikeAPI, Common):
        def parameterize(self):
            return {
                "keys": ["t1", "t2", "t3"],
                "klass": str,
                "not_in_dut": "Blah"
            }

        def init_dict_under_test(self):
            s = self.blank_session_for("dict_like_test")
            s.store("t1", "one")
            s.store("t2", "two")
            s.store("t3", "three")

        def boot_dict_under_test(self):
            return self.sessions.standalone("dict_like_test")

class TestSessionGroup(Common):
    def test_new_session_group(self):
        sg = self.blank_sg_for()
        assert sg.path == self.root.joinpath("test_new_session_group")
        assert sg.name == "test_new_session_group"
        assert sg.sessions == {}

    def test_adding_sessions(self):
        sg = self.blank_sg_for()
        r = self.root.joinpath('test_adding_sessions')
        assert "s1" not in sg
        assert sg.get("s2") == None

        s1 = sg.add_session("s1")
        s2 = sg.add_session("s2")
        assert "s1" in sg
        assert "s2" in sg
        assert isinstance(s1, self.session_class)
        assert isinstance(s2, self.session_class)
        assert s1 == sg["s1"]
        assert s2 == sg.get("s2")
        assert s1.path == r.joinpath("s1")
        assert s2.path == r.joinpath("s2")
        assert s1.name == "s1"
        assert s2.name == "s2"
    
    def test_adding_session_content(self):
        sg = self.blank_sg_for()
        s1 = sg.add_session("s1")
        s2 = sg.add_session("s2")

        assert "test" not in s1
        s1["test"] = True
        assert s1["test"] == True
        assert sg["s1"]["test"] == True

        sg2 = self.sessions.groups["test_adding_session_content"]
        assert sg2 == sg
        assert sg2["s1"] == sg["s1"]
        assert sg2["s1"]["test"] == True

    def test_removing_sessions(self):
        sg = self.blank_sg_for()
        s1 = sg.add_session("s1")
        s2 = sg.add_session("s2")
        assert len(sg) == 2

        assert "test" not in s1
        s1["test"] = True
        assert "test" not in s2
        s2["test"] = True
        s1_path = s1.path
        assert s1_path.exists()
        assert s2.path.exists()

        sg.delete_session("s1")
        assert not s1_path.exists()
        assert "s1" not in sg
        assert len(sg) == 1

        assert s2.path.exists()
        assert s2["test"] == True

        with pytest.raises(RuntimeError, match=f"Session s1 has not been added to group {sg.name} yet"):
            s1.name
        with pytest.raises(RuntimeError, match=f"Session s1 has not been added to group {sg.name} yet"):
            s1.path

    def test_removing_nonexistant_sessions_returns_false(self):
        sg = self.blank_sg_for()
        assert "s1" not in sg
        assert sg.delete_session("s1") == False

    def test_session_group_autoloads(self):
        n = "test_session_group_autoloads"
        assert n not in self.sessions.groups
        sg = self.sessions.add_group(n, self.prepopulated_root)
        assert sg.path.exists()
        assert len(sg) == 2
        assert sg["s1"]["test_s1"] == "s1 test"
        assert sg["s2"]["test_s2"] == 's2 test'

    def test_adding_existing_session_does_not_override(self):
        sg = self.blank_sg_for()
        s1 = sg.add_session("s1")
        s2 = sg.add_session("s2")
        assert len(sg) == 2

        assert "test" not in s1
        s1["test"] = True
        assert s1["test"] == True
        assert sg["s1"]["test"] == True

        s1_b = sg.add_session("s1")
        assert s1 == s1_b
        assert s1["test"] == True
        assert s1_b["test"] == True
        assert len(sg) == 2

    class TestSessionGroupDictLike(Fixture_DictLikeAPI, Common):
        def parameterize(self):
            return {
                "keys": ["s1", "s2", "s3"],
                "klass": self.session_class,
                "not_in_dut": "Blah"
            }

        def init_dict_under_test(self):
            s = self.blank_sg_for("sg_dict_like_test")
            s.add_session("s1")
            s.add_session("s2")
            s.add_session("s3")

        def boot_dict_under_test(self):
            return self.sessions.groups["sg_dict_like_test"]