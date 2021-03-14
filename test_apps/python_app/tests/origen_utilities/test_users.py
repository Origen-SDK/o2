import origen, _origen, pytest, getpass
from tests.shared.python_like_apis import Fixture_DictLikeAPI
from tests.shared import in_new_origen_proc
from tests.origen_utilities.test_ldap import USER_USERNAME, PASSWORD
from configs import users as user_configs

data_fields = ["email", "first_name", "last_name"]
data_fields_exceptions = ["display_name", "username"]


class TestUsers:
    @property
    def current(self):
        return getpass.getuser()

    @property
    def user(self):
        return origen.users["test"]

    @property
    def user2(self):
        return origen.users["test2"]

    @property
    def user3(self):
        return origen.users["test3"]

    # @property
    # def home_dir(self):
    #     if ENV['GITHUB_ACTIONS']:
    #         return ...
    #     else:
    #         return "?"

    def test_current_user(self):
        user = origen.current_user()
        assert isinstance(user, _origen.users.User)
        assert user.id == self.current

    def test_adding_users(self):
        # Current user is always added
        assert len(origen.users) == 1
        assert "test" not in origen.users

        origen.users.add("test")
        assert "test" in origen.users
        assert len(origen.users) == 2
        assert isinstance(origen.users["test"], _origen.users.User)

        origen.users.add("test2")
        assert "test2" in origen.users
        assert len(origen.users) == 3

        origen.users.add("test3")
        assert "test3" in origen.users
        assert len(origen.users) == 4

    def test_accessing_datasets(self):
        u = self.user
        assert isinstance(u.datasets, dict)
        assert "test" in u.datasets
        assert "test2" in u.datasets
        assert "forumsys" in u.datasets
        assert set(u.datasets.keys()) == {
            "test", "test2", "forumsys", "backup", "git"
        }

    def test_accessing_data_lookup_hierarchy(self):
        assert self.user.data_lookup_hierarchy == ["test", "backup"]
        assert self.user.top_datakey == "test"

    def test_accessing_username(self):
        u = origen.users["test"]
        d = u.datasets["test"]

        assert u.username == "test"
        assert d.username is None
        u.username = "test_username"
        assert u.username == "test_username"
        assert d.username == "test_username"
        setattr(u, "username", "test2")
        assert u.username == "test2"
        d.username = "test3"
        assert d.username == "test3"
        assert u.username == "test3"
        assert u.id == "test"

    def test_confirm_data_fields(self):
        assert origen.users.DATA_FIELDS == data_fields + data_fields_exceptions

    # Test setting various data pieces
    @pytest.mark.parametrize("field", data_fields)
    def test_accessing_fields(self, field):
        data = f"test_{field}"
        u = self.user
        d = u.datasets["test"]
        assert getattr(u, field) is None
        assert getattr(d, field) is None
        setattr(u, field, data)
        assert getattr(u, field) == data
        assert getattr(d, field) == data
        data2 = f"{data}_via_dataset"
        setattr(d, field, data2) is None
        assert getattr(u, field) == data2
        assert getattr(d, field) == data2
        assert getattr(u.datasets["test2"], field) is None

    def test_accessing_display_name(self):
        u = self.user2
        d = self.user2.datasets["test"]
        d2 = self.user2.datasets["test2"]

        assert u.id == "test2"
        assert u.first_name is None
        assert u.last_name is None
        assert d.__display_name__ is None
        assert d.username is None

        # If nothing is set, ID is used
        assert u.display_name == "test2"
        assert d.display_name == "test2"
        assert d.__display_name__ is None
        assert d2.display_name == "test2"

        # If no first name & last name, or no display name, but a username, username is used
        u.username = "display_username"
        assert u.display_name == "display_username"
        assert d.display_name == "display_username"
        assert d.__display_name__ is None
        assert d2.display_name == "test2"

        # If a first name but no last name is given, the username is still used
        u.first_name = "User"
        assert u.display_name == "display_username"
        assert d.display_name == "display_username"
        assert d.__display_name__ is None
        assert d2.display_name == "test2"

        # Vice-versa is also true
        u.first_name = None
        u.last_name = "Test"
        assert u.display_name == "display_username"
        assert d.display_name == "display_username"
        assert d.__display_name__ is None
        assert d2.display_name == "test2"

        # But, if both first and last names are available, that is used.
        u.first_name = "User"
        u.last_name = "Test"
        assert u.display_name == "User Test"
        assert d.__display_name__ is None
        assert d.display_name == "User Test"
        assert d2.display_name == "test2"

        # Finally, any given display name is used
        u.display_name = "user_display_name"
        assert u.display_name == "user_display_name"
        assert d.__display_name__ == "user_display_name"
        assert d2.display_name == "test2"

        # Display name can also be set from the dataset
        d.display_name = "dataset_display_name"
        assert u.display_name == "dataset_display_name"
        assert d.display_name == "dataset_display_name"
        assert d.__display_name__ == "dataset_display_name"
        assert d2.display_name == "test2"

    def test_getting_and_setting_arbitrary_data(self):
        u = self.user
        d = u.datasets["test"]
        d2 = u.datasets["test2"]

        assert len(u.data_store) == 0
        assert "test" not in u.data_store
        u.data_store["test"] = 1
        assert u.data_store["test"] == 1
        assert d.data_store["test"] == 1
        assert "test" not in d2.data_store

    def test_lookup_hierarch(self):
        u = self.user3
        d1 = u.datasets["test"]
        d2 = u.datasets["backup"]
        assert d1.first_name is None
        assert d2.first_name is None
        assert u.first_name is None

        n = "backup"
        d2.first_name = n
        assert d1.first_name is None
        assert d2.first_name == n
        assert u.first_name == n

    def test_customizing_data_lookup_hierarchy(self):
        u = self.user3
        test_n = "test"
        backup_n = "backup"
        u.datasets["test"].first_name = test_n
        u.datasets["backup"].first_name = backup_n

        assert u.data_lookup_hierarchy == ["test", "backup"]
        assert u.first_name == test_n

        # Swap the lookup hierarchy
        u.data_lookup_hierarchy = list(reversed(u.data_lookup_hierarchy))
        assert u.data_lookup_hierarchy == ["backup", "test"]
        assert u.first_name == backup_n

        # Check that the hierarchy lookup is still functioning
        u.datasets["backup"].first_name = None
        assert u.first_name == test_n

        # Messing about with the datakey hierarchy is a per-user ordeal
        assert self.user.data_lookup_hierarchy == ["test", "backup"]
        assert self.user2.data_lookup_hierarchy == ["test", "backup"]

    def test_data_lookup_hierarchy_is_returned_by_value(self):
        u = self.user3
        assert u.data_lookup_hierarchy == ["backup", "test"]
        u.data_lookup_hierarchy.append(
            "hi"
        )  # should be no error here as the update attempt never makes it to the backend
        assert u.data_lookup_hierarchy == ["backup", "test"]

    def test_error_on_unknown_dataset_when_setting_hierarchies(self):
        u = self.user3
        assert u.data_lookup_hierarchy == ["backup", "test"]
        with pytest.raises(
                OSError,
                match=
                "'hi' is not a valid dataset and cannot be used in the dataset hierarchy"
        ):
            u.data_lookup_hierarchy = ["test", "backup", "hi"]
        assert u.data_lookup_hierarchy == ["backup", "test"]

    def test_error_on_duplicate_datasets_when_setting_hierarchies(self):
        u = self.user3
        assert u.data_lookup_hierarchy == ["backup", "test"]
        with pytest.raises(
                OSError,
                match=
                r"dataset 'test' can only appear once in the dataset hierarchy \(first appearance at index 0 - duplicate at index 2\)"
        ):
            u.data_lookup_hierarchy = ["test", "backup", "test"]
        assert u.data_lookup_hierarchy == ["backup", "test"]

    def test_empty_data_lookup_hierarchy(self):
        u = self.user3
        assert u.datasets["backup"].first_name == None
        assert u.datasets["test"].first_name == "test"

        u.data_lookup_hierarchy = []
        assert u.data_lookup_hierarchy == []

        with pytest.raises(
                OSError,
                match=
                "Dataset hierarchy is empty! Data lookups must explicitly name the dataset to query"
        ):
            u.first_name
        with pytest.raises(
                OSError,
                match="Data lookup hierarchy for user 'test3' is empty"):
            u.top_datakey
        assert u.datasets["backup"].first_name == None
        assert u.datasets["test"].first_name == "test"

    class TestDataStoreDictLike(Fixture_DictLikeAPI):
        def parameterize(self):
            return {
                "keys": ["test", "test 1", "test 2"],
                "klass": str,
                "not_in_dut": "Blah"
            }

        def boot_dict_under_test(self):
            if not getattr(self, "booted_up", False):
                origen.current_user().data_store["test"] = "zero"
                origen.current_user().data_store["test 1"] = "one"
                origen.current_user().data_store["test 2"] = "two"
                self.booted_up = True
            return origen.current_user().data_store

    def test_passwords(self):
        u = self.user
        d = u.datasets["test"]
        d2 = u.datasets["test2"]

        u.password = "blah!"
        d2.password = "!PASSWORD!"
        assert u.password == "blah!"
        assert d.password == "blah!"
        assert d2.password == "!PASSWORD!"

        d.password = "PASSWORD"
        assert u.password == "PASSWORD"
        assert d.password == "PASSWORD"
        assert d2.password == "!PASSWORD!"

    def test_password_reasons(self):
        u = self.user
        d = u.datasets["test"]
        d2 = u.datasets["test2"]

        d.password = "PASSWORD"
        d2.password = "!PASSWORD!"

        assert u.password_for("just because") == "!PASSWORD!"
        with pytest.raises(
                OSError,
                match=f"No password available for reason: 'Nothing!'"):
            u.password_for("Nothing!")
        assert u.dataset_for("just because") == "test2"
        assert u.dataset_for("nothing") == None
        assert u.password_for("Nothing!", default=None) == "PASSWORD"
        assert u.password_for("Nothing!", default="test2") == "!PASSWORD!"

    def test_populated_dataset(self):
        # By the config, dataset is not populated automoatically
        u = self.user
        d = u.datasets["forumsys"]
        assert d.populated == False

        # Set the username before populating
        d.username = "euler"

        # Populate
        d.populate()
        assert d.populated == True

        # Check some items
        assert d.email == "euler@ldap.forumsys.com"
        assert d.last_name == "Euler"
        assert d.username == "euler"

        # Check that other data fields were populated
        assert d.data_store["full_name"] == "Leonhard Euler"

