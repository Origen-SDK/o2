import pytest
import origen_metal as om
from .tests__datasets import Base


class T_UserMotives(Base):
    def test_adding_and_retrieving_default_motives(self, unload_users, users,
                                                   ddk):
        assert users.motives == {}
        assert users.add_motive("test", ddk) is None
        assert users.motives["test"] == ddk
        assert users.dataset_for("test") == ddk

    def test_missing_motives(self, unload_users, users):
        assert users.dataset_for("missing") == None

    def test_default_datasets_propagate_to_a_user(self, unload_users, users, u,
                                                  ddk):
        assert u.motives == {}
        assert users.motives == {}

        # Add a default motive and create a new user
        assert users.add_motive("test", ddk) is None

        u2 = users.add("u2")
        assert u2.motives == {"test": ddk}

        # Original user should be unchanged
        assert u.motives == {}

    def test_error_adding_motives_with_undefined_dataset(
            self, unload_users, users):
        assert users.motives == {}
        with pytest.raises(
                RuntimeError,
                match=
                "Cannot add motive corresponding to nonexistent dataset 'nonexistent'"
        ):
            users.add_motive("test_ne", "nonexistent")
        assert users.motives == {}

    def test_replacing_default_motives(self, unload_users, users, ddk,
                                       ensure_users_tdk, tdk):
        tr = "test_replacement"
        users.add_motive(tr, ddk)
        msg = f"Motive '{tr}' already corresponds to dataset '{ddk}'. Use the 'replace_existing' option to update the motive"
        with pytest.raises(RuntimeError, match=msg):
            users.add_motive(tr, tdk)
        assert users.dataset_for(tr) == ddk

        with pytest.raises(RuntimeError, match=msg):
            users.add_motive(tr, ddk, replace_existing=False)
        assert users.dataset_for(tr) == ddk

        assert users.add_motive(tr, tdk, replace_existing=True) == ddk
        assert users.dataset_for(tr) == tdk

    def test_adding_and_retrieving_motives_for_a_user(self, unload_users,
                                                      users, u, ddk):
        assert u.motives == {}
        assert users.motives == {}
        assert u.add_motive("test", ddk) is None
        u.motives["test"] == ddk
        d = u.dataset_for("test")
        assert isinstance(d, self.user_dataset_class)
        assert d.dataset_name == ddk

        assert u.add_motive("test_ds", d) is None
        d2 = u.dataset_for("test_ds")
        assert d2.dataset_name == ddk

    def test_missing_motives_for_a_user(self, unload_users, u):
        assert u.dataset_for("Unknown") == None

    def test_error_adding_motives_with_undefined_dataset_for_a_user(
            self, unload_users, u):
        with pytest.raises(
                RuntimeError,
                match=
                f"Cannot add motive for user '{u.id}' corresponding to nonexistent dataset 'non_existent'"
        ):
            u.add_motive("test_ne", "non_existent")

    def test_replacing_motives_for_a_user(self, unload_users, ensure_users_tdk,
                                          u, ddk, tdk):
        n = "test_replacement"
        u.add_motive(n, ddk)
        msg = f"Motive '{n}' for user '{u.id}' already corresponds to dataset '{ddk}'. Use the 'replace_existing' option to update the motive"
        with pytest.raises(RuntimeError, match=msg):
            u.add_motive(n, tdk)
        assert u.motives[n] == ddk

        with pytest.raises(RuntimeError, match=msg):
            u.add_motive(n, tdk, replace_existing=False)
        assert u.motives[n] == ddk

        assert u.add_motive(n, tdk, replace_existing=True) == ddk
        assert u.motives[n] == tdk

    # TEST_NEEDED
    @pytest.mark.skip
    def test_retrieving_datasets_from_motives_with_default(
            self, unload_users, u):
        raise NotImplementedError

    # TEST_NEEDED
    @pytest.mark.skip
    def test_retrieving_datasets_from_motives_with_lookup_chain(
            self, unload_users, u):
        raise NotImplementedError

    # TEST_NEEDED
    @pytest.mark.skip
    def test_retreiving_data_from_motives(self, unload_users):
        raise NotImplementedError

    # TEST_NEEDED
    @pytest.mark.skip
    def test_retreiving_data_from_motives_with_default(self, unload_users):
        raise NotImplementedError

    # TEST_NEEDED
    @pytest.mark.skip
    def test_retreiving_data__from_motives_with_lookup_chain(
            self, unload_users):
        raise NotImplementedError

    def test_password_motives(self, unload_users, u, d, d2):
        d.password = "PASSWORD"
        d2.password = "!PASSWORD!"

        u.add_motive("just because", d2.dataset_name)

        assert u.password_for("just because") == "!PASSWORD!"
        with pytest.raises(
                RuntimeError,
                match=f"No password available for motive: 'Nothing!'"):
            u.password_for("Nothing!")
        assert u.dataset_for("just because").dataset_name == d2.dataset_name
        assert u.dataset_for("nothing") == None
        assert u.password_for("Nothing!", default=None) == "PASSWORD"
        assert u.password_for("Nothing!",
                              default=d2.dataset_name) == "!PASSWORD!"
        assert u.password_for("Nothing!",
                              default=d2) == "!PASSWORD!"

        # Corner case where a default dataset that doesn't exist is given
        missing = "MIA dataset"
        with pytest.raises(
                RuntimeError,
                match=
                f"A default dataset '{missing}' was provided, but this dataset does not exists for user '{u.id}'"
        ):
            u.password_for("Nothing!", default=missing)
