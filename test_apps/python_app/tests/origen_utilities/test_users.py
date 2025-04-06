import origen, pytest
from tests.shared import in_new_origen_proc
from configs import users as user_configs
from tests import om_shared

with om_shared():
    from om_tests.framework.users.shared import Base as UsersBase  # type:ignore
    from om_tests.utils.test_ldap import Common as LdapCommon  # type:ignore

ldap_config = LdapCommon.get_dummy_config()

class TestUsers(UsersBase):
    def test_users_are_accessible(self):
        assert isinstance(origen.users, self.users_class)

    def test_current_and_initial_user_are_set_upon_boot(self):
        assert len(origen.users) == 2
        assert origen.users.current_user.id == self.logged_in_id
        assert origen.users.initial_user.id == self.logged_in_id
        assert isinstance(origen.current_user, self.user_class)
        assert origen.current_user.id == self.logged_in_id
        assert isinstance(origen.initial_user, self.user_class)
        assert origen.initial_user.id == self.logged_in_id

    def test_default_users_are_loaded_upon_boot(self):
        assert len(origen.users) == 2
        n = "dummy_ldap_read_only"
        assert n in origen.users

        u = origen.users[n]
        assert u.username == ldap_config.auth_username
        assert u.password == ldap_config.auth_password

    def test_datasets_are_set_upon_boot(self, def_dsc_dict):
        dsets = {"test", "test2", "backup", "dummy_ldap_ds", "git"}
        assert set(origen.users.datasets.keys()) == dsets
        assert set(origen.current_user.datasets.keys()) == dsets
        assert dict(origen.users.datasets["test"]) == def_dsc_dict
        assert dict(origen.users.datasets["test2"]) == def_dsc_dict
        assert dict(origen.users.datasets["backup"]) == def_dsc_dict
        assert dict(
            origen.users.datasets["dummy_ldap_ds"]) == self.def_dsc_dict_with(
                category="ldaps",
                data_store="dummy_ldap",
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

    # TODO decouple this from ldap
    @pytest.mark.ldap
    def test_autopopulated_user(self):
        in_new_origen_proc(mod=user_configs, options={"user": ldap_config.users[0].fields})

    def test_suppress_initializing_current_user(self):
        retn = in_new_origen_proc(mod=user_configs)
        assert retn["user_ids"] == set()
        assert retn["current_user"] is None
        assert retn["initial_user"] is None

    # TODO decouple this from ldap
    @pytest.mark.ldap
    def test_loading_default_users(self):
        ldap_u1 = ldap_config.users[0]
        ldap_u2 = ldap_config.users[1]

        retn = in_new_origen_proc(mod=user_configs)
        assert retn["user_ids"] == {"basic", "full user", "gauss", "euler"}
        u = retn["basic"]
        assert u["dataset_names"] == {'autopop_ldap', 'other'}
        assert u["username"] == "basic"
        assert u["password"] == False
        assert u["email"] == None
        assert u["first_name"] == None
        assert u["last_name"] == None
        assert u["__auto_populate__"] == False
        assert u["__should_validate_passwords__"] == None
        # TODO
        # roles

        u = retn["full user"]
        assert u["dataset_names"] == {'autopop_ldap', 'other'}
        assert u["username"] == "test full user"
        assert u["password"] == "password!"
        assert u["email"] == "full.user@origen.org"
        assert u["first_name"] == "TEST"
        assert u["last_name"] == "USER"
        assert u["__auto_populate__"] == False
        assert u["__should_validate_passwords__"] == False

        u = retn[ldap_u1.id]
        assert u["dataset_names"] == {'autopop_ldap', 'other'}
        assert u["username"] == ldap_u1.id
        assert u["password"] == False
        assert u["email"] == ldap_u1.mail
        assert u["first_name"] == None
        assert u["last_name"] == ldap_u1.sn
        assert u["__auto_populate__"] == True
        assert u["__should_validate_passwords__"] == False

        u = retn[ldap_u2.id]
        assert u["dataset_names"] == {'autopop_ldap', 'other'}
        assert u["username"] == ldap_u2.id
        assert u["password"] == "pw_guest2"
        assert u["email"] == "guest2@origen.org"
        assert u["first_name"] == None
        assert u["last_name"] == ldap_u2.sn
        assert u["__auto_populate__"] == True
        assert u["__should_validate_passwords__"] == False

class TestUserConfigSetups(UsersBase):
    @pytest.fixture
    def dsets(self):
        return set(['test', 'test2', 'git', 'dummy_ldap_ds', 'backup'])

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
        assert "Encountered an error when initializing users: Error encountered setting the default lookup hierarchy" in stdout
        assert "The following datasets do not exists and cannot be used in the data lookup hierarchy: 'blah'" in stdout
        assert "Forcing empty dataset lookup hierarchy..." in stdout

    def test_error_on_duplicate_datasets_in_hierarchy(self, capfd, dsets):
        retn = in_new_origen_proc(mod=user_configs)
        assert retn["users__data_lookup_hierarchy"] == []
        assert retn["data_lookup_hierarchy"] == []

        assert set(retn["users__datasets"]) == dsets
        assert set(retn["datasets"]) == dsets

        stdout = capfd.readouterr().out
        assert "Encountered an error when initializing users: Error encountered setting the default lookup hierarchy" in stdout
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
        assert "Encountered an error when initializing users: Error encountered setting the default lookup hierarchy" in stdout
        assert "RuntimeError: The following datasets do not exists and cannot be used in the data lookup hierarchy: 'test'" in stdout
        assert "Forcing empty dataset lookup hierarchy..." in stdout

    # TODO decouple this from ldap
    @pytest.mark.ldap
    def test_error_adding_default_users(q, capfd):
        retn = in_new_origen_proc(mod=user_configs)
        assert retn["user_ids"] == {"u1", "u2", "u3"}
        assert retn["u1"]["username"] == "u1"
        assert retn["u2"]["username"] == "u2"
        assert retn["u3"]["username"] == "user 3"

        stdout = capfd.readouterr().out
        assert "Encountered an error when initializing users: Failed to initialize default user 'u2'" in stdout
        assert "RuntimeError: Encountered Exception 'RuntimeError' with message: Cannot find mapped value 'cn' in LDAP 'dummy_autopop'" in stdout

    def test_error_setting_default_user_fields(q, capfd):
        retn = in_new_origen_proc(mod=user_configs)
        assert retn["user_ids"] == {origen.current_user.id, 'test'}

        stdout = capfd.readouterr().out
        assert "Encountered an error when initializing users: Failed to initialize default user 'test'" in stdout
        assert "Failed to set field 'username'" in stdout
        assert "RuntimeError: Data lookup hierarchy for user 'test' is empty" in stdout
        assert "Bailing on initializing default user 'test'" in stdout

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