class TestUsersDictLike(Fixture_DictLikeAPI):
    def parameterize(self):
        return {
            "keys": [getpass.getuser(), "test", "test2", "test3"],
            "klass": _origen.users.User,
            "not_in_dut": "Blah"
        }

    def boot_dict_under_test(self):
        return origen.users

class TestConfigSetups:

    def test_error_on_invalid_datasets_in_hierarchy(self, capfd):
        retn = in_new_origen_proc(mod=user_configs)
        assert retn["data_lookup_hierarchy"] == []
        assert set(retn["datasets"]) == set(['test', 'test2', 'git', 'forumsys', 'backup'])
        stdout = capfd.readouterr().out
        assert "'blah' is not a valid dataset and cannot be used in the dataset hierarchy" in stdout
        assert "Forcing empty dataset lookup hierarchy..." in stdout
        assert f"Data lookup hierarchy for user '{getpass.getuser()}' is empty" in stdout

    def test_error_on_duplicate_datasets_in_hierarchy(self, capfd):
        retn = in_new_origen_proc(mod=user_configs)
        assert retn["data_lookup_hierarchy"] == []
        assert set(retn["datasets"]) == set(['test', 'test2', 'git', 'forumsys', 'backup'])
        stdout = capfd.readouterr().out
        assert "dataset 'test' can only appear once in the dataset hierarchy (first appearance at index 0 - duplicate at index 2)" in stdout
        assert "Forcing empty dataset lookup hierarchy..." in stdout
        assert f"Data lookup hierarchy for user '{getpass.getuser()}' is empty" in stdout

    def test_single_dataset_and_default_hierarchy(self):
        ''' Default hierarchy should be the only dataset '''
        retn = in_new_origen_proc(mod=user_configs)
        assert retn["datasets"] == ["test"]
        assert retn["hierarchy"] == ["test"]
        assert retn["first_name_unset"] == None
        assert retn["first_name_dataset_unset"] == None
        assert retn["first_name"] == "Corey"
        assert retn["first_name_dataset"] == "Corey"

    def test_single_dataset_and_explicit_hierarchy(self):
        ''' Very close to what is already tested, but simple enough to do'''
        retn = in_new_origen_proc(mod=user_configs)
        assert retn["datasets"] == ["test"]
        assert retn["hierarchy"] == ["test"]
        assert retn["first_name_unset"] == None
        assert retn["first_name_dataset_unset"] == None
        assert retn["first_name"] == "Corey2"
        assert retn["first_name_dataset"] == "Corey2"

    def test_single_dataset_and_empty_hierarchy(self):
        retn = in_new_origen_proc(mod=user_configs)
        assert retn["datasets"] == ["test"]
        assert retn["hierarchy"] == []
        assert isinstance(retn["first_name_unset"], OSError)
        assert str(retn["first_name_unset"]) == "Dataset hierarchy is empty! Data lookups must explicitly name the dataset to query"
        assert retn["first_name_dataset_unset"] == None
        assert retn["first_name_unset_2"] == None

    def test_multi_datasets_and_default_hierarchy(self):
        ''' Default hierarchy for multiple datasets is empty '''
        retn = in_new_origen_proc(mod=user_configs)
        assert set(retn["datasets"]) == set(["test_1st", "test_2nd"])
        assert retn["hierarchy"] == []
        assert retn["hierarchy_2"] == ["test_1st", "test_2nd"]

    def test_default_dataset_and_hierarchy(self):
        ''' No datasets given, nor any hierarchy. Absolute base case '''
        retn = in_new_origen_proc(mod=user_configs)
        assert retn["datasets"] == ["__origen__default__"]
        assert retn["hierarchy"] == ["__origen__default__"]
        assert retn["first_name_unset"] == None
        assert retn["first_name_dataset_unset"] == None
        assert retn["first_name"] == "COREY"
        assert retn["first_name_dataset"] == "COREY"

    def test_error_message_on_default_data_and_given_hierarchy(self, capfd):
        ''' As no datasets were given, giving a hierarchy is not allowed '''
        retn = in_new_origen_proc(mod=user_configs)
        assert retn["datasets"] == ["__origen__default__"]
        assert retn["hierarchy"] == ["__origen__default__"]
        stdout = capfd.readouterr().out
        assert "Providing config value 'user__data_lookup_hierarchy' without providing 'user__datasets' is not allowed" in stdout
        assert "Forcing default dataset..." in stdout

    def test_error_message_on_default_data_and_empty_hierarchy(self, capfd):
        ''' Same as the above - cannot give an empty hierarchy either '''
        retn = in_new_origen_proc(mod=user_configs)
        assert retn["datasets"] == ["__origen__default__"]
        assert retn["hierarchy"] == ["__origen__default__"]
        stdout = capfd.readouterr().out
        assert "Providing config value 'user__data_lookup_hierarchy' without providing 'user__datasets' is not allowed" in stdout
        assert "Forcing default dataset..." in stdout

    def test_empty_datasets(self):
        retn = in_new_origen_proc(mod=user_configs)
        assert retn["datasets"] == ["__origen__default__"]
        assert retn["hierarchy"] == ["__origen__default__"]

#     def test_home_dir():
#         ...

#     def test_session():
#         ...
