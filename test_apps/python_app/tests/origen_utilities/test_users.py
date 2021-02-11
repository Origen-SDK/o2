import origen, _origen, pytest, os, getpass
from tests.shared.python_like_apis import Fixture_DictLikeAPI
from tests.origen_utilities.test_ldap import USER_USERNAME, PASSWORD

data_fields = ["email", "first_name", "last_name"]  # name?
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

    def test_accessing_datasets(self):
        u = self.user
        assert "test" in u.datasets
        assert "test2" in u.datasets
        assert isinstance(u.datasets, dict)
        assert "forumsys" in u.datasets
        assert u.dataset == "test"

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
            "keys": [getpass.getuser(), "test", "test2"],
            "klass": _origen.users.User,
            "not_in_dut": "Blah"
        }

    def boot_dict_under_test(self):
        return origen.users


#     def test_home_dir():
#         ...

#     def test_session():
#         ...
