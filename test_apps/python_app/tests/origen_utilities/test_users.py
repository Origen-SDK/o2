import origen, pytest, getpass
from tests.shared import in_new_origen_proc
from configs import users as user_configs
from tests import om_shared

with om_shared():
    from om_tests.framework.users.shared import Base as UsersBase  # type:ignore


class TestUsers(UsersBase):
    def test_users_are_accessible(self):
        assert isinstance(origen.users, self.users_class)

    def test_current_and_initial_user_are_set_upon_boot(self):
        assert len(origen.users) == 1
        assert origen.users.current_user.id == self.logged_in_id
        assert origen.users.initial_user.id == self.logged_in_id
        assert isinstance(origen.current_user, self.user_class)
        assert origen.current_user.id == self.logged_in_id
        assert isinstance(origen.initial_user, self.user_class)
        assert origen.initial_user.id == self.logged_in_id

    def test_datasets_are_set_upon_boot(self, def_dsc_dict):
        dsets = {"test", "test2", "backup", "forumsys", "git"}
        assert set(origen.users.datasets.keys()) == dsets
        assert set(origen.current_user.datasets.keys()) == dsets
        assert dict(origen.users.datasets["test"]) == def_dsc_dict
        assert dict(origen.users.datasets["test2"]) == def_dsc_dict
        assert dict(origen.users.datasets["backup"]) == def_dsc_dict
        assert dict(
            origen.users.datasets["forumsys"]) == self.def_dsc_dict_with(
                category="ldaps",
                data_store="forumsys",
                auto_populate=False,
                should_validate_password=True)
        assert dict(origen.users.datasets["git"]) == self.def_dsc_dict_with(
            data_store="git", auto_populate=False)

    def test_dataset_hierarchy_is_set_upon_boot(self):
        assert origen.users.data_lookup_hierarchy == ["test", "backup"]
        assert origen.current_user.data_lookup_hierarchy == ["test", "backup"]

    def test_dataset_motives_are_set_from_config(self):
        assert origen.users.motives == {"rc": "git", "just because": "test2"}

    def test_current_user_can_be_set(self):
        assert origen.current_user.id == self.logged_in_id
        u = origen.users.add(self.user_id_root)
        origen.users.set_current_user(u)
        assert origen.current_user.id == u.id

        # Switch the current user back to the logged in user
        origen.users.set_current_user(self.logged_in_id)
        assert origen.current_user.id == self.logged_in_id

    def test_autopopulated_user(self):
        in_new_origen_proc(mod=user_configs)


