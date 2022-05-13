from typing import Dict
import pytest, pathlib
from pathlib import Path
import origen_metal as om

# from framework import FilePermissions
from .shared import Base

from .tests__users_basics import T_Users
from .tests__initial_and_current_user import T_InitialAndCurrentUser
from .tests__datasets import T_Datasets
from .tests__user_motives import T_UserMotives
from .tests__populating import T_PopulatingUsers
from .tests__datasets import Base as DSBase


class TestUsers(T_Users):
    pass


class TestInitialAndCurrentUser(T_InitialAndCurrentUser):
    pass


class TestUnloadingUsers(Base):
    def test_unloading_users(self, users, u_id):
        users.current_user = u_id
        assert users.current_user.id == u_id
        assert users.initial_user.id == u_id
        users.unload()

        assert len(users.ids) == 0
        assert users.current == None
        assert users.current_user == None
        assert users.initial == None
        assert users.initial_user == None
        assert om.current_user == None


class TestDatasets(T_Datasets):
    pass


class TestUserMotives(T_UserMotives):
    pass


class TestPopulatingUsers(T_PopulatingUsers):
    pass


# TODO move this to another file
from .shared import Base
from tests.utils.test_sessions import Common as SessionsBase
from tests.test_file_permissions import new_fp


class TestUserSession(Base, SessionsBase):
    @property
    def default_root(self):
        return None

    @property
    def default_offset(self):
        return pathlib.Path('./.o2/.session')

    @property
    def default_fp(self):
        return new_fp('private')

    @property
    def updated_root(self):
        return pathlib.Path("my/path/")

    @property
    def updated_offset(self):
        return pathlib.Path('at/my/offset')

    @property
    def updated_fp(self):
        return new_fp('public')

    @pytest.fixture
    def u_sg_name(self):
        return f"__user__{self.user_id_root}__"

    @pytest.fixture
    def u_sg_def_path(self, u_sg_name):
        return self.default_offset.joinpath(u_sg_name)

    @pytest.fixture
    def def_ss_name(self):
        return "__user__"

    @pytest.fixture
    def u_ss_def_path(self, u_sg_def_path, def_ss_name):
        return u_sg_def_path.joinpath(def_ss_name)

    def assert_sc(self,
                  sc,
                  root=None,
                  offset=None,
                  fp=None,
                  offset_is_none=False):
        assert sc.root == (
            root or self.default_root
        )  # for completeness even though both may resolve to None
        if offset_is_none:
            assert sc.offset == None
        else:
            assert sc.offset == (offset or self.default_offset)
        assert sc.file_permissions == (fp or self.default_fp)
        assert sc.fp == (fp or self.default_fp)

    def assert_updated_sc(self,
                          sc,
                          root=None,
                          offset=None,
                          fp=None,
                          offset_is_none=False):
        return self.assert_sc(sc, root or self.updated_root, offset
                              or self.updated_offset, fp or self.updated_fp,
                              offset_is_none)

    def test_session_defaults_from_users(self, unload_users, users):
        sc = users.session_config
        assert isinstance(sc, self.users_session_config_class)

    def test_session_config_is_accessible_per_user(self, unload_users, u):
        sc = u.session_config
        assert isinstance(sc, self.user_session_config_class)
        self.assert_sc(sc)

    def test_session_config_defaults_can_be_updated(self, unload_users, users):
        sc = users.session_config
        self.assert_sc(sc)

        sc.root = self.updated_root
        sc.offset = self.updated_offset
        sc.file_permissions = self.updated_fp

        self.assert_updated_sc(sc)

    def test_session_config_propagates_to_users(self, u):
        sc = u.session_config
        self.assert_updated_sc(sc)

    def test_session_config_can_be_updated_per_user(self, u):
        sc = u.session_config
        self.assert_updated_sc(sc)
        sc.file_permissions = new_fp('world_writable')
        self.assert_updated_sc(sc, fp=new_fp('world_writable'))

    def test_session_group_is_accessible_and_lazily_created(
            self, unload_users, u, u_sg_name, u_sg_def_path):
        assert u_sg_name not in self.sessions.groups
        grp = u.sessions
        assert u_sg_name in self.sessions.groups
        assert isinstance(grp, self.sg_class)

        assert grp.name == u_sg_name
        assert grp.path == u_sg_def_path
        assert grp.path.exists() is False

    def test_session_is_accessible_and_lazily_created(self, unload_sessions,
                                                      unload_users, u,
                                                      u_sg_name, def_ss_name,
                                                      u_ss_def_path,
                                                      u_sg_def_path):
        assert u_sg_name not in self.sessions.groups

        # Create and get the default session
        session = u.session
        assert u_sg_name in self.sessions.groups
        assert def_ss_name in u.sessions
        assert isinstance(session, self.ss_class)

        assert session.path == u_ss_def_path
        assert session.name == def_ss_name
        assert session.path.exists() is False
        assert def_ss_name in u.sessions
        assert u.sessions[def_ss_name] == u.session

        # Create and get a namespaced session
        s2_name = "s2"
        assert s2_name not in u.sessions
        s2 = u.sessions.add_session(s2_name)
        assert isinstance(s2, self.ss_class)
        assert s2_name in u.sessions
        assert s2.name == s2_name
        assert s2.path == u_sg_def_path.joinpath(s2_name)
        assert u.sessions[s2_name] != u.session

    def test_session_config_updates_for_future_users(self, users, u):
        sc = users.session_config
        self.assert_sc(sc)

        sc.root = self.updated_root
        sc.offset = self.updated_offset
        sc.fp = self.updated_fp

        self.assert_updated_sc(self.user(2).session_config)

        # User created prior to updates should be unchanged
        self.assert_sc(u.session_config)

    def test_user_session_config_is_updated_only_pre_session_creation(
            self, unload_users, unload_sessions, u, u_sg_name):
        assert u_sg_name not in self.sessions.groups
        sc = u.session_config
        self.assert_sc(sc)

        sc.root = self.updated_root
        sc.offset = self.updated_offset
        self.assert_updated_sc(sc, fp=new_fp('private'))

        s = u.sessions
        assert s.path == self.updated_root.joinpath(
            self.updated_offset).joinpath(u_sg_name)

        # Can still query the session config
        self.assert_updated_sc(sc, fp=new_fp('private'))

        # But cannot update any values
        err = f"The session config cannot be updated for user '{u.id}' after the session has been created"
        with pytest.raises(RuntimeError, match=err):
            sc.root = "somewhere_else"
        with pytest.raises(RuntimeError, match=err):
            sc.offset = "anywhere_else"
        with pytest.raises(RuntimeError, match=err):
            sc.file_permissions = new_fp('public')
        with pytest.raises(RuntimeError, match=err):
            sc.fp = new_fp('public')

        self.assert_updated_sc(sc, fp=new_fp('private'))

    def test_when_user_session_already_exists(self, unload_users,
                                              unload_sessions, u_sg_name):
        self.sessions.add_group(u_sg_name, root="/random/place")
        u = self.user()
        with pytest.raises(
                RuntimeError,
                match=
                f"Session group '{u_sg_name}' does not match the session config for user '{u.id}'"
        ):
            u.session

    def test_no_root_results_in_users_home_dir_for_top_dataset(
            self, unload_users, unload_sessions, u, u_sg_name, def_ss_name):
        u.home_dir = "home/dir"
        assert u.session_config.root is None
        s = u.session
        assert s.path == pathlib.Path("home/dir").joinpath(
            self.default_offset).joinpath(u_sg_name).joinpath(def_ss_name)

    def test_no_offset(self, unload_users, unload_sessions, u, u_sg_name,
                       def_ss_name):
        sc = u.session_config
        sc.root = "home/dir"
        sc.offset = None
        s = u.session
        assert s.path == pathlib.Path(f"home/dir/{u_sg_name}/{def_ss_name}")

    @pytest.mark.parametrize("target", ("users", "user"),
                             ids=["users", "user"])
    def test_error_on_absolute_offsets(self, unload_users, unload_sessions,
                                       users, target):
        if target == "users":
            sc = users.session_config
        elif target == "user":
            u = self.user()
            sc = u.session_config
        d = "/home/dir"
        if om.running_on_windows:
            d = f"C:{d}"

        msg = fr"Absolute offsets are not allowed in a user's session config \(given: {d}\)"
        assert sc.offset == self.default_offset
        with pytest.raises(RuntimeError, match=msg):
            sc.offset = d
        assert sc.offset == self.default_offset

    @pytest.mark.parametrize("target", ("users", "user"),
                             ids=["users", "user"])
    def test_root_type_acceptance(self, unload_users, unload_sessions, users,
                                  target):
        if target == "users":
            sc = users.session_config
        elif target == "user":
            u = self.user()
            sc = u.session_config

        assert sc.root == None

        # Try as pathlib.Path
        p = Path("home/as/pathlib")
        sc.root = p
        assert sc.root == p

        # Try as String
        p = "home/as/string"
        sc.root = p
        assert sc.root == Path(p)

        # Try as None
        sc.root = None
        assert sc.root is None

    @pytest.mark.parametrize("target", ("users", "user"),
                             ids=["users", "user"])
    def test_offset_type_acceptance(self, unload_users, unload_sessions, users,
                                    target):
        if target == "users":
            sc = users.session_config
        elif target == "user":
            u = self.user()
            sc = u.session_config

        assert sc.offset == self.default_offset

        # Try as pathlib.Path
        p = Path("offset/as/pathlib")
        sc.offset = p
        assert sc.offset == p

        # Try as String
        p = "offset/as/string"
        sc.offset = p
        assert sc.offset == Path(p)

        # Try as None
        sc.offset = None
        assert sc.offset is None

    @pytest.mark.parametrize("target", ("users", "user"),
                             ids=["users", "user"])
    def test_fp_and_file_permissions_methods_are_equivalent(
            self, unload_users, unload_sessions, users, target):
        if target == "users":
            sc = users.session_config
        elif target == "user":
            u = self.user()
            sc = u.session_config

        public_fp = new_fp("public")
        private_fp = new_fp("private")

        sc = users.session_config
        assert sc.fp != public_fp
        assert sc.file_permissions != public_fp

        # Try with file permissions object
        sc.fp = public_fp
        assert sc.file_permissions == public_fp
        assert sc.fp == public_fp

        sc.file_permissions = private_fp
        assert sc.file_permissions == private_fp
        assert sc.fp == private_fp

        # Try with string
        sc.fp = "public"
        assert sc.file_permissions == public_fp
        assert sc.fp == public_fp

        sc.file_permissions = "private"
        assert sc.file_permissions == private_fp
        assert sc.fp == private_fp

        # Try with integer
        sc.fp = 0o755
        assert sc.file_permissions == public_fp
        assert sc.fp == public_fp

        sc.file_permissions = 0o700
        assert sc.file_permissions == private_fp
        assert sc.fp == private_fp


