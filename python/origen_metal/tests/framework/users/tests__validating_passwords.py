import pytest, copy
import origen_metal as om
from .shared import Base
from origen_metal.frontend.data_store_api import DataStoreAPI
from tests.test_frontend import Common as FECommon

class T_ValidatingPasswords(Base, FECommon):
    class DSValidatePasswordAPI(FECommon.DummyGetStoreDataStore, Base):
        @DataStoreAPI.validate_password
        def validate_password_params(self, username, password, **kwargs):
            assert username == self.user_id_root
            assert password == "hi"
            assert len(kwargs) == 2
            assert kwargs["user"] == om.users[self.user_id_root]
            assert kwargs["dataset"] == om.users[self.user_id_root].datasets["ds_api_test"]
            return True

    class DSPasswordIsTaco(FECommon.DummyGetStoreDataStore, Base):
        @DataStoreAPI.validate_password
        def validate_password(self, username, password, **kwargs):
            if password == "Enchilada":
                raise RuntimeError("Raised exception for Enchilada")
            return password == "Taco"

    class DSPasswordIsChalupa(FECommon.DummyGetStoreDataStore, Base):
        @DataStoreAPI.validate_password
        def validate_password(self, username, password, **kwargs):
            return password == "Chalupa"

    @pytest.fixture(autouse=True)
    def dstore_pw_validation(self, ensure_frontend_dummy_category):
        n = self.dstore_pw_validation_name
        if n not in ensure_frontend_dummy_category:
            ensure_frontend_dummy_category.add(n, self.DSPasswordIsTaco)

        n = "dstore_chalupa"
        if n not in ensure_frontend_dummy_category:
            ensure_frontend_dummy_category.add(n, self.DSPasswordIsChalupa)

        n = "dstore_api_test"
        if n not in ensure_frontend_dummy_category:
            ensure_frontend_dummy_category.add(n, self.DSValidatePasswordAPI)

        n = "min_ds"
        if n not in ensure_frontend_dummy_category:
            ensure_frontend_dummy_category.add(n, self.MinimumDataStore)

    @property
    def dstore_pw_validation_name(self):
        return "ds_pw_is_taco"

    def add_pw_ds(self, user, name=None, default_ops=False):
        if default_ops:
            return user.add_dataset(
                (name or self.pw_dstore_n),
                self.user_dataset_config_class(
                    data_store=self.dstore_pw_validation_name,
                    category=self.get_cat_name,
                )
            )
        else:
            return user.add_dataset(
                (name or self.pw_dstore_n),
                self.user_dataset_config_class(
                    data_store=self.dstore_pw_validation_name,
                    category=self.get_cat_name,
                    should_validate_password=True,
                )
            )

    def add_chalupa_ds(self, user, name=None):
        return user.add_dataset(
            (name or "chalupa_ds"),
            self.user_dataset_config_class(
                data_store="dstore_chalupa",
                category=self.get_cat_name,
                should_validate_password=True,
            )
        )

    @pytest.fixture
    def register_ds_pw(self, users, cat_name, pw_dstore_n):
        if pw_dstore_n not in users.datasets:
            return users.add_dataset(
                self.pw_dstore_n,
                self.user_dataset_config_class(
                    data_store="",
                    category=self.get_cat_name,
                    auto_populate=True,
                    should_validate_password=True,
                ))

    @property
    def pw_dstore_n(self):
        return "ds_pw"

    @pytest.fixture
    def u_ds_pw_name(self):
        return "u_ds_pw"

    @pytest.fixture
    def u_ds_pw(self, users, u, u_ds_pw_name, register_ds_pw):
        if u_ds_pw_name not in u.datasets:
            u.add_dataset(u_ds_pw_name)
        return u.datasets[u_ds_pw_name]

    @property
    def invalid_pw_msg(self):
        return "Sorry, that password is not correct"
    
    @property
    def not_current_user_msg(self):
        return "Can't get the password for a user which is not the current user"

    def test_should_validate_password_defaults(self, unload_users, users, u):
        d = self.add_pw_ds(u, default_ops=True)
        assert users.default_should_validate_passwords is None
        assert u.should_validate_passwords is True
        assert u.__should_validate_passwords__ is None
        assert d.should_validate_password is False
        assert d.config.should_validate_password is None

        # TODO
        # assert d.can_validate_password is True

    def test_validating_passwords_during_set(self, unload_users, users, u):
        d = self.add_pw_ds(u)
        assert d.should_validate_password is True
        with pytest.raises(RuntimeError, match=self.invalid_pw_msg):
            u.password = "not taco"
        u.password = "Taco"

        d2 = self.add_chalupa_ds(u)
        with pytest.raises(RuntimeError, match=self.invalid_pw_msg):
            d2.password = "Taco"
        d2.password = "Chalupa"

    def test_direct_password_validation(self, unload_users, users, u):
        d = self.add_pw_ds(u, default_ops=True)
        with pytest.raises(RuntimeError, match=self.not_current_user_msg):
            u.validate_password()
        with pytest.raises(RuntimeError, match=self.not_current_user_msg):
            d.validate_password()

        u.password = "Not Taco"
        r = u.validate_password()
        assert r.failed is True

        r = d.validate_password()
        assert r.failed is True

        u.password = "Taco"
        r = u.validate_password()
        assert r.succeeded is True

        r = d.validate_password()
        assert r.succeeded is True

    def test_validation_but_feature_not_supported(self, unload_users, users, u):
        err_msg = "Requested operation 'password validation' for user id 'test_u' requires that dataset 'test_d' contains a data source, but no 'data source' was provided."
        d = u.add_dataset("test_d", self.user_dataset_config_class())
        u.password = "hi"
        with pytest.raises(RuntimeError, match=err_msg):
            u.validate_password()
        with pytest.raises(RuntimeError, match=err_msg):
            d.validate_password()

        d = u.add_dataset("d2", self.user_dataset_config_class(
                data_store="min_ds",
                category=self.get_cat_name,
            )
        )
        d.password = "hi"
        with pytest.raises(RuntimeError, match=r"'min_ds' does not implement feature 'validate_password' \(data store category: 'dummy_cat'\)"):
            d.validate_password()

    def test_password_validation_frontend_call(self, unload_users, users, u):
        d = u.add_dataset("ds_api_test", self.user_dataset_config_class(
                data_store="dstore_api_test",
                category=self.get_cat_name,
                should_validate_password=True,
            )
        )
        u.password = "hi"
        assert u.validate_password().succeeded is True

    def test_exception_occurs_during_validation(self, unload_users, users, u):
        d = self.add_pw_ds(u)
        err = "Raised exception for Enchilada"
        with pytest.raises(RuntimeError, match=err):
            u.password = "Enchilada"

        # Same result should happen again
        with pytest.raises(RuntimeError, match=err):
            u.password = "Enchilada"

        with pytest.raises(RuntimeError, match=err):
            d.password = "Enchilada"

        d = self.add_pw_ds(u, name="ds_pw_no_val", default_ops=True)
        u.password = "Enchilada"
        with pytest.raises(RuntimeError, match=err):
            u.validate_password()
        with pytest.raises(RuntimeError, match=err):
            d.validate_password()

    def test_configuring_should_validate_passwords(self, unload_users, users, u):
        assert users.default_should_validate_passwords is None
        assert u.should_validate_passwords is True
        assert u.__should_validate_passwords__ is None

        d = self.add_pw_ds(u)
        assert d.should_validate_password is True
        assert d.config.should_validate_password is True

        # user overrides with False, DS is True - no password validation
        u1 = self.user(1)
        d1 = self.add_pw_ds(u1)
        assert u1.should_validate_passwords is True
        assert u1.__should_validate_passwords__ is None
        with pytest.raises(RuntimeError, match=self.invalid_pw_msg):
            u1.password = "hi"

        u1.should_validate_passwords = False
        assert u1.should_validate_passwords is False
        assert u1.__should_validate_passwords__ is False
        assert d1.should_validate_password is True
        assert d1.config.should_validate_password is True
        u1.password = "hi"
        assert u1.password == "hi"
        assert u1.validate_password().failed
        assert d1.validate_password().failed

        # Dataset should respect user settings
        d1.password = "hello"
        assert u1.password == "hello"
        assert d1.password == "hello"

        # user's default is False with a new user inheriting
        users.default_should_validate_passwords = False
        assert users.default_should_validate_passwords is False
        u2 = self.user(2)
        d2 = self.add_pw_ds(u2)
        assert u2.should_validate_passwords is False
        assert u2.__should_validate_passwords__ is False
        assert d2.should_validate_password is True
        assert d2.config.should_validate_password is True
        u2.password = "hi"

        # user can still override
        u2.should_validate_passwords = True
        assert u2.should_validate_passwords is True
        assert u2.__should_validate_passwords__ is True
        with pytest.raises(RuntimeError, match=self.invalid_pw_msg):
            u2.password = "hi"

        # Dataset can veto password validation
        users.default_should_validate_passwords = True
        assert users.default_should_validate_passwords is True
        u3 = self.user(3)
        d3 = self.add_pw_ds(u3, default_ops=True)
        d3_b = self.add_pw_ds(u3, default_ops=True, name="d3_b")
        assert u3.should_validate_passwords is True
        assert u3.__should_validate_passwords__ is True
        assert d3.should_validate_password is False
        assert d3.config.should_validate_password is None
        assert d3_b.should_validate_password is False
        assert d3_b.config.should_validate_password is None
        u3.password = "Hi"
        d3.password = "Hello"
        d3_b.password = "Hello"

        d3.should_validate_password = True
        assert d3.should_validate_password is True
        assert d3.config.should_validate_password is True
        assert d3_b.should_validate_password is False
        assert d3_b.config.should_validate_password is None
        with pytest.raises(RuntimeError, match=self.invalid_pw_msg):
            d3.password = "Hello"
        d3_b.password = "Hello"

        d3.should_validate_password = None
        assert d3.should_validate_password is False
        assert d3.config.should_validate_password is None

        # Previous users should be unaffected
        assert u.should_validate_passwords is True
        assert u.__should_validate_passwords__ is None
        assert u1.should_validate_passwords is False
        assert u1.__should_validate_passwords__ is False
        assert u2.should_validate_passwords is True
        assert u2.__should_validate_passwords__ is True

        users.default_should_validate_passwords = None
        assert users.default_should_validate_passwords is None
        u4 = self.user(4)
        assert u4.should_validate_passwords is True
        assert u4.__should_validate_passwords__ is None
