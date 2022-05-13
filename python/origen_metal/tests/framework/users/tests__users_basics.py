import pytest
import origen_metal as om
from .shared import Base
from tests.shared.python_like_apis import Fixture_DictLikeAPI

class T_Users(Base):
    def test_users_are_accessible(self):
        assert isinstance(om.users, om._origen_metal.framework.users.Users)

    def test_default_state(self, unload_users, users):
        assert len(users.ids) == 0
        assert users.current == None
        assert users.current_user == None
        assert users.initial == None
        assert users.initial_user == None
        assert om.current_user == None

    def test_adding_a_simple_user(self, users):
        u_id = self.user_id_root
        assert u_id not in users
        assert isinstance(users.add(u_id), self.user_class)
        assert u_id in users

    def test_error_on_adding_existing_users(self, users, u_id):
        assert len(users.ids) == 1
        assert u_id in users
        with pytest.raises(RuntimeError, match=f"User '{u_id}' has already been added"):
            users.add(u_id)
        assert len(users.ids) == 1
        assert u_id in users

    def test_retrieving_users(self, u, u_id):
        assert isinstance(u, self.user_class)
        assert u.id == u_id

    class TestUsersDictLike(Fixture_DictLikeAPI, Base):
        def parameterize(self):
            return {
                "keys": [self.to_user_id(1), self.to_user_id(2), self.to_user_id(3), self.to_user_id(4)],
                "klass": self.user_class,
                "not_in_dut": "Blah"
            }

        def boot_dict_under_test(self):
            return self.get_users
        
        def init_dict_under_test(self):
            self.clean_users()
            self.user(1)
            self.user(2)
            self.user(3)
            self.user(4)
