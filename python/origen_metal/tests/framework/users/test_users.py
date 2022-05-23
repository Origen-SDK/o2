from typing import Dict
import pytest, pathlib
from pathlib import Path
import origen_metal as om
from .shared import Base

from .tests__users_basics import T_Users
from .tests__initial_and_current_user import T_InitialAndCurrentUser
from .tests__datasets import T_Datasets
from .tests__user_motives import T_UserMotives
from .tests__populating import T_PopulatingUsers
from .tests__datasets import Base as DSBase
from .tests__user_sessions import T_UserSessions

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

class TestUserSessions(T_UserSessions):
    pass

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
