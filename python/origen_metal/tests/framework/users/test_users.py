import pytest
import origen_metal as om
from .shared import Base

from .tests__users_basics import T_Users
from .tests__initial_and_current_user import T_InitialAndCurrentUser
from .tests__datasets import T_Datasets
from .tests__user_motives import T_UserMotives
from .tests__populating import T_PopulatingUsers
from .tests__user_sessions import T_UserSessions
from .tests__validating_passwords import T_ValidatingPasswords
from .tests__user_roles import T_UserRoles
from .tests__home_dir import T_UserHomeDirectory

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

class TestUserHomeDirectory(T_UserHomeDirectory):
    pass

class TestDisablingPasswordPrompt(Base):
    @property
    def prompt_error_msg(self):
        return f"Cannot prompt for passwords for user '{self.get_users.current_user.id}'. Passwords must be loaded by the config or set directly."

    def test_password_prompt_enabled_by_default(self, fresh_frontend, unload_users, cu):
        assert cu.prompt_for_passwords == True
        assert cu.__prompt_for_passwords__ == None

    def test_setting_password_prompt(self, cu):
        assert cu.prompt_for_passwords == True
        assert cu.__prompt_for_passwords__ == None

        cu.prompt_for_passwords = True
        assert cu.prompt_for_passwords == True
        assert cu.__prompt_for_passwords__ == True

    def test_disabling_password_prmopt(self, cu):
        cu.prompt_for_passwords = False
        assert cu.prompt_for_passwords == False
        assert cu.__prompt_for_passwords__ == False

        with pytest.raises(RuntimeError, match=self.prompt_error_msg):
            cu.password

        cu.add_dataset("test")
        with pytest.raises(RuntimeError, match=self.prompt_error_msg):
            cu.datasets['test'].password
