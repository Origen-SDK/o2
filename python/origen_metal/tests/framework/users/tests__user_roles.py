import pytest
import origen_metal as om
from pathlib import Path
from .shared import Base
from tests.utils.test_sessions import Common as SessionsBase
from tests.test_file_permissions import new_fp

class T_UserRoles(Base):
    def rn(self, id):
        return f"role_name_{id}"

    @pytest.fixture
    def rn1(self):
        return self.rn(1)

    @pytest.fixture
    def rn2(self):
        return self.rn(2)

    @pytest.fixture
    def rn3(self):
        return self.rn(3)

    @pytest.fixture
    def rn4(self):
        return self.rn(4)

    @pytest.fixture
    def rn5(self):
        return self.rn(5)

    @pytest.fixture
    def rn6(self):
        return self.rn(6)

    @pytest.fixture
    def rn_ne(self):
        return "non_existent"

    @pytest.fixture
    def err_msg_for_required(self, rn_ne):
        return f"No users with role '{rn_ne}' could be found"

    def test_user_roles_start_empty(self, unload_users, u, rn1):
        assert u.roles == []
        assert rn1 not in u.roles

    def test_user_roles_can_be_added(self, u, rn1, rn2, rn3, rn4):
        assert rn1 not in u.roles
        assert u.add_roles(rn1) == [True]
        assert u.roles == [rn1]
        assert rn1 in u.roles

        assert u.add_roles(rn2, rn3, rn4) == [True, True, True]
        assert set(u.roles) == {rn1, rn2, rn3, rn4}

    def test_adding_existing_role_to_user(self, u, rn1, rn2, rn3, rn4, rn5):
        assert u.add_roles(rn1) == [False]
        assert u.add_roles(rn2, rn5, rn4) == [False, True, False]
        assert set(u.roles) == {rn1, rn2, rn3, rn4, rn5}

    def test_user_roles_can_be_removed(self, u, rn1, rn2, rn3, rn4, rn_ne):
        assert rn1 in u.roles
        assert rn2 in u.roles
        assert u.remove_roles(rn2) == [True]
        assert rn1 in u.roles
        assert rn2 not in u.roles
        assert u.remove_roles(rn3, rn4, rn_ne) == [True, True, False]

    def test_user_roles_can_be_applied_during_creation(self, users, rn1, rn2, rn3):
        users.default_roles == []
        u2 = users.add(self.to_user_id(2))
        assert u2.roles == []

        users.default_roles = [rn1, rn2]
        assert set(users.default_roles) == {rn1, rn2}
        u3 = users.add(self.to_user_id(3))
        assert set(u3.roles) == {rn1, rn2}

        users.default_roles = None
        assert users.default_roles == []

        users.default_roles = rn3
        assert users.default_roles == [rn3]

        users.default_roles = []
        assert users.default_roles == []

        users.default_roles = rn3
        assert users.default_roles == [rn3]
        with pytest.raises(TypeError, match="Cannot interpret roles as either 'str', 'list of strs', or 'None'."):
            users.default_roles = 0
        assert users.default_roles == [rn3]

        with pytest.raises(RuntimeError, match=fr"Input contains duplicate value '{rn1}' at index 2, which is not allowed in the this context \(first occurrence at index 0\)"):
            users.default_roles = [rn1, rn2, rn1]
        assert users.default_roles == [rn3]

    def test_getting_all_roles(self, unload_users, users, u, u2, rn1, rn2, rn3, rn4, rn5, rn6):
        u.add_roles(rn1, rn2, rn3, rn4)
        u2.add_roles(rn3, rn4, rn5, rn6)
        roles = users.roles
        assert isinstance(roles, list)
        assert set(roles) == {rn1, rn2, rn3, rn4, rn5, rn6}

    def test_getting_all_roles_by_user(self, unload_users, users, u, u2, rn1, rn2, rn3, rn4, rn5, rn6):
        u.add_roles(rn1, rn2, rn3, rn4)
        u2.add_roles(rn3, rn4, rn5, rn6)
        roles_by_user = users.by_role
        assert isinstance(roles_by_user, dict)
        assert roles_by_user == {
            rn1: [u],
            rn2: [u],
            rn3: [u, u2],
            rn4: [u, u2],
            rn5: [u2],
            rn6: [u2]
        }

    def test_getting_users_for_enumerated_roles(self, users, u, u2, rn1, rn3, rn_ne):
        assert users.for_role(rn1) == [u]
        assert users.for_role(rn3) == [u, u2]
        assert users.for_role(rn_ne) == []

        assert users.for_exclusive_role(rn1) == u
        assert users.for_exclusive_role(rn_ne) == None

    def test_requiring_at_least_one_user_is_available_for_role(self, users, u, u2, rn1, rn3, rn_ne, err_msg_for_required):
        assert users.for_role(rn1, required=True) == [u]
        assert users.for_role(rn3, required=True) == [u, u2]

        with pytest.raises(RuntimeError, match=err_msg_for_required):
            users.for_role(rn_ne, required=True)

        assert users.for_exclusive_role(rn1, required=True) == u
        with pytest.raises(RuntimeError, match=err_msg_for_required):
            users.for_exclusive_role(rn_ne, required=True)

    def test_getting_exclusive_user_roles(self, users, u, u2, rn1, rn3, rn_ne, err_msg_for_required):
        assert users.for_role(rn1, exclusive=True) == [u]
        assert users.for_role(rn_ne, exclusive=True) == []

        assert users.for_exclusive_role(rn1) == u
        assert users.for_exclusive_role(rn_ne) == None

        err_msg = f"Found multiple users matching exclusive role '{rn3}': '{u.id}', '{u2.id}'"
        with pytest.raises(RuntimeError, match=err_msg):
            users.for_role(rn3, exclusive=True)
        with pytest.raises(RuntimeError, match=err_msg):
            users.for_exclusive_role(rn3)

        with pytest.raises(RuntimeError, match=err_msg_for_required):
            users.for_role(rn_ne, exclusive=True, required=True)
        with pytest.raises(RuntimeError, match=err_msg_for_required):
            users.for_exclusive_role(rn_ne, required=True)
