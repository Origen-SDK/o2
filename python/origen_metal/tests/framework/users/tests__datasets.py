import pytest
import origen_metal as om
from origen_metal.framework.users import UserDatasetConfig
from .shared import Base as SharedBase
from tests.shared.python_like_apis import Fixture_DictLikeAPI


class Base(SharedBase):
    @pytest.fixture
    def tdk2(self, tdk):
        return f"{tdk}_2"

    @pytest.fixture
    def d(self, u, ddk):
        return u.datasets[ddk]

    @pytest.fixture
    def d2_name(self):
        return "test_dset_2"

    @pytest.fixture
    def d2(self, u, d2_name):
        if d2_name not in u.datasets:
            u.add_dataset(d2_name, as_topmost=False)
        return u.datasets[d2_name]


class T_Datasets(Base):
    def test_default_dataset_and_hierarchy(self, unload_users, users, u, ddk,
                                           def_dsc, def_dsc_dict):
        ''' No datasets given, nor any hierarchy. Absolute base case '''
        assert users.datakeys == [ddk]
        assert users.data_lookup_hierarchy == [ddk]
        assert isinstance(users.datasets[ddk], self.user_dataset_config_class)
        assert dict(users.datasets[ddk]) == def_dsc_dict
        assert users.datasets[ddk] == def_dsc

        assert isinstance(u.datasets[ddk], self.user_dataset_class)
        assert isinstance(u.datasets, dict)
        assert list(u.datasets.keys()) == [ddk]
        assert dict(u.datasets[ddk].config) == def_dsc_dict
        assert u.data_lookup_hierarchy == [ddk]
        assert u.top_datakey == ddk

    def test_confirm_data_fields(self, users):
        # TODO move this into test module
        assert users.DATA_FIELDS == self.DATA_FIELDS + self.DATA_FIELD_EXCEPTIONS

    # Test setting various data pieces
    @pytest.mark.parametrize("field", Base.DATA_FIELDS)
    def test_accessing_fields(self, unload_users, u, ddk, field):
        data = f"test_{field}"
        d = u.datasets[ddk]
        assert getattr(u, field) is None
        assert getattr(d, field) is None
        setattr(u, field, data)
        assert getattr(u, field) == data
        assert getattr(d, field) == data
        data2 = f"{data}_via_dataset"
        setattr(d, field, data2) is None
        assert getattr(u, field) == data2
        assert getattr(d, field) == data2

    def test_adding_datasets_before_user_creation(self, unload_users, users,
                                                  ddk, tdk, def_dsc_dict):
        assert tdk not in users.datasets
        assert users.add_dataset(tdk) is None

        exp = [tdk, ddk]
        assert list(users.datasets.keys()) == list(reversed(exp))
        assert list(users.datakeys) == list(reversed(exp))
        assert dict(users.datasets[tdk]) == def_dsc_dict
        assert users.data_lookup_hierarchy == exp

        u = self.user()
        assert list(u.datasets.keys()) == list(reversed(exp))
        assert u.data_lookup_hierarchy == exp
        assert dict(u.datasets[tdk].config) == def_dsc_dict

    def test_adding_datasets_after_user_creation(self, unload_users, users,
                                                 ddk, tdk):
        assert list(users.datakeys) == [ddk]
        assert users.data_lookup_hierarchy == [ddk]

        u = self.user()
        assert list(u.datasets.keys()) == [ddk]
        assert u.data_lookup_hierarchy == [ddk]

        exp = [tdk, ddk]
        rev_exp = list(reversed(exp))
        assert users.add_dataset(tdk) is None
        assert list(users.datakeys) == rev_exp
        assert users.data_lookup_hierarchy == exp

        u2 = self.user(2)
        assert list(u2.datasets.keys()) == rev_exp
        assert u2.data_lookup_hierarchy == exp
        assert u2.top_datakey == tdk

        assert list(u.datasets.keys()) == [ddk]
        assert u.data_lookup_hierarchy == [ddk]

    def test_registering_datasets(self, unload_users, users, ddk, tdk):
        ''' Adds a new dataset, but does not add it into the hierarchy'''
        assert list(users.datakeys) == [ddk]
        assert users.data_lookup_hierarchy == [ddk]

        assert users.register_dataset(tdk) is None
        assert list(users.datakeys) == [ddk, tdk]
        assert users.data_lookup_hierarchy == [ddk]

        u = self.user()
        assert list(u.datasets.keys()) == [ddk, tdk]
        assert u.data_lookup_hierarchy == [ddk]

    def test_adding_datasets_at_end_of_hierarchy(self, unload_users, users,
                                                 ddk, tdk):
        ''' Adds a new dataset as the lowest priority'''
        assert list(users.datakeys) == [ddk]
        assert users.data_lookup_hierarchy == [ddk]

        exp = [ddk, tdk]
        assert users.add_dataset(tdk, as_topmost=False) is None
        assert list(users.datakeys) == exp
        assert users.data_lookup_hierarchy == exp

        u = self.user()
        assert list(u.datasets.keys()) == exp
        assert u.data_lookup_hierarchy == exp
        assert u.top_datakey == ddk

    def test_adding_default_datasets_with_config(self, unload_users, users,
                                                 tdk, cat_name, dstore_name):
        assert tdk not in users.datasets
        assert users.add_dataset(tdk, self.dummy_dsc) is None

        exp = self.dummy_dsc_dict
        assert dict(users.datasets[tdk]) == exp

        u = self.user()
        assert dict(u.datasets[tdk].config) == exp

    def test_error_on_adding_duplicate_datasets(self, users, tdk, dstore_name,
                                                cat_name):
        exp = self.dummy_dsc_dict

        with pytest.raises(RuntimeError,
                           match=f"A dataset '{tdk}' is already present"):
            users.add_dataset(tdk)
        assert dict(users.datasets[tdk]) == exp

        with pytest.raises(RuntimeError,
                           match=f"A dataset '{tdk}' is already present"):
            users.register_dataset(tdk)
        assert dict(users.datasets[tdk]) == exp

    def test_adding_dataset_individually(self, unload_users, users, ddk, tdk,
                                         tdk2, def_dsc):
        assert list(users.datasets.keys()) == [ddk]
        assert users.data_lookup_hierarchy == [ddk]

        # Create two users with default setup
        u = self.user()
        assert list(u.datasets.keys()) == [ddk]
        assert u.data_lookup_hierarchy == [ddk]
        u2 = self.user(2)
        assert list(u2.datasets.keys()) == [ddk]
        assert u2.data_lookup_hierarchy == [ddk]

        # Add a dataset to first user
        ds = u.add_dataset(tdk)
        assert list(u.datasets.keys()) == [ddk, tdk]
        assert u.data_lookup_hierarchy == [tdk, ddk]
        assert ds.config == def_dsc

        # Defaults should remain the same
        assert list(users.datasets.keys()) == [ddk]
        assert users.data_lookup_hierarchy == [ddk]
        assert list(u2.datasets.keys()) == [ddk]
        assert u2.data_lookup_hierarchy == [ddk]

        # Add the same dataset name to u2, but with options
        # u vs. u2 should have different values and it shouldn't be added as default
        u2.add_dataset(tdk, self.dummy_dsc)
        assert list(u2.datasets.keys()) == [ddk, tdk]
        assert u2.data_lookup_hierarchy == [tdk, ddk]
        assert dict(u2.datasets[tdk].config) == self.dummy_dsc_dict

        # Add a second dataset to u as 'least-most'
        ds = u.add_dataset(tdk2,
                           self.dummy_dsc_with(data_store="s1"),
                           as_topmost=False)
        assert list(u.datasets.keys()) == [ddk, tdk, tdk2]
        assert u.data_lookup_hierarchy == [tdk, ddk, tdk2]
        assert dict(ds.config) == self.dummy_dsc_dict_with(data_store="s1")

        # Defaults with the same name can still be updated
        users.add_dataset(tdk, UserDatasetConfig(data_store="s2"))
        assert list(users.datasets.keys()) == [ddk, tdk]
        assert users.data_lookup_hierarchy == [tdk, ddk]
        assert dict(
            users.datasets[tdk]) == self.def_dsc_dict_with(data_store="s2")

    def test_registering_datasets_individually(self, u, ddk, tdk, tdk2):
        assert list(u.datasets.keys()) == [ddk, tdk, tdk2]
        assert u.data_lookup_hierarchy == [tdk, ddk, tdk2]

        ds = u.register_dataset("r1", UserDatasetConfig("Registered", "r1"))
        assert list(u.datasets.keys()) == [ddk, tdk, tdk2, "r1"]
        assert u.data_lookup_hierarchy == [tdk, ddk, tdk2]
        assert dict(ds.config) == self.def_dsc_dict_with(category="Registered",
                                                         data_store="r1")

    def test_adding_duplicate_datasets_to_user(self, u, ddk, tdk2, tdk):
        assert list(u.datasets.keys()) == [ddk, tdk, tdk2, "r1"]
        assert u.data_lookup_hierarchy == [tdk, ddk, tdk2]
        assert u.datasets[
            tdk2].config.data_store != "add_with_replace_existing_test"
        assert u.datasets[
            "r1"].config.data_store != "register_with_replace_existing_test"
        assert u.datasets[
            tdk].config.data_store != "test_dk_replace_existing_test"

        # Adding a duplicate dataset should fail
        with pytest.raises(RuntimeError,
                           match=f"User '{u.id}' already has dataset '{tdk2}"):
            u.add_dataset(tdk2)

        # Unless "replace_existing" is used
        u.add_dataset(
            tdk2,
            UserDatasetConfig(data_store="add_with_replace_existing_test"),
            replace_existing=True)
        assert u.datasets[
            tdk2].config.data_store == "add_with_replace_existing_test"

        # Hierarchy and dataset order are updated appropriately
        assert list(u.datasets.keys()) == [ddk, tdk, "r1", tdk2]
        assert u.data_lookup_hierarchy == [tdk2, tdk, ddk]

        # Same is true with registering datasets
        with pytest.raises(RuntimeError,
                           match=f"User '{u.id}' already has dataset 'r1"):
            u.register_dataset("r1")

        u.register_dataset(
            "r1",
            UserDatasetConfig(
                data_store="register_with_replace_existing_test"),
            replace_existing=True)
        assert list(u.datasets.keys()) == [ddk, tdk, tdk2, "r1"]
        assert u.data_lookup_hierarchy == [tdk2, tdk, ddk]
        assert u.datasets[
            "r1"].config.data_store == "register_with_replace_existing_test"

        # Corner case: Registering with replacement will update the dataset, but not impact the hierarchy
        u.register_dataset(
            tdk,
            UserDatasetConfig(data_store="test_dk_replace_existing_test"),
            replace_existing=True)
        assert list(u.datasets.keys()) == [ddk, tdk2, "r1", tdk]
        assert u.data_lookup_hierarchy == [tdk2, tdk, ddk]
        assert u.datasets[
            tdk].config.data_store == "test_dk_replace_existing_test"

    def test_datasets_are_independent(self, u, u2, ddk, tdk):
        ds1 = u.datasets[ddk]
        ds2 = u.datasets[tdk]
        ds3 = u2.datasets[ddk]
        assert ds1.email is None
        assert ds2.email is None
        assert ds3.email is None

        ds1.email = "ds1@origen_metal.com"
        assert ds1.email == "ds1@origen_metal.com"
        assert ds2.email is None
        assert ds3.email is None

    def test_updating_default_dataset(self, unload_users, users, ddk, tdk,
                                      def_dsc_dict):
        assert users.datakeys == [ddk]
        assert users.data_lookup_hierarchy == [ddk]

        users.override_default_dataset(tdk)
        assert users.datakeys == [tdk]
        assert users.data_lookup_hierarchy == [tdk]
        assert dict(users.datasets[tdk]) == def_dsc_dict

        u = self.user()
        assert u.datakeys == [tdk]
        assert u.data_lookup_hierarchy == [tdk]

    def test_updating_default_dataset_with_config(self, unload_users, users,
                                                  ddk, tdk):
        assert list(users.datasets.keys()) == [ddk]
        assert users.data_lookup_hierarchy == [ddk]

        users.override_default_dataset(tdk, UserDatasetConfig(None, "ds0"))
        assert users.datakeys == [tdk]
        assert users.data_lookup_hierarchy == [tdk]
        assert dict(
            users.datasets[tdk]) == self.def_dsc_dict_with(data_store="ds0")

        u = self.user()
        assert u.datakeys == [tdk]
        assert u.data_lookup_hierarchy == [tdk]
        assert dict(
            u.datasets[tdk].config) == self.def_dsc_dict_with(data_store="ds0")

    def test_updating_default_dataset_multiple_times(self, unload_users, users,
                                                     ddk, tdk):
        assert list(users.datasets.keys()) == [ddk]
        assert users.data_lookup_hierarchy == [ddk]

        users.override_default_dataset(tdk, UserDatasetConfig(None, "ds0"))
        assert users.datakeys == [tdk]
        assert users.data_lookup_hierarchy == [tdk]
        assert dict(
            users.datasets[tdk]) == self.def_dsc_dict_with(data_store="ds0")

        users.override_default_dataset("override",
                                       UserDatasetConfig("c0", "ds1"))
        assert users.datakeys == ["override"]
        assert users.data_lookup_hierarchy == ["override"]
        assert dict(users.datasets["override"]) == self.def_dsc_dict_with(
            category="c0", data_store="ds1")

        u = self.user()
        assert u.datakeys == ["override"]
        assert u.data_lookup_hierarchy == ["override"]
        assert dict(u.datasets["override"].config) == self.def_dsc_dict_with(
            category="c0", data_store="ds1")

    def test_error_on_updating_defaults_after_user_creation(
            self, unload_users, users, u, u2, ddk):
        assert list(users.datasets.keys()) == [ddk]
        assert users.data_lookup_hierarchy == [ddk]

        with pytest.raises(
                RuntimeError,
                match=
                f"The default dataset can only be overridden prior to adding any users. Found users: '{u.id}', '{u2.id}'"
        ):
            users.override_default_dataset("ds_override")

        assert list(users.datasets.keys()) == [ddk]
        assert users.data_lookup_hierarchy == [ddk]

    def test_error_on_updating_defaults_after_additional_datasets(
            self, unload_users, users, ddk):
        users.add_dataset("ds1")
        users.add_dataset("ds2")
        assert list(users.datasets.keys()) == [ddk, 'ds1', 'ds2']
        assert users.data_lookup_hierarchy == ['ds2', 'ds1', ddk]

        with pytest.raises(
                RuntimeError,
                match=
                "The default dataset can only be overridden prior to adding any additional datasets. Found additional datasets: 'ds1', 'ds2'"
        ):
            users.override_default_dataset("ds_override")

        assert list(users.datasets.keys()) == [ddk, 'ds1', 'ds2']
        assert users.data_lookup_hierarchy == ['ds2', 'ds1', ddk]

    def test_accessing_username(self, unload_users, u_id, u, d):
        assert u.username == u_id
        assert d.username is None
        u.username = "test_username"
        assert u.username == "test_username"
        assert d.username == "test_username"
        setattr(u, "username", "test2")
        assert u.username == "test2"
        d.username = "test3"
        assert d.username == "test3"
        assert u.username == "test3"
        assert u.id == u_id

    def test_accessing_display_name(self, unload_users, u_id, u, d, d2):
        assert u.id == u_id
        assert u.first_name is None
        assert u.last_name is None
        assert d.__display_name__ is None
        assert d.username is None

        # If nothing is set, ID is used
        assert u.display_name == u_id
        assert d.display_name == u_id
        assert d.__display_name__ is None
        assert d2.display_name == u_id

        # If no first name & last name, or no display name, but a username, username is used
        u.username = "display_username"
        assert u.display_name == "display_username"
        assert d.display_name == "display_username"
        assert d.__display_name__ is None
        assert d2.display_name == u_id

        # If a first name but no last name is given, the username is still used
        u.first_name = "User"
        assert u.display_name == "display_username"
        assert d.display_name == "display_username"
        assert d.__display_name__ is None
        assert d2.display_name == u_id

        # Vice-versa is also true
        u.first_name = None
        u.last_name = "Test"
        assert u.display_name == "display_username"
        assert d.display_name == "display_username"
        assert d.__display_name__ is None
        assert d2.display_name == u_id

        # But, if both first and last names are available, that is used.
        u.first_name = "User"
        u.last_name = "Test"
        assert u.display_name == "User Test"
        assert d.__display_name__ is None
        assert d.display_name == "User Test"
        assert d2.display_name == u_id

        # Finally, any given display name is used
        u.display_name = "user_display_name"
        assert u.display_name == "user_display_name"
        assert d.__display_name__ == "user_display_name"
        assert d2.display_name == u_id

        # Display name can also be set from the dataset
        d.display_name = "dataset_display_name"
        assert u.display_name == "dataset_display_name"
        assert d.display_name == "dataset_display_name"
        assert d.__display_name__ == "dataset_display_name"
        assert d2.display_name == u_id

    def test_getting_and_setting_arbitrary_data(self, unload_users, u, d, d2):
        assert len(u.data_store) == 0
        assert "test" not in u.data_store
        u.data_store["test"] = 1
        assert u.data_store["test"] == 1
        assert d.data_store["test"] == 1
        assert "test" not in d2.data_store

    def test_lookup_hierarchy(self, unload_users, u, d, d2):
        assert d.first_name is None
        assert d2.first_name is None
        assert u.first_name is None

        n = "backup"
        d2.first_name = n
        assert d.first_name is None
        assert d2.first_name == n
        assert u.first_name == n

    def test_customizing_data_lookup_hierarchy(self, unload_users, users, ddk):
        test_n = "test"
        backup_n = "backup"

        users.add_dataset(backup_n)
        users.add_dataset(test_n)

        u = self.user()
        u2 = self.user(2)

        assert u.data_lookup_hierarchy == ["test", "backup", ddk]
        assert u2.data_lookup_hierarchy == ["test", "backup", ddk]

        u.datasets["test"].first_name = test_n
        u.datasets["backup"].first_name = backup_n
        assert u.first_name == test_n

        # Swap the lookup hierarchy
        u.data_lookup_hierarchy = list(reversed(u.data_lookup_hierarchy))
        assert u.data_lookup_hierarchy == [ddk, "backup", "test"]
        assert u.first_name == backup_n

        # Check that the hierarchy lookup is still functioning
        u.datasets["backup"].first_name = None
        assert u.first_name == test_n

        # Messing about with the datakey hierarchy is a per-user ordeal
        assert u2.data_lookup_hierarchy == ["test", "backup", ddk]

    def test_data_lookup_hierarchy_is_returned_by_value(
            self, unload_users, u, d, d2, d2_name, ddk):
        assert u.data_lookup_hierarchy == [ddk, d2_name]

        # should be no error here as the update attempt never makes it to the backend
        u.data_lookup_hierarchy.append("hi")
        assert u.data_lookup_hierarchy == [ddk, d2_name]

    def test_error_on_unknown_dataset_when_setting_hierarchies(
            self, unload_users, u, d2, d2_name, ddk):
        assert u.data_lookup_hierarchy == [ddk, d2_name]
        with pytest.raises(
                RuntimeError,
                match=
                "The following datasets do not exists and cannot be used in the data lookup hierarchy: '(test|hi)', '(test|hi)'"
        ):
            u.data_lookup_hierarchy = [d2_name, "test", "hi"]
        assert u.data_lookup_hierarchy == [ddk, d2_name]

    def test_error_on_duplicate_datasets_when_setting_hierarchies(
            self, unload_users, u, d2, d2_name, ddk):
        assert u.data_lookup_hierarchy == [ddk, d2_name]
        with pytest.raises(
                RuntimeError,
                match=
                rf"Dataset '{d2_name}' can only appear once in the dataset hierarchy \(first appearance at index 0 - duplicate at index 2\)"
        ):
            u.data_lookup_hierarchy = [d2_name, ddk, d2_name]
        assert u.data_lookup_hierarchy == [ddk, d2_name]

    def test_empty_data_lookup_hierarchy(self, unload_users, u, d, d2):
        assert d.first_name == None
        d2.first_name = "first"
        assert d2.first_name == "first"

        u.data_lookup_hierarchy = []
        assert u.data_lookup_hierarchy == []

        with pytest.raises(
                RuntimeError,
                match=
                "Dataset hierarchy is empty! Data lookups must explicitly name the dataset to query"
        ):
            u.first_name
        with pytest.raises(
                RuntimeError,
                match=f"Data lookup hierarchy for user '{u.id}' is empty"):
            u.top_datakey
        assert d.first_name == None
        assert d2.first_name == "first"

    def test_passwords(self, unload_users, u, d, d2):
        u.password = "blah!"
        d2.password = "!PASSWORD!"
        assert u.password == "blah!"
        assert d.password == "blah!"
        assert d2.password == "!PASSWORD!"

        d.password = "PASSWORD"
        assert u.password == "PASSWORD"
        assert d.password == "PASSWORD"
        assert d2.password == "!PASSWORD!"

    def test_comparing_datasets(self, unload_users, u, u2, d, d2):
        assert u.datasets[d.dataset_name] == d
        assert u.datasets[d.dataset_name] == u.datasets[d.dataset_name]
        assert u.datasets[d.dataset_name] != u.datasets[d2.dataset_name]
        assert u.datasets[d.dataset_name] != u2.datasets[d.dataset_name]

    class TestDataStoreDictLike(Fixture_DictLikeAPI, Base):
        def parameterize(self):
            return {
                "keys": ["test", "test 1", "test 2"],
                "klass": str,
                "not_in_dut": "Blah"
            }

        def boot_dict_under_test(self):
            return self.user().datasets[self.get_ddk].data_store

        def init_dict_under_test(self):
            users = self.get_users
            users.unload()
            u = self.user()
            d = u.datasets[self.get_ddk]
            d.data_store["test"] = "zero"
            d.data_store["test 1"] = "one"
            d.data_store["test 2"] = "two"