class TestUserConfigSetups(UsersBase):
    @pytest.fixture
    def dsets(self):
        return set(['test', 'test2', 'git', 'forumsys', 'backup'])

    @pytest.mark.skip
    def test_autoloading_current_user_can_be_disabled(self):
        raise NotImplementedError

    def test_error_on_invalid_datasets_in_hierarchy(self, capfd, dsets):
        retn = in_new_origen_proc(mod=user_configs)
        assert retn["users__data_lookup_hierarchy"] == []
        assert retn["data_lookup_hierarchy"] == []

        assert set(retn["users__datasets"]) == dsets
        assert set(retn["datasets"]) == dsets

        stdout = capfd.readouterr().out
        assert "Encountered an error when initializing users: Error encountered setting the default lookup hierarchy"
        assert "Dataset 'blah' does not exists and cannot be used in the data lookup hierarchy"
        assert "Forcing empty dataset lookup hierarchy..." in stdout

    def test_error_on_duplicate_datasets_in_hierarchy(self, capfd, dsets):
        retn = in_new_origen_proc(mod=user_configs)
        assert retn["users__data_lookup_hierarchy"] == []
        assert retn["data_lookup_hierarchy"] == []

        assert set(retn["users__datasets"]) == dsets
        assert set(retn["datasets"]) == dsets

        stdout = capfd.readouterr().out
        assert "Encountered an error when initializing users: Error encountered setting the default lookup hierarchy"
        assert "Dataset 'test' can only appear once in the dataset hierarchy (first appearance at index 0 - duplicate at index 2)" in stdout
        assert "Forcing empty dataset lookup hierarchy..." in stdout

    def test_error_on_default_data_and_invalid_hierarchy(self, capfd, ddk):
        ''' As no datasets were given, giving a hierarchy is not allowed '''
        retn = in_new_origen_proc(mod=user_configs)
        assert retn["users__datasets"] == [ddk]
        assert retn["datasets"] == [ddk]
        assert retn["users__hierarchy"] == []
        assert retn["hierarchy"] == []

        stdout = capfd.readouterr().out
        assert "Encountered an error when initializing users: Error encountered setting the default lookup hierarchy"
        assert "Dataset 'test' does not exists and cannot be used in the data lookup hierarchy"
        assert "Forcing empty dataset lookup hierarchy..." in stdout

    def test_single_dataset_and_default_hierarchy(self):
        ''' Default hierarchy should be the only dataset '''
        retn = in_new_origen_proc(mod=user_configs)
        assert retn["users__datasets"] == ["test"]
        assert retn["users__hierarchy"] == ["test"]
        assert retn["datasets"] == ["test"]
        assert retn["hierarchy"] == ["test"]
        assert retn["first_name_unset"] == None
        assert retn["first_name_dataset_unset"] == None
        assert retn["first_name"] == "Corey"
        assert retn["first_name_dataset"] == "Corey"

    def test_single_dataset_and_explicit_hierarchy(self):
        ''' Very close to what is already tested, but simple enough to do'''
        retn = in_new_origen_proc(mod=user_configs)
        assert retn["users__datasets"] == ["test"]
        assert retn["users__hierarchy"] == ["test"]
        assert retn["datasets"] == ["test"]
        assert retn["hierarchy"] == ["test"]
        assert retn["first_name_unset"] == None
        assert retn["first_name_dataset_unset"] == None
        assert retn["first_name"] == "Corey2"
        assert retn["first_name_dataset"] == "Corey2"

    def test_single_dataset_and_empty_hierarchy(self):
        retn = in_new_origen_proc(mod=user_configs)
        assert retn["users__datasets"] == ["test"]
        assert retn["users__hierarchy"] == []
        assert retn["datasets"] == ["test"]
        assert retn["hierarchy"] == []
        assert isinstance(retn["first_name_unset"], RuntimeError)
        assert str(
            retn["first_name_unset"]
        ) == "Dataset hierarchy is empty! Data lookups must explicitly name the dataset to query"
        assert retn["first_name_dataset_unset"] == None
        assert retn["first_name_unset_2"] == None

    def test_multi_datasets_and_default_hierarchy(self):
        ''' Default hierarchy for multiple datasets is empty '''
        retn = in_new_origen_proc(mod=user_configs)
        assert set(retn["datasets"]) == set(["test_1st", "test_2nd"])
        assert retn["hierarchy"] == []
        assert retn["hierarchy_2"] == ["test_1st", "test_2nd"]

    def test_default_dataset_and_hierarchy(self, ddk):
        ''' No datasets given, nor any hierarchy. Absolute base case '''
        retn = in_new_origen_proc(mod=user_configs)
        assert retn["users__datasets"] == [ddk]
        assert retn["users__hierarchy"] == [ddk]
        assert retn["datasets"] == [ddk]
        assert retn["hierarchy"] == [ddk]
        assert retn["first_name_unset"] == None
        assert retn["first_name_dataset_unset"] == None
        assert retn["first_name"] == "COREY"
        assert retn["first_name_dataset"] == "COREY"

    def test_empty_hierarchy_and_default_dataset(self, capfd, ddk):
        retn = in_new_origen_proc(mod=user_configs)
        assert retn["users__datasets"] == [ddk]
        assert retn["datasets"] == [ddk]
        assert retn["users__hierarchy"] == []
        assert retn["hierarchy"] == []

    def test_empty_datasets(self, ddk):
        retn = in_new_origen_proc(mod=user_configs)
        assert retn["users__datasets"] == [ddk]
        assert retn["datasets"] == [ddk]
        assert retn["users__data_lookup_hierarchy"] == [ddk]
        assert retn["hierarchy"] == [ddk]
