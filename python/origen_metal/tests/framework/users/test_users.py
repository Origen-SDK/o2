from typing import Dict
import pytest, pathlib
from pathlib import Path
import origen_metal as om
from .shared import Base

from origen_metal._helpers import in_new_proc

from .tests__users_basics import T_Users
from .tests__initial_and_current_user import T_InitialAndCurrentUser
from .tests__datasets import T_Datasets
from .tests__user_motives import T_UserMotives
from .tests__populating import T_PopulatingUsers
from .tests__datasets import Base as DSBase
from .tests__user_sessions import T_UserSessions
from .tests__validating_passwords import T_ValidatingPasswords
from .tests__user_roles import T_UserRoles

from .in_new_proc_funcs import try_home_dir

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

class TestValidatingPasswords(T_ValidatingPasswords):
    pass

class TestUserRoles(T_UserRoles):
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

    @property
    def logged_in_user_home_dir(self):
        return pathlib.Path.home()

    @property
    def routed_user(self):
        if not hasattr(self, "_routed_user"):
            self._routed_user = False
        return self._routed_user

    @property
    def routed_dataset(self):
        if not hasattr(self, "_routed_dataset"):
            self._routed_dataset = False
        return self._routed_dataset

    callback_home_dir = Path("/callback_home/")

    def lookup_home_dir_function(self, user, dataset, is_current):
        if user.id == "return_str":
            return str(self.callback_home_dir.joinpath("str/u_str"))
        elif user.id == "return_error":
            raise RuntimeError("Encountered user 'return_error'!")
        elif user.id == "return_none":
            return None
        elif user.id == self.logged_in_id:
            if dataset is None:
                self._routed_user = True
            else:
                self._routed_dataset = True
            return False
        elif user.id == "return_true":
            return True
        else:
            hd = self.callback_home_dir.joinpath(user.id)
            if dataset is None:
                return hd
            else:
                return hd.joinpath(f"dataset/{dataset.dataset_name}")

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

    def test_set_method(self, unload_users, u, d, d2, ddk, d2_name, hd, hd2, hd3):
        assert u.home_dir is None
        assert d.home_dir is None
        assert d2.home_dir is None

        u.set_home_dir(hd)
        assert u.home_dir == hd
        assert d.home_dir == hd
        assert d2.home_dir is None

        d2.set_home_dir(hd2)
        assert u.home_dir == hd
        assert d.home_dir == hd
        assert d2.home_dir == hd2

        d.set_home_dir(hd3)
        assert u.home_dir == hd3
        assert d.home_dir == hd3
        assert d2.home_dir == hd2

        hd_str = "home/dir/str"
        d.set_home_dir(hd_str)
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

    def test_clearing_home_dir(self, unload_users, u, d, d2, hd, hd2, hd3):
        # Add a 3rd dataset outside of the data lookup hierarchy
        # TODO add option to not add to data lookup hierarchy?
        old_hierarchy = u.data_lookup_hierarchy
        d3 = u.add_dataset("ds3")
        d3.home_dir = hd3
        u.data_lookup_hierarchy = old_hierarchy

        d.set_home_dir(hd)
        d2.set_home_dir(hd2)
        assert u.home_dir == hd
        assert d.home_dir == hd
        assert d2.home_dir == hd2

        # Clears all home directories in the hierarchy
        # d3 (not in hierarchy) should remain
        u.clear_home_dir()
        assert u.home_dir is None
        assert d.home_dir is None
        assert d2.home_dir is None
        assert d3.home_dir == hd3

        d.set_home_dir(hd)
        d2.set_home_dir(hd2)
        assert u.home_dir == hd
        assert d.home_dir == hd
        assert d2.home_dir == hd2
        assert d3.home_dir == hd3

        # Clear a specific home dir
        d.clear_home_dir()
        assert u.home_dir == hd2
        assert d.home_dir is None
        assert d2.home_dir == hd2
        assert d3.home_dir == hd3

        # Use setter method to clear
        u.home_dir = None
        assert u.home_dir is None
        assert d.home_dir is None
        assert d2.home_dir is None
        assert d3.home_dir == hd3

        d3.home_dir = None
        assert d3.home_dir is None

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

    def test_looking_up_home_dir(self, unload_users, users, ddk):
        u = users.add(self.logged_in_id)
        assert u.home_dir is None
        assert u.datasets[ddk].home_dir is None

        u.set_home_dir()
        hd = self.logged_in_user_home_dir
        assert u.home_dir == hd
        assert u.datasets[ddk].home_dir == hd

    def test_looking_up_home_dir_from_callback(self, fresh_frontend, unload_users, users, u):
        assert users.lookup_home_dir_function is None
        users.lookup_home_dir_function = self.lookup_home_dir_function
        assert users.lookup_home_dir_function == users.lookup_home_dir_function

        assert u.home_dir is None
        u.set_home_dir()
        assert u.home_dir == self.callback_home_dir.joinpath(u.id)

        u_str = users.add("return_str")
        assert u_str.home_dir is None
        u_str.set_home_dir()
        assert u_str.home_dir == self.callback_home_dir.joinpath("str/u_str")

    def test_looking_up_home_dir_for_dataset(self, fresh_frontend, unload_users, users, ddk, hd):
        u = users.add(self.logged_in_id)
        d = u.datasets[ddk]
        d2 = u.add_dataset("ds2", as_topmost=False)
        assert u.home_dir is None
        assert d.home_dir is None

        u.home_dir = hd
        assert u.home_dir == hd
        assert d.home_dir == hd
        assert d2.home_dir is None

        d2.set_home_dir()
        assert u.home_dir == hd
        assert d2.home_dir == self.logged_in_user_home_dir

    def test_looking_up_home_dir_for_dataset_from_callback(self, fresh_frontend, users, unload_users, u, d, d2):
        users.lookup_home_dir_function = self.lookup_home_dir_function
        assert d.home_dir is None
        assert d2.home_dir is None

        d.set_home_dir()
        assert d.home_dir == self.callback_home_dir.joinpath(f"{u.id}/dataset/{d.dataset_name}")
        assert d2.home_dir is None

        d2.set_home_dir()
        assert d.home_dir == self.callback_home_dir.joinpath(f"{u.id}/dataset/{d.dataset_name}")
        assert d2.home_dir == self.callback_home_dir.joinpath(f"{u.id}/dataset/{d2.dataset_name}")

    def test_lookup_home_dir_with_none_argument(self, fresh_frontend, users, unload_users, u, d, d2):
        users.lookup_home_dir_function = self.lookup_home_dir_function
        assert u.home_dir is None
        assert d.home_dir is None
        assert d2.home_dir is None

        u.set_home_dir(None)
        assert u.home_dir == self.callback_home_dir.joinpath(u.id)
        assert d.home_dir == self.callback_home_dir.joinpath(u.id)
        assert d2.home_dir is None

        d2.set_home_dir(None)
        assert d2.home_dir == self.callback_home_dir.joinpath(f"{u.id}/dataset/{d2.dataset_name}")

    def test_error_in_callback(self, fresh_frontend, unload_users, users, hd, hd2, hd3):
        users.lookup_home_dir_function = self.lookup_home_dir_function
        u = users.add("return_error")
        d2 = u.add_dataset("ds2", as_topmost=False)

        err = "Encountered Exception 'RuntimeError' with message: Encountered user 'return_error'!"

        assert u.home_dir is None
        with pytest.raises(RuntimeError, match=err):
            u.set_home_dir()
        assert u.home_dir is None

        u.home_dir = hd
        with pytest.raises(RuntimeError, match=err):
            u.set_home_dir()
        assert u.home_dir == hd

        assert d2.home_dir is None
        with pytest.raises(RuntimeError, match=err):
            d2.set_home_dir()
        assert d2.home_dir is None

        d2.home_dir = hd3
        with pytest.raises(RuntimeError, match=err):
            d2.set_home_dir()
        assert d2.home_dir == hd3

    def test_callback_falls_back_to_default(self, fresh_frontend, unload_users, users):
        assert self.routed_user is False
        users.lookup_home_dir_function = self.lookup_home_dir_function
        u = users.add(self.logged_in_id)
        d = u.add_dataset("ds")
        d2 = u.add_dataset("ds2", as_topmost=False)
        assert u.home_dir is None

        u.set_home_dir()
        assert u.home_dir == self.logged_in_user_home_dir
        assert self.routed_user is True

        assert d2.home_dir is None
        assert self.routed_dataset is False
        d2.set_home_dir()
        assert u.home_dir == self.logged_in_user_home_dir
        assert self.routed_dataset is True

    def test_callback_returns_none(self, fresh_frontend, unload_users, users, hd, hd2):
        users.lookup_home_dir_function = self.lookup_home_dir_function
        u = users.add("return_none")
        d = u.add_dataset("ds")
        d2 = u.add_dataset("ds2", as_topmost=False)

        assert u.home_dir is None
        u.set_home_dir()
        assert u.home_dir is None

        u.home_dir = hd
        assert u.home_dir == hd
        u.set_home_dir()
        assert u.home_dir is None

        u.home_dir = hd
        assert d2.home_dir is None
        d2.set_home_dir()
        assert d2.home_dir is None
        assert u.home_dir == hd

        d2.home_dir = hd2
        assert d2.home_dir == hd2
        d2.set_home_dir()
        assert d2.home_dir is None
        assert u.home_dir == hd

    def test_callback_returns_true(self, fresh_frontend, unload_users, users, hd, hd2):
        users.lookup_home_dir_function = self.lookup_home_dir_function
        u = users.add("return_true")
        d = u.add_dataset("ds")
        err = "'True' is not a valid return value when looking up a user's home directory"

        u.home_dir = hd
        with pytest.raises(RuntimeError, match=err):
            u.set_home_dir()
        assert u.home_dir == hd

        d.home_dir = hd2
        with pytest.raises(RuntimeError, match=err):
            d.set_home_dir()
        assert u.home_dir == hd2

    def test_inappropriate_home_dir_for_user(self, fresh_frontend, unload_users, users, u, hd):
        assert u.home_dir is None
        err = f"Home directory '.*' is not appropriate for current user with id '{u.id}'"

        with pytest.raises(RuntimeError, match=err):
            u.set_home_dir()
        assert u.home_dir is None
        u.home_dir = hd

        d = u.add_dataset("ds")
        with pytest.raises(RuntimeError, match=err):
            d.set_home_dir()
        assert d.home_dir is None
        assert u.home_dir == hd

    def test_error_when_env_is_not_set(self, fresh_frontend, unload_users, users, capfd):
        in_new_proc(func=try_home_dir, expect_fail=True)
        if om.running_on_windows:
            env = "USERPROFILE"
        else:
            env = "HOME"
        assert f"RuntimeError: Please set environment variable {env} to point to your home directory, then try again" in capfd.readouterr().err
    
    # TEST_NEEDED
    @pytest.mark.skip
    def test_om_home_dir(self):
        raise NotImplementedError()
        assert om.home_dir == hd
        assert om.home_dir == hd1
        assert om.home_dir == hd2

    # TEST_NEEDED
    @pytest.mark.skip
    def test_error_on_om_home_dir_with_no_current_user(self):
        raise NotImplementedError()

    # TEST_NEEDED
    # TODO move elsewhere?
    @pytest.mark.skip
    def test_origen_dot_dir(self):
        raise NotImplementedError()

    # TEST_NEEDED
    @pytest.mark.skip
    def test_setting_home_dir_from_frontend(self):
        raise NotImplementedError()
