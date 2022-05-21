import pytest, pathlib, origen, _origen
import origen_metal as om
from tests import om_shared
from tests.shared import tmp_dir
from tests.shared import in_new_origen_proc
from configs import session as session_configs

with om_shared():
    from om_tests.utils.test_sessions import Common  # type:ignore


class Base(Common):
    @property
    def app_test_session_root(self):
        return pathlib.Path(tmp_dir().joinpath(".session/__app__"))

    @property
    def user_test_session_root(self):
        return pathlib.Path(
            tmp_dir().joinpath(f".o2/.session/{self.user_sg_name}"))

    @property
    def user_sg_name(self):
        return f"__user__{origen.current_user.id}__"

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
        return origen.current_user.id

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
        assert isinstance(self.sessions,
                          _origen.utility.sessions.OrigenSessions)
        assert isinstance(self.user_sg, self.sg_class)
        assert isinstance(self.app_sg, self.sg_class)

        assert set(
            self.sessions.groups.keys()) == {self.user_sg_name, "__app__"}
        assert len(self.sessions.standalones) == 0

        # Only the default session should be in the group initially
        assert len(self.user_sg) == 1
        assert len(self.app_sg) == 1
        assert "__user__" in self.user_sg
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
        assert self.app_session().path == self.app_test_session_root.joinpath(
            self.app_name)

        self.app_sessions.add_session("blah")
        assert self.app_session(
            "blah").path == self.app_test_session_root.joinpath("blah")

    def test_user_session_paths(self):
        assert self.user_session(
        ).path == self.user_test_session_root.joinpath("__user__")

        self.user_sessions.add_session("blah")
        assert self.user_session(
            "blah").path == self.user_test_session_root.joinpath("blah")

    def test_app_session_aliases(self):
        assert self.app_session() == self.app_session(self.app_name)
        assert self.app_session() == self.app_session(origen.app)
        assert self.app_session() == origen.app.session
        assert self.app_session() != self.app_session("blah")

    def test_user_session_aliases(self):
        assert self.user_session() == self.user_session("__user__")
        assert self.user_session() == origen.current_user.session

    def test_app_and_plugin_session_names_are_auto_added_to_user_session(self):
        assert self.app_name not in self.user_sessions
        self.user_session(origen.app)
        assert self.app_name in self.user_sessions

        assert self.pl_name not in self.user_sessions
        self.user_session(origen.plugin(self.pl_name))
        assert self.pl_name in self.user_sessions

    def test_user_session_for_app(self):
        assert self.user_session(self.app_name) == self.user_session(
            origen.app)
        assert self.user_session(self.app_name) == origen.app.user_session

    def test_plugin_session_names_are_auto_added_to_app_session(self):
        assert self.pl_name not in self.app_sessions
        self.app_session(origen.plugin(self.pl_name))
        assert self.pl_name in self.app_sessions

    def test_plugin_app_session_aliases(self):
        assert self.app_session(
            self.pl_name).path == self.app_test_session_root.joinpath(
                self.pl_name)
        assert self.app_session(self.pl_name) == self.app_session(self.pl)
        assert self.app_session(self.pl_name) == origen.plugin(
            self.pl_name).session

    def test_plugin_user_session_aliases(self):
        assert self.user_session(
            self.pl_name).path == self.user_test_session_root.joinpath(
                self.pl_name)
        assert self.user_session(self.pl_name) == self.user_session(self.pl)
        assert self.user_session(self.pl_name) == origen.plugin(
            self.pl_name).user_session

class TestSessionConfig(Base):
    @property
    def config_dir(self):
        return pathlib.Path(__file__).parent.joinpath("configs/session")

    def test_user_session_root_can_be_updated_from_config(self):
        retn = in_new_origen_proc(mod=session_configs)
        assert retn["root"] == self.config_dir.joinpath(f"user_session_test_root/.o2/.session/{self.user_sg_name}")

    def test_app_session_root_can_be_updated_from_app_config(self):
        retn = in_new_origen_proc(mod=session_configs)
        assert retn["root"] == self.config_dir.joinpath("app_session_test_root/.session/__app__")
