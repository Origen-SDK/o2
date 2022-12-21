import pytest
import origen_metal as om
from .shared import Base


class T_InitialAndCurrentUser(Base):
    def test_setting_existing_user_as_current(self, users, u_id):
        # This will also update the initial user
        assert users.current_user == None
        assert users.initial_user == None
        assert om.current_user == None

        assert users.set_current_user(u_id)

        assert isinstance(users.current_user, self.user_class)
        assert isinstance(users.initial_user, self.user_class)
        assert isinstance(om.current_user, self.user_class)
        assert users.current_user.id == u_id
        assert users.initial_user.id == u_id
        assert om.current_user.id == u_id

    def test_error_setting_non_existant_user_as_current(
            self, users, u_id, missing_id):
        with pytest.raises(
                RuntimeError,
                match=
                f"Cannot set current user with id '{missing_id}'. User has not been added yet!"
        ):
            users.set_current_user(missing_id)
        assert users.current_user.id == u_id

    def test_updating_current_user_preserves_initial_user(
            self, users, u_id, u2_id):
        assert users.current_user.id == u_id
        assert users.initial_user.id == u_id
        assert om.current_user.id == u_id

        assert users.set_current_user(u2_id)
        assert users.current_user.id == u2_id
        assert users.initial_user.id == u_id
        assert om.current_user.id == u2_id

    def test_setting_current_user_with_user_class(self, users, u2_id, u3,
                                                  u3_id):
        assert users.current_user.id == u2_id
        assert users.set_current_user(u3)
        assert users.current_user.id == u3_id
        assert om.current_user.id == u3_id

    def test_setting_current_user_with_setter(self, users, u_id, u2, u2_id,
                                              u3_id):
        assert users.current_user.id == u3_id
        assert users.initial_user.id == u_id
        assert om.current_user.id == u3_id

        users.current_user = u_id
        assert users.current_user.id == u_id
        assert users.initial_user.id == u_id
        assert om.current_user.id == u_id

        users.current_user = u2
        assert users.current_user.id == u2_id
        assert users.initial_user.id == u_id
        assert om.current_user.id == u2_id

        with pytest.raises(TypeError,
                           match="Cannot resolve user from type 'list'"):
            users.current_user = []

    def test_setting_same_current_user_returns_false(self, users, u2_id):
        assert users.current_user.id == u2_id
        assert om.current_user.id == u2_id

        assert not users.set_current_user(u2_id)
        assert users.current_user.id == u2_id
        assert om.current_user.id == u2_id

    def test_clearing_current_user(self, users, u, u_id, u2_id):
        assert users.current_user.id == u2_id

        users.current_user = None
        assert users.current_user == None
        assert users.initial_user.id == u_id

        assert users.set_current_user(u)
        assert users.current_user.id == u_id

        assert users.clear_current_user() is True
        assert users.current_user == None

        assert users.clear_current_user() is False
        assert users.current_user == None

    def test_removing_the_current_user(self, unload_users, users, u, u_id):
        users.set_current_user(u)
        assert users.current_user.id == u_id
        assert users.initial_user.id == u_id
        users.remove(u.id)
        assert u_id not in users
        assert users.current_user is None

        # The initial ID will stick around, even though its (now) invalid
        # Will return an error but the user ID could be extracted from that
        with pytest.raises(
                RuntimeError,
                match=f"Initial user '{u_id}' is no longer an active user!"):
            assert users.initial_user == u_id

        self.user()
        assert users.initial_user.id == u_id

    def test_resolving_current_user(self, unload_users, users):
        # This should resolve the current user, but not actually set it,
        # nor add it to the 'users' list.
        assert len(users.ids) == 0
        assert users.current == None
        assert users.initial == None

        assert users.lookup_current_id() == self.logged_in_id
        assert len(users.ids) == 0
        assert users.current == None
        assert users.initial == None

    def test_resolving_current_user_via_frontend(self, fresh_frontend, users):
        # This should resolve the current user, but not actually set it,
        # nor add it to the 'users' list.
        assert len(users.ids) == 0
        assert users.current == None
        assert users.initial == None

        assert users.lookup_current_id() == self.logged_in_id
        assert users.lookup_current_id_function is None
        users.lookup_current_id_function = self.lookup_current_id_function
        assert users.lookup_current_id_function == users.lookup_current_id_function

        assert users.lookup_current_id() == "__frontend_user__"
        assert len(users.ids) == 0
        assert users.current == None
        assert users.initial == None

        users.lookup_current_id_function = None
        assert users.lookup_current_id_function is None
        assert users.lookup_current_id() == self.logged_in_id

    def test_resolving_and_setting_current_user(self, unload_users, users):
        assert len(users.ids) == 0
        assert users.current == None
        assert users.initial == None

        users.lookup_current_id(update_current=True)
        id = self.logged_in_id
        assert len(users.ids) == 1
        assert id in users
        assert users.current.id == id
        assert users.initial.id == id

    def test_resolving_and_setting_current_user_preserves_initial(
            self, unload_users, users):
        assert len(users.ids) == 0
        assert users.current == None
        assert users.initial == None

        u_id = self.user_id_root
        users.add(u_id)
        users.set_current_user(u_id)
        assert users.current.id == u_id
        assert users.initial.id == u_id

        users.lookup_current_id(update_current=True)
        assert users.current.id == self.logged_in_id
        assert users.initial.id == u_id

    def test_knows_if_it_is_current_user(self, unload_users, users, u, u2):
        users.set_current_user(u)
        assert u.is_current is True
        assert u.is_current_user is True
        assert u2.is_current is False
        assert u2.is_current_user is False

        users.set_current_user(u2)
        assert u.is_current is False
        assert u.is_current_user is False
        assert u2.is_current is True
        assert u2.is_current_user is True

    # TODO
    # def test_auto_populating_with_username(self):
    #     fail

    def test_temporarily_switching_current_user(self, unload_users, users, u, u2):
        users.set_current_user(u)
        assert users.current == u

        # Switch with user instance
        with users.current_user_as(u2) as current:
            assert current == u2
            assert users.current.id == u2.id
        assert users.current == u

        # Switch with user ID
        with users.current_user_as(u2.id) as current:
            assert current == u2
            assert users.current.id == u2.id
        assert users.current == u

        # Unsupported type
        with pytest.raises(TypeError, match="Cannot resolve user from type 'int'"):
            with users.current_user_as(0) as current:
                pass


    def test_temporarily_switching_current_user_from_none(self, unload_users, users, u, u_id):
        assert users.current is None
        with users.current_user_as(u_id) as current:
            assert current.id == u_id
            assert users.current.id == u_id
        assert users.current is None

    def test_temporarily_switching_current_user_to_none(self, unload_users, users, u, u_id):
        users.set_current_user(u)
        assert users.current == u

        with users.current_user_as(None) as current:
            assert current is None
            assert users.current is None
        assert users.current == u

    def test_user_is_switched_back_even_if_errors_occur(self, unload_users, users, u, u_id):
        assert users.current == None
        with pytest.raises(RuntimeError, match="Error!"):
            with users.current_user_as(u_id):
                assert users.current.id == u_id
                raise RuntimeError("Error!")
        assert users.current == None

    def test_error_switching_to_non_existent_user(self, unload_users, users, u, u_id):
        users.set_current_user(u)
        mia = "missing_user"
        with pytest.raises(RuntimeError, match=f"Cannot set current user with id '{mia}'. User has not been added yet!"):
            with users.current_user_as(mia):
                raise RuntimeError("Should never get here")
        assert users.current == u
        assert users.current == u
