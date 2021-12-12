import pytest, pathlib, origen, _origen
import origen_metal as om
from tests import om_shared
from tests.shared import tmp_dir

with om_shared():
    from om_tests.utils.test_sessions import Common

class Base(Common):
    @property
    def app_test_session_root(self):
        return pathlib.Path(tmp_dir().joinpath(".session/__app__"))

    @property
    def user_test_session_root(self):
        return pathlib.Path(tmp_dir().joinpath(".o2/.session/__user__"))

    @property
    def user_sg(self):
        return origen.sessions.user_sessions

    @property
    def app_sg(self):
        return origen.sessions.app_sessions

    @property
    def user_sessions(self):
        return origen.sessions.user_sessions

    @property
    def app_sessions(self):
        return origen.sessions.app_sessions

    def user_session(self, s=None):
        return origen.sessions.user_session(s)

    def app_session(self, s=None):
        return origen.sessions.app_session(s)

    @property
    def app_name(self):
        return origen.app.name

    @property
    def user_id(self):
        return origen.current_user().id

    @property
    def pl(self):
        return origen.plugin(self.pl_name)

    @property
    def pl_name(self):
        return "python_plugin"

    @property
    def sessions(self):
        return origen.sessions

    @pytest.fixture
    def clear_test_sessions(self):
        assert self.sessions.user_session_root == self.user_test_session_root
        assert self.sessions.app_session_root == self.app_test_session_root

        self.sessions.clean()
        assert self.app_test_session_root.exists() is False
        assert self.user_test_session_root.exists() is False


class TestOrigenSessions(Base):
    def test_blank_sessions(self, clear_test_sessions):
        ''' Should return a Session class but not actually create any files yet'''
        assert isinstance(self.sessions, _origen.utility.sessions.OrigenSessions)
        assert isinstance(self.user_sg, self.sg_class)
        assert isinstance(self.app_sg, self.sg_class)

        assert set(self.sessions.groups.keys()) == {"__user__", "__app__"}
        assert len(self.sessions.standalones) == 0

        # Only the default session should be in the group initially
        assert len(self.user_sg) == 1
        assert len(self.app_sg) == 1
        assert self.user_id in self.user_sg
        assert self.app_name in self.app_sg

        assert isinstance(self.sessions.user_session(), self.ss_class)
        assert isinstance(self.sessions.app_session(), self.ss_class)
        assert not self.app_test_session_root.exists()
        assert not self.user_test_session_root.exists()
        assert len(self.user_session()) == 0
        assert len(self.app_session()) == 0

        # TODO update this to enums (when available)
        assert str(self.user_sg.permissions) == "private"
        assert str(self.app_sg.permissions) == "group_writable"

    def test_app_session_paths(self):
        assert self.app_session().path == self.app_test_session_root.joinpath(self.app_name)
        assert self.app_session("blah").path == self.app_test_session_root.joinpath("blah")

    def test_user_session_paths(self):
        assert self.user_session().path == self.user_test_session_root.joinpath(self.user_id)
        assert self.user_session("blah").path == self.user_test_session_root.joinpath("blah")

    def test_app_session_aliases(self):
        assert self.app_session() == self.app_session(self.app_name)
        assert self.app_session() == self.app_session(origen.app)
        assert self.app_session() == origen.app.session
        assert self.app_session() != self.app_session("blah")

    def test_user_session_aliases(self):
        assert self.user_session() == self.user_session(self.user_id)
        assert self.user_session() == origen.current_user().session

    def test_user_session_for_app(self):
        assert self.user_session(self.app_name) == self.user_session(origen.app)
        assert self.user_session(self.app_name) == origen.app.user_session

    def test_plugin_app_session_aliases(self):
        assert self.app_session(self.pl_name).path == self.app_test_session_root.joinpath(self.pl_name)
        assert self.app_session(self.pl_name) == self.app_session(self.pl)
        assert self.app_session(self.pl_name) == origen.plugin(self.pl_name).session

    def test_plugin_user_session_aliases(self):
        assert self.user_session(self.pl_name).path == self.user_test_session_root.joinpath(self.pl_name)
        assert self.user_session(self.pl_name) == self.user_session(self.pl)
        assert self.user_session(self.pl_name) == origen.plugin(self.pl_name).user_session

class TestSessionStore(Base):
    def test_adding_app_sessions(self):
        n = "app_session"
        s = self.app_session(n)
        assert isinstance(s, self.ss_class)
        assert s.path == self.app_test_session_root.joinpath(n)

    def test_adding_user_sessions(self):
        n = "my_session"
        assert n not in self.user_sessions
        s = self.user_session(n)
        assert isinstance(s, self.ss_class)
        assert s.path == self.user_test_session_root.joinpath(n)

    def test_default_user_session(self):
        assert self.user_session().get("test") == None
        self.user_session().store("test", 123)
        assert self.user_session().get("test") == 123

    def test_roundtrip_user_session(self):
        s = self.user_session("test_roundtrip_user_session")
        assert s.get("test") == None
        assert s.path.exists() is False

        s.store("test", "abc")
        s.store("test2", "def")
        assert s.path.exists() is True

        assert s.get("test") == "abc"
        assert s.get("test2") == "def"

        # Should not be added to default user session
        assert self.user_session().get("test") == 123

    def test_getting_all_user_sessions(self):
        ''' This will include all added sessions under 'user', even if no data (and no actual file) is present'''
        assert set(self.user_sessions.keys()) == {
            self.user_id, self.app_name, 'my_session', 'python_plugin', 'blah', 'test_roundtrip_user_session'
        }

    def test_default_app_session(self):
        assert self.app_session().get("test") == None
        self.app_session().store("test", 123)
        assert self.app_session().get("test") == 123

    def test_roundtrip_app_session(self):
        s = self.app_session("test_roundtrip_app_session")
        assert s.get("test") == None
        assert s.path.exists() is False

        s.store("test", "abc")
        s.store("test2", "def")
        assert s.path.exists() is True

        assert s.get("test") == "abc"
        assert s.get("test2") == "def"

        # Should not be added to default user session
        assert self.app_session().get("test") == 123

    def test_getting_all_app_sessions(self):
        assert set(self.app_sessions.keys()) == {
            self.app_name, 'app_session', 'python_plugin', 'blah', 'test_roundtrip_app_session'
        }
