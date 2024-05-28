import pytest, getpass
import origen_metal as om
from origen_metal.framework.users import UserDatasetConfig


def get_users():
    return om.users


def clean_users():
    users = get_users()
    users.unload()
    assert users.current is None
    assert users.initial is None
    assert len(users) == 0


@pytest.fixture
def unload_users():
    users = get_users()
    users.unload()
    assert users.current is None
    assert users.initial is None
    assert len(users) == 0


@pytest.fixture
def users():
    return get_users()


default_datakey = "__origen__default__"

no_current_user_error_msg = "No current user has been set!"

class Base:
    DATA_FIELDS = ["email", "first_name", "last_name"]
    DATA_FIELD_EXCEPTIONS = ["display_name", "username"]

    @property
    def get_users(self):
        return get_users()

    def ensure_user(self, id):
        if not id in self.get_users:
            self.get_users.add(id)
        return self.get_users[id]

    def clean_users(self):
        return clean_users()

    @pytest.fixture
    def unload_users(self, users):
        return self.clean_users()

    @pytest.fixture
    def users(self):
        return self.get_users

    @pytest.fixture
    def u_id(self):
        id = self.user_id_root
        self.ensure_user(id)
        return id

    @pytest.fixture
    def u2_id(self):
        return self.user(2).id

    @pytest.fixture
    def u3_id(self):
        return self.user(3).id

    @pytest.fixture
    def u(self, u_id):
        return self.ensure_user(u_id)

    @pytest.fixture
    def u2(self):
        return self.user(2)

    @pytest.fixture
    def u3(self):
        return self.user(3)

    @pytest.fixture
    def missing_id(self):
        return self.to_user_id("missing")

    @property
    def user_id_root(self):
        return "test_u"

    def user(self, id=None):
        if id:
            return self.ensure_user(self.to_user_id(id))
        else:
            return self.ensure_user(self.user_id_root)

    def to_user_id(self, id):
        return f"{self.user_id_root}_{id}"

    @pytest.fixture
    def cu(self, users, u):
        if users.current_user is None:
            users.current_user = u
        return users.current_user

    @property
    def users_class(self):
        return om._origen_metal.framework.users.Users

    @property
    def user_class(self):
        return om._origen_metal.framework.users.User

    @property
    def user_dataset_class(self):
        return om._origen_metal.framework.users.UserDataset

    @property
    def user_dataset_config_class(self):
        return om._origen_metal.framework.users.UserDatasetConfig

    @property
    def users_session_config_class(self):
        return om._origen_metal.framework.users.UsersSessionConfig

    @property
    def user_session_config_class(self):
        return om._origen_metal.framework.users.UserSessionConfig

    @property
    def logged_in_id(self):
        return getpass.getuser()

    # Test-data-key
    @pytest.fixture
    def tdk(self):
        return "test_dk"

    @pytest.fixture
    def ensure_users_tdk(self, users, tdk):
        if tdk not in users.datasets:
            users.add_dataset(tdk)
            return False
        else:
            return True

    @property
    def get_cat_name(self):
        return "dummy_cat"

    # Test category name
    @pytest.fixture
    def cat_name(self):
        return self.get_cat_name

    @property
    def get_dstore_name(self):
        return "dummy_ds"

    # Test datastore name
    @pytest.fixture
    def dstore_name(self):
        return self.get_dstore_name

    @pytest.fixture
    def def_dsc(self):
        return UserDatasetConfig()

    @pytest.fixture
    def def_dsc_dict(self):
        return dict(UserDatasetConfig())

    def def_dsc_with(self, **config):
        return UserDatasetConfig(**config)

    def def_dsc_dict_with(self, **config):
        return dict(self.def_dsc_with(**config))

    @property
    def dummy_dsc(self):
        return self.dummy_dsc_with(**{})

    @property
    def dummy_dsc_dict(self):
        return dict(self.dummy_dsc_with(**{}))

    def dummy_dsc_with(self, **config_vals):
        '''Creates a new base/dummy dataset config merged with the given config'''
        base = {
            "category": self.get_cat_name,
            "data_store": self.get_dstore_name,
            "auto_populate": False
        }
        base.update(**config_vals)
        return om._origen_metal.framework.users.UserDatasetConfig(**base)

    def dummy_dsc_dict_with(self, **config_vals):
        '''Creates a new base/dummy dataset config merged with the given config and casts to a dict'''
        return dict(self.dummy_dsc_with(**config_vals))

    @property
    def get_ddk(self):
        ''' Default-data-key property '''
        return default_datakey

    @pytest.fixture
    def ddk(self):
        ''' Default-data-key pytest fixture '''
        return self.get_ddk

    def lookup_current_id_function(self):
        ''' Function that will be called by the backend to get the current user ID from a frontend-defined function '''
        return "__frontend_user__"

    @pytest.fixture
    def ensure_frontend_dummy_category(self, needs_frontend, cat_name):
        if cat_name not in needs_frontend.data_stores:
            needs_frontend.data_stores.add_category(cat_name)
        return needs_frontend.data_stores[cat_name]

    def missing_email_error_msg(self, u):
        if isinstance(u, self.user_class):
            u = u.id
        return f"Tried to retrieve email for user '{u}' but no email has been set!"