class TestUserHomeDirectory(DSBase):
    @pytest.fixture
    def hd(self):
        return Path("test/home/dir")

    @pytest.fixture
    def hd1(self, hd):
        return hd.joinpath("1")

    @pytest.fixture
    def hd2(self, hd):
        return hd.joinpath("2")

    @pytest.fixture
    def hd3(self, hd):
        return hd.joinpath("3")

    def test_home_dir_starts_as_none(self, unload_users, u, ddk):
        assert u.home_dir is None
        assert u.datasets[ddk].home_dir is None

    def test_home_dir_can_be_set(self, u, d, hd):
        u.home_dir = hd
        assert u.home_dir == hd
        assert d.home_dir == hd

        hd_str = "home/dir/str"
        u.home_dir = hd_str
        assert u.home_dir == Path(hd_str)
        assert d.home_dir == Path(hd_str)

    def test_home_dir_can_be_set_and_retrieved_per_dataset(
            self, unload_users, u, d, d2, hd, hd2, hd3):
        assert u.home_dir is None
        assert d.home_dir is None
        assert d2.home_dir is None

        u.home_dir = hd
        assert u.home_dir == hd
        assert d.home_dir == hd
        assert d2.home_dir is None

        d2.home_dir = hd2
        assert u.home_dir == hd
        assert d.home_dir == hd
        assert d2.home_dir == hd2

        d.home_dir = hd3
        assert u.home_dir == hd3
        assert d.home_dir == hd3
        assert d2.home_dir == hd2

        hd_str = "home/dir/str"
        d.home_dir = hd_str
        assert u.home_dir == Path(hd_str)
        assert d.home_dir == Path(hd_str)
        assert d2.home_dir == hd2

    def test_home_dir_obeys_hierarchy(self, unload_users, u, d, d2, ddk,
                                      d2_name, hd1, hd2):
        assert u.data_lookup_hierarchy == [ddk, d2_name]
        assert u.home_dir is None

        d.home_dir = hd1
        d2.home_dir = hd2

        assert u.home_dir == hd1

        u.data_lookup_hierarchy = [d2_name, ddk]
        assert u.home_dir == hd2

        u.data_lookup_hierarchy = []
        with pytest.raises(
                RuntimeError,
                match=
                "Dataset hierarchy is empty! Data lookups must explicitly name the dataset to query"
        ):
            u.home_dir

    def test_home_dir_iterates_through_hierarchy(self, unload_users, u, d, d2,
                                                 ddk, d2_name, hd1, hd2):
        assert u.data_lookup_hierarchy == [ddk, d2_name]
        assert u.home_dir is None

        d.home_dir is None
        d2.home_dir = hd2
        assert u.home_dir == hd2

        d.home_dir = hd1
        assert u.home_dir == hd1

    def test_requiring_a_home_dir_is_set(self, unload_users, u, hd1):
        assert u.home_dir is None
        with pytest.raises(
                RuntimeError,
                match=
                f"Required a home directory for user '{u.id}' but none has been set"
        ):
            u.require_home_dir

        u.home_dir = hd1
        assert u.require_home_dir == hd1

    def test_requiring_a_home_dir_is_set_per_dataset(self, unload_users, u, d,
                                                     d2, ddk, hd1, hd2):
        assert u.home_dir is None
        d2.home_dir = hd2
        assert u.home_dir == hd2

        with pytest.raises(
                RuntimeError,
                match=
                f"Required a home directory for user '{u.id}' and dataset '{ddk}', but none has been set"
        ):
            d.require_home_dir

        d.home_dir = hd1
        assert d.require_home_dir == hd1

    # TEST_NEEDED
    @pytest.mark.skip
    def test_auto_setting_home_dir(self):
        raise NotImplementedError()

    # TEST_NEEDED
    @pytest.mark.skip
    def test_setting_home_dir_from_frontend(self):
        raise NotImplementedError()
