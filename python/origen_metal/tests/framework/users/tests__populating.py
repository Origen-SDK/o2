from __future__ import nested_scopes
import pytest, copy
import origen_metal as om
from .shared import Base
from origen_metal.frontend.data_store_api import DataStoreAPI
from tests.test_frontend import Common as FECommon


class T_PopulatingUsers(Base, FECommon):
    # special_names = {
    #     'exception_raised': { 'msg': "Error case 'exception_raised' reached!" },
    #     'raise_exception_for_u_error': {
    #         'msg': f"Error case 'raise_exception_for_u_error' for user \'{Base.to_user_id(None, 'error')}\' reached!",
    #         "user": Base.to_user_id(None, 'error')
    #     },
    #     'error_outcome_returned': { 'msg': "Error case 'error_outcome_returned' reached!" },
    #     'failed_outcome': { 'msg': "Failure case 'failed_outcome' reached!" },
    #     'fail_outcome_for_u_fail': {
    #         "msg": f"Failure case 'fail_outcome_for_u_fail' for user \'{Base.to_user_id(None, 'fail')}\' reached!",
    #         "user": Base.to_user_id(None, 'fail')
    #     },
    #     'error_first_pass_second': { 'msg': "Error first..." },
    #     'fail_first_pass_second': { 'msg': "Fail first..." },
    # }

    # TODO need to add an inheritance based test
    class DSPopUser(FECommon.DummyGetStoreDataStore, Base):
        never_popped = True
        counter = 0
        error_first = True
        fail_first = True

        # @property
        # def problem_cases(self):
        #     return TestPopulatingUsers.problem_cases

        @DataStoreAPI.populate_user
        def populate_user(self, user, ds):
            if ds.dataset_name == 'exception_raised':  #self.problem_cases['exception']['name']:
                raise NameError(f"Error case {ds.dataset_name} reached!")
            elif ds.dataset_name == 'raise_exception_for_u_error' and user.id == self.to_user_id(
                    'error'):
                raise NameError(
                    f"Error case {ds.dataset_name} for user {user.id} reached!"
                )
            elif ds.dataset_name == 'error_outcome_returned':
                return om.framework.Outcome(
                    NameError(f"Error case {ds.dataset_name} reached!"))
            elif ds.dataset_name == 'failed_outcome':
                return om.framework.Outcome(
                    False, f"Failure case {ds.dataset_name} reached!")
            elif ds.dataset_name == 'fail_outcome_for_u_fail' and user.id == self.to_user_id(
                    'fail'):
                return om.framework.Outcome(
                    False,
                    f"Failure case {ds.dataset_name} for user {user.id} reached!"
                )
            elif ds.dataset_name == 'error_first_pass_second':
                if self.error_first:
                    self.error_first = False
                    raise RuntimeError("Error first...")
            elif ds.dataset_name == 'fail_first_pass_second':
                if self.fail_first:
                    self.fail_first = False
                    return om.framework.Outcome(False, "Fail first...")
            ds.username = f"{user.id}__{self.name}"
            ds.other["populated_from"] = self.__class__.__qualname__
            ds.other["pop-ed?"] = True
            ds.other["pop counter"] = self.counter
            if self.never_popped:
                ds.other['first_pop_only'] = True
            self.never_popped = True
            self.counter += 1

    class DSPopUserChild(DSPopUser):
        ...

    class DSPopUserChildOverrideMethod(DSPopUser):
        @DataStoreAPI.populate_user
        def populate_user(self, user, ds):
            super().populate_user(user, ds)
            ds.other["populated_from_2"] = "override"

    class DSPopUserChildOverrideMethodNoDecorator(DSPopUser):
        def populate_user(self, user, ds):
            raise RuntimeError("This should not be run!")

    class DSPopUserChildNewMethod(DSPopUser):
        @DataStoreAPI.populate_user
        def populate_user_new_method(self, user, ds):
            ds.other["populated_from_2"] = "new_method"

    @property
    def pop_users_return_class(self):
        return om._origen_metal.framework.users.PopulateUsersReturn

    @property
    def pop_user_return_class(self):
        return om._origen_metal.framework.users.PopulateUserReturn

    @property
    def get_cat_name(self):
        return "FE_test_cat__users"

    @pytest.fixture
    def cat_name(self):
        return self.get_cat_name

    @pytest.fixture
    def cat(self, cat_name):
        return self.ds[cat_name]

    @pytest.fixture
    def dstore_n(self):
        return "pop_test_ds"

    @pytest.fixture
    def dstore_clsn(self):
        return self.DSPopUser.__qualname__

    def test_populating_dummy_datasets(self, fresh_frontend, unload_users,
                                       users, cat_name):
        cat = fresh_frontend.data_stores.add_category(cat_name)
        n = "pop_test_ds"
        cat.add(n, self.DSPopUser)
        n2 = "popped_test_ds"
        users.add_dataset(
            n2,
            self.user_dataset_config_class(
                data_store=n,
                category=cat_name,
            ))

        u = self.user()
        d = u.datasets[n2]
        assert d.populated == False
        assert d.populate_attempted == False
        assert d.populate_succeeded == None
        assert d.populate_failed == None
        assert d.username == None
        assert "populated_from" not in d.other
        assert "pop-ed?" not in d.other
        assert "pop counter" not in d.other
        assert "first_pop_only" not in d.other

        outcome = d.populate()
        assert outcome.succeeded == True
        assert outcome.inferred == True
        assert outcome.positional_results == None
        assert outcome.keyword_results == None
        assert outcome.metadata == None

        assert d.populated == True
        assert d.populate_attempted == True
        assert d.populate_succeeded == True
        assert d.populate_failed == False
        assert d.username == f"{u.id}__{n}"
        assert d.other["populated_from"] == self.DSPopUser.__qualname__
        assert d.other["pop-ed?"] == True
        assert d.other["pop counter"] == 0
        assert d.other["first_pop_only"] == True

    def test_repopulating(self, u):
        n2 = "popped_test_ds"
        d = u.datasets[n2]

        assert d.other["pop-ed?"] == True
        assert d.other["pop counter"] == 0
        assert d.other["first_pop_only"] == True

        assert d.populate() == None
        assert d.other["pop counter"] == 0
        assert d.other["first_pop_only"] == True

        outcome = d.populate(repopulate=True)
        assert outcome.succeeded == True
        assert outcome.inferred == True

        assert d.other["pop counter"] == 1

        # Results of previous populates persist
        assert d.other["first_pop_only"] == True

    def test_autopopulating_dummy_datasets(self, u, cat_name, dstore_n,
                                           dstore_clsn):
        dset_n = "auto_popped_test_ds"
        assert dset_n not in u.datasets

        u.add_dataset(
            dset_n,
            self.user_dataset_config_class(data_store=dstore_n,
                                           category=cat_name,
                                           auto_populate=True))
        d = u.datasets[dset_n]

        assert d.populated == True
        assert d.populate_attempted == True
        assert d.populate_succeeded == True
        assert d.populate_failed == False
        assert d.username == f"{u.id}__{dstore_n}"
        assert d.other["populated_from"] == dstore_clsn
        assert d.other["pop-ed?"] == True

    def test_errors_populating_bad_datastore_configs(self, u, dstore_n, cat,
                                                     cat_name):
        n = "missing_datastore"
        d = u.add_dataset(n,
                          self.user_dataset_config_class(category=cat_name, ))
        assert d.populated == False
        assert d.populate_attempted == False
        assert d.populate_succeeded == None
        assert d.populate_failed == None
        with pytest.raises(
                RuntimeError,
                match=
                f"Requested operation 'populate' for user id '{u.id}' requires that dataset '{n}' contains a data source, but no 'data source' was provided."
        ):
            d.populate()
        assert d.populated == False
        assert d.populate_attempted == True
        assert d.populate_succeeded == False
        assert d.populate_failed == True

        n = "invalid_datastore"
        d = u.add_dataset(
            n, self.user_dataset_config_class(
                data_store=n,
                category=cat_name,
            ))
        assert d.populated == False
        assert d.populate_attempted == False
        assert d.populate_succeeded == None
        assert d.populate_failed == None
        with pytest.raises(
                RuntimeError,
                match=
                f"Required data store '{n}' not found in category '{cat_name}"
        ):
            d.populate()
        assert d.populated == False
        assert d.populate_attempted == True
        assert d.populate_succeeded == False
        assert d.populate_failed == True

        n = "missing_category"
        d = u.add_dataset(
            n, self.user_dataset_config_class(data_store=dstore_n, ))
        assert d.populated == False
        assert d.populate_attempted == False
        assert d.populate_succeeded == None
        assert d.populate_failed == None
        with pytest.raises(
                RuntimeError,
                match=
                f"Requested operation 'populate' for user id '{u.id}' requires that dataset '{n}' contains a data source, but no 'category' was provided."
        ):
            d.populate()
        assert d.populated == False
        assert d.populate_attempted == True
        assert d.populate_succeeded == False
        assert d.populate_failed == True

        n = "invalid_category"
        d = u.add_dataset(
            n, self.user_dataset_config_class(
                data_store=dstore_n,
                category=n,
            ))
        assert d.populated == False
        assert d.populate_attempted == False
        assert d.populate_succeeded == None
        assert d.populate_failed == None
        with pytest.raises(
                RuntimeError,
                match=f"Required data store category '{n}' was not found!"):
            d.populate()
        assert d.populated == False
        assert d.populate_attempted == True
        assert d.populate_succeeded == False
        assert d.populate_failed == True

        n = "datastore_does_not_support_populating"
        ds_n = "min_ds"
        cat.add(ds_n, FECommon.MinimumDataStore)
        d = u.add_dataset(
            n,
            self.user_dataset_config_class(
                data_store=ds_n,
                category=cat_name,
            ))
        assert d.populated == False
        assert d.populate_attempted == False
        assert d.populate_succeeded == None
        assert d.populate_failed == None
        with pytest.raises(
                RuntimeError,
                match=
                rf"'{ds_n}' does not implement feature 'populate_user' \(data store category: '{cat_name}'\)"
        ):
            d.populate()
        assert d.populated == False
        assert d.populate_attempted == True
        assert d.populate_succeeded == False
        assert d.populate_failed == True

    def test_error_occurs_when_populating(self, u, dstore_n, cat_name):
        # Error case results from an exception from the populate method
        n = 'exception_raised'
        d = u.add_dataset(
            n,
            self.user_dataset_config_class(
                data_store=dstore_n,
                category=cat_name,
            ))
        assert d.populated == False
        assert d.populate_attempted == False
        assert d.populate_succeeded == None
        assert d.populate_failed == None
        # TODO try throwing actual error classes
        with pytest.raises(
                RuntimeError,
                match=
                f"Encountered Exception 'NameError' with message: Error case exception_raised reached!"
        ):
            d.populate()
        assert d.populated == False
        assert d.populate_attempted == True
        assert d.populate_succeeded == False
        assert d.populate_failed == True

        # Error case results from an outcome marked as 'error'
        n = 'error_outcome_returned'
        d = u.add_dataset(
            n,
            self.user_dataset_config_class(
                data_store=dstore_n,
                category=cat_name,
            ))
        assert d.populated == False
        assert d.populate_attempted == False
        assert d.populate_succeeded == None
        assert d.populate_failed == None
        with pytest.raises(
                RuntimeError,
                match=
                f"Errors encountered populating dataset '{n}' for user '{u.id}': Error case error_outcome_returned reached!"
        ):
            d.populate()
        assert d.populated == False
        assert d.populate_attempted == True
        assert d.populate_succeeded == False
        assert d.populate_failed == True

        # Can use an option to suppress exceptions if an error occurs
        # Implied error (Exception raised directly)
        n = 'exception_raised'
        d = u.add_dataset(n,
                          self.user_dataset_config_class(
                              data_store=dstore_n,
                              category=cat_name,
                          ),
                          replace_existing=True)
        assert d.populated == False
        assert d.populate_attempted == False
        assert d.populate_succeeded == None
        assert d.populate_failed == None
        outcome = d.populate(continue_on_error=True)
        assert d.populated == False
        assert d.populate_attempted == True
        assert d.populate_succeeded == False
        assert d.populate_failed == True
        assert outcome.succeeded == False
        assert outcome.failed == False
        assert outcome.errored == True
        assert outcome.inferred == True
        assert "Encountered Exception 'NameError' with message: Error case exception_raised reached!" in outcome.message
        assert "With traceback:" in outcome.message
        assert outcome.positional_results == None
        assert outcome.keyword_results == None
        assert outcome.metadata == None

        # Explicit error (Error outcome returned)
        n = 'error_outcome_returned'
        d = u.add_dataset(n,
                          self.user_dataset_config_class(
                              data_store=dstore_n,
                              category=cat_name,
                          ),
                          replace_existing=True)
        assert d.populated == False
        assert d.populate_attempted == False
        assert d.populate_succeeded == None
        assert d.populate_failed == None
        outcome = d.populate(continue_on_error=True)
        assert d.populated == False
        assert d.populate_attempted == True
        assert d.populate_succeeded == False
        assert d.populate_failed == True
        assert outcome.succeeded == False
        assert outcome.failed == False
        assert outcome.errored == True
        assert outcome.inferred == False
        assert outcome.message == 'Error case error_outcome_returned reached!'
        assert outcome.positional_results == None
        assert outcome.keyword_results == None
        assert outcome.metadata == None

    def test_repopulating_after_error(self, u, dstore_n, cat_name):
        n = 'error_first_pass_second'
        d = u.add_dataset(
            n,
            self.user_dataset_config_class(
                data_store=dstore_n,
                category=cat_name,
            ))
        with pytest.raises(RuntimeError, match="Error first..."):
            d.populate()
        assert d.populated == False
        assert d.populate_attempted == True
        assert d.populate_succeeded == False
        assert d.populate_failed == True

        outcome = d.populate()
        assert d.populated == True
        assert d.populate_attempted == True
        assert d.populate_succeeded == True
        assert d.populate_failed == False
        assert outcome.succeeded == True
        assert outcome.inferred == True

    def test_failure_occurs_when_populating(self, u, dstore_n, cat_name):
        # Fail case as indicated by the outcome
        # Does not raise an exception, but dataset is not marked
        # as populated and may remain partially populated
        n = 'failed_outcome'
        d = u.add_dataset(
            n,
            self.user_dataset_config_class(
                data_store=dstore_n,
                category=cat_name,
            ))
        assert d.populated == False
        assert d.populate_attempted == False
        assert d.populate_succeeded == None
        assert d.populate_failed == None
        outcome = d.populate()
        assert d.populated == False
        assert d.populate_attempted == True
        assert d.populate_succeeded == False
        assert d.populate_failed == True
        assert outcome.succeeded == False
        assert outcome.failed == True
        assert outcome.errored == False
        assert outcome.inferred == False
        assert outcome.message == 'Failure case failed_outcome reached!'

        # Can use an options to raise exceptions on failures
        with pytest.raises(RuntimeError,
                           match="Failure case failed_outcome reached!"):
            d.populate(stop_on_failure=True)
        assert d.populated == False
        assert d.populate_attempted == True
        assert d.populate_succeeded == False
        assert d.populate_failed == True

    def test_repopulating_after_failure(self, u, dstore_n, cat_name):
        n = 'fail_first_pass_second'
        d = u.add_dataset(
            n,
            self.user_dataset_config_class(
                data_store=dstore_n,
                category=cat_name,
            ))
        outcome = d.populate()
        assert d.populated == False
        assert d.populate_attempted == True
        assert d.populate_succeeded == False
        assert d.populate_failed == True
        assert outcome.failed == True
        assert outcome.inferred == False
        assert outcome.msg == "Fail first..."

        outcome = d.populate()
        assert d.populated == True
        assert d.populate_attempted == True
        assert d.populate_succeeded == True
        assert d.populate_failed == False
        assert outcome.succeeded == True
        assert outcome.inferred == True

    def test_populating_a_user(self, unload_users, u, dstore_n, cat_name):
        # Empty datasets
        pop_rtn = u.populate()
        assert isinstance(pop_rtn, self.pop_user_return_class)
        assert bool(pop_rtn) == True
        assert pop_rtn.succeeded == True
        assert pop_rtn.failed == False
        assert pop_rtn.errored == False
        assert pop_rtn.outcomes == {}

        # Populate with all successes
        n1 = "ds1"
        n2 = "ds2"
        d1 = u.add_dataset(
            n1,
            self.user_dataset_config_class(
                data_store=dstore_n,
                category=cat_name,
            ))
        d2 = u.add_dataset(
            n2,
            self.user_dataset_config_class(
                data_store=dstore_n,
                category=cat_name,
            ))
        res = u.populate()
        assert res.succeeded == True
        assert isinstance(res.outcomes, dict)
        assert list(res.outcomes.keys()) == [n1, n2]
        assert isinstance(res.outcomes[n1], om.framework.Outcome)
        assert res.outcomes[n1].succeeded == True
        assert res.outcomes[n2].succeeded == True
        assert pop_rtn.failed_datasets == []
        assert pop_rtn.errored_datasets == []
        assert pop_rtn.failed_outcomes == {}
        assert pop_rtn.errored_outcomes == {}
        assert d1.populated == True
        assert d2.populated == True

        # Populate again. No populations should actually occur as both were
        # previously populated
        res = u.populate()
        assert res.succeeded == True
        assert list(res.outcomes.keys()) == [n1, n2]
        assert res.outcomes[n1] is None
        assert res.outcomes[n2] is None

        # Add another dataset and populate again
        n3 = "ds3"
        d3 = u.add_dataset(
            n3,
            self.user_dataset_config_class(
                data_store=dstore_n,
                category=cat_name,
            ))
        res = u.populate()
        assert res.succeeded == True
        assert list(res.outcomes.keys()) == [n1, n2, n3]
        assert res.outcomes[n1] is None
        assert res.outcomes[n2] is None
        assert isinstance(res.outcomes[n3], om.framework.Outcome)
        assert res.outcomes[n3].succeeded == True

        # Should get all None's again
        res = u.populate()
        assert res.succeeded == True
        assert list(res.outcomes.keys()) == [n1, n2, n3]
        assert res.outcomes[n1] is None
        assert res.outcomes[n2] is None
        assert res.outcomes[n3] is None

        # Repopulate
        res = u.populate(repopulate=True)
        assert res.succeeded == True
        assert list(res.outcomes.keys()) == [n1, n2, n3]
        assert isinstance(res.outcomes[n1], om.framework.Outcome)
        assert res.outcomes[n1].succeeded == True
        assert isinstance(res.outcomes[n2], om.framework.Outcome)
        assert res.outcomes[n2].succeeded == True
        assert isinstance(res.outcomes[n3], om.framework.Outcome)
        assert res.outcomes[n3].succeeded == True

        # FEATURE support checking populate at a user level
        # assert u.populated == True

    def test_populating_a_user_with_failures(self, unload_users, u, dstore_n,
                                             cat_name):
        # Populate with failed datasets
        # Note: all datasets should be populated, even if the failure comes first
        nf = "failed_outcome"
        n = "ds0"
        m = "Failure case failed_outcome reached!"
        exc_m = f"Failed to populate dataset '{nf}' for user '{u.id}': {m}"
        df = u.add_dataset(
            nf,
            self.user_dataset_config_class(
                data_store=dstore_n,
                category=cat_name,
            ))
        d0 = u.add_dataset(
            n,
            self.user_dataset_config_class(
                data_store=dstore_n,
                category=cat_name,
            ))
        res = u.populate()
        assert bool(res) == False
        assert res.succeeded == False
        assert res.failed == True
        assert res.errored == False
        assert list(res.outcomes.keys()) == [nf, n]
        assert res.outcomes[nf].succeeded == False
        assert res.outcomes[nf].failed == True
        assert res.outcomes[nf].message == m
        assert res.outcomes[n].succeeded == True
        assert res.failed_datasets == [nf]
        assert list(res.failed_outcomes.keys()) == [nf]
        assert isinstance(res.failed_outcomes[nf], om.framework.Outcome)
        assert res.failed_outcomes[nf].failed == True
        assert res.failed_outcomes[nf].message == m
        assert df.populated == False
        assert d0.populated == True

        # Treat failures as an error
        d1 = u.add_dataset(
            "ds1",
            self.user_dataset_config_class(
                data_store=dstore_n,
                category=cat_name,
            ))
        with pytest.raises(RuntimeError, match=exc_m):
            u.populate(stop_on_failure=True)
        # Should have bailed before ds1 was ever tried
        assert d1.populated == False
        assert d1.populate_attempted == False

    def test_populating_a_user_with_errors(self, unload_users, u, dstore_n,
                                           cat_name):
        # Populate with an error
        ne = "exception_raised"
        n = "ds0"
        m = "Error case exception_raised reached!"
        exc_m = f"Encountered Exception 'NameError' with message: {m}\nWith traceback"
        de = u.add_dataset(
            ne,
            self.user_dataset_config_class(
                data_store=dstore_n,
                category=cat_name,
            ))
        d = u.add_dataset(
            n,
            self.user_dataset_config_class(
                data_store=dstore_n,
                category=cat_name,
            ))
        assert de.populated == False
        assert de.populate_attempted == False
        assert d.populated == False
        assert d.populate_attempted == False

        with pytest.raises(RuntimeError, match=exc_m):
            u.populate()
        assert de.populated == False
        assert de.populate_attempted == True
        assert d.populated == False
        assert d.populate_attempted == False

        # Populate with an error but continue on regardless
        res = u.populate(continue_on_error=True)
        assert bool(res) == False
        assert res.succeeded == False
        assert res.failed == False
        assert res.errored == True
        assert list(res.outcomes.keys()) == [ne, n]
        assert res.outcomes[ne].succeeded == False
        assert res.outcomes[ne].failed == False
        assert res.outcomes[ne].errored == True
        assert exc_m in res.outcomes[ne].message
        assert res.outcomes[n].succeeded == True
        assert res.failed_datasets == []
        assert res.errored_datasets == [ne]
        assert list(res.errored_outcomes.keys()) == [ne]
        assert isinstance(res.errored_outcomes[ne], om.framework.Outcome)
        assert res.errored_outcomes[ne].errored == True
        assert exc_m in res.errored_outcomes[ne].message
        assert de.populated == False
        assert d.populated == True

    def test_populating_with_errors_and_failures(self, unload_users, u,
                                                 dstore_n, cat_name):
        ne = "exception_raised"
        nf = "failed_outcome"
        n = "ds0"

        de = u.add_dataset(
            ne,
            self.user_dataset_config_class(
                data_store=dstore_n,
                category=cat_name,
            ))
        df = u.add_dataset(
            nf,
            self.user_dataset_config_class(
                data_store=dstore_n,
                category=cat_name,
            ))
        d = u.add_dataset(
            n,
            self.user_dataset_config_class(
                data_store=dstore_n,
                category=cat_name,
            ))

        # Populate with an error but continue on regardless
        res = u.populate(continue_on_error=True)
        assert bool(res) == False
        assert res.succeeded == False
        assert res.failed == True
        assert res.errored == True
        assert list(res.outcomes.keys()) == [ne, nf, n]

        assert res.outcomes[ne].succeeded == False
        assert res.outcomes[ne].failed == False
        assert res.outcomes[ne].errored == True
        assert res.outcomes[nf].succeeded == False
        assert res.outcomes[nf].failed == True
        assert res.outcomes[nf].errored == False

        assert res.outcomes[n].succeeded == True
        assert res.failed_datasets == [nf]
        assert res.errored_datasets == [ne]
        assert list(res.errored_outcomes.keys()) == [ne]
        assert list(res.failed_outcomes.keys()) == [nf]
        assert de.populated == False
        assert df.populated == False
        assert d.populated == True

    def test_populating_empty_users(self, unload_users, users):
        assert len(users) == 0
        pop_rtn = users.populate()
        assert isinstance(pop_rtn, self.pop_users_return_class)
        assert bool(pop_rtn) == True
        assert pop_rtn.succeeded == True
        assert pop_rtn.failed == False
        assert pop_rtn.errored == False
        assert pop_rtn.outcomes == {}
        assert pop_rtn.failed_datasets == {}
        assert pop_rtn.errored_datasets == {}
        assert pop_rtn.failed_outcomes == {}
        assert pop_rtn.errored_outcomes == {}

    def test_populating_non_applicable_datasets(self, unload_users, users, u):
        assert len(users) == 1
        pop_rtn = users.populate()
        assert isinstance(pop_rtn, self.pop_users_return_class)
        assert bool(pop_rtn) == True
        assert pop_rtn.succeeded == True
        assert pop_rtn.failed == False
        assert pop_rtn.errored == False
        assert list(pop_rtn.outcomes.keys()) == [u.id]
        assert isinstance(pop_rtn.outcomes[u.id], self.pop_user_return_class)
        assert pop_rtn.outcomes[u.id].outcomes == {}
        assert pop_rtn.failed_datasets == {}
        assert pop_rtn.errored_datasets == {}
        assert pop_rtn.failed_outcomes == {}
        assert pop_rtn.errored_outcomes == {}

    # @pytest.mark.skip
    def test_populating_users(self, unload_users, users, dstore_n, cat_name):
        n1 = "ds_1"
        users.add_dataset(
            n1,
            self.user_dataset_config_class(
                data_store=dstore_n,
                category=cat_name,
            ))

        u1 = self.user(1)
        u2 = self.user(2)
        assert u1.datasets[n1].populated == False
        assert u2.datasets[n1].populated == False

        pop_rtn = users.populate()
        assert isinstance(pop_rtn, self.pop_users_return_class)
        assert bool(pop_rtn) == True
        assert pop_rtn.succeeded == True
        assert pop_rtn.failed == False
        assert pop_rtn.errored == False
        assert list(pop_rtn.outcomes.keys()) == [u1.id, u2.id]
        assert isinstance(pop_rtn.outcomes[u1.id], self.pop_user_return_class)
        assert list(pop_rtn.outcomes[u1.id].outcomes.keys()) == [n1]
        assert pop_rtn.outcomes[u1.id].outcomes[n1].succeeded == True
        assert pop_rtn.failed_datasets == {}
        assert pop_rtn.errored_datasets == {}
        assert pop_rtn.failed_outcomes == {}
        assert pop_rtn.errored_outcomes == {}

        assert u1.datasets[n1].populated == True
        assert u2.datasets[n1].populated == True

        # Add a user and populate again
        # Also add a dataset just for fun
        n2 = "ds_2"
        users.add_dataset(
            n2,
            self.user_dataset_config_class(
                data_store=dstore_n,
                category=cat_name,
            ))
        u3 = self.user(3)
        pop_rtn = users.populate()
        assert bool(pop_rtn) == True
        assert pop_rtn.succeeded == True
        assert u3.datasets[n1].populated == True
        assert u3.datasets[n2].populated == True
        assert list(pop_rtn.outcomes.keys()) == [u1.id, u2.id, u3.id]
        assert pop_rtn.outcomes[u1.id].outcomes == {n1: None}
        assert pop_rtn.outcomes[u2.id].outcomes == {n1: None}
        assert list(pop_rtn.outcomes[u3.id].outcomes.keys()) == [n1, n2]
        assert pop_rtn.outcomes[u3.id].outcomes[n1].succeeded == True
        assert pop_rtn.outcomes[u3.id].outcomes[n2].succeeded == True

        # Try a repopulate
        pop_rtn = users.populate(repopulate=True)
        assert bool(pop_rtn) == True
        assert pop_rtn.succeeded == True
        assert list(pop_rtn.outcomes[u1.id].outcomes.keys()) == [n1]
        assert list(pop_rtn.outcomes[u2.id].outcomes.keys()) == [n1]
        assert list(pop_rtn.outcomes[u3.id].outcomes.keys()) == [n1, n2]

    def test_populating_users_with_failures(self, unload_users, users,
                                            dstore_n, cat_name):
        nf = "fail_outcome_for_u_fail"
        m = f"Failure case fail_outcome_for_u_fail for user {self.to_user_id('fail')} reached!"
        exc_m = f"Failed to populate dataset 'fail_outcome_for_u_fail' for user \'{self.to_user_id('fail')}\': {m}"
        users.add_dataset(
            nf,
            self.user_dataset_config_class(
                data_store=dstore_n,
                category=cat_name,
            ))

        u1 = self.user(1)
        uf = self.user("fail")
        u2 = self.user(2)
        assert u1.datasets[nf].populated == False
        assert u1.datasets[nf].populate_attempted == False
        assert uf.datasets[nf].populated == False
        assert uf.datasets[nf].populate_attempted == False
        assert u2.datasets[nf].populated == False
        assert u2.datasets[nf].populate_attempted == False

        pop_rtn = users.populate()
        assert bool(pop_rtn) == False
        assert pop_rtn.succeeded == False
        assert pop_rtn.failed == True
        assert pop_rtn.errored == False
        assert list(pop_rtn.outcomes.keys()) == [u1.id, uf.id, u2.id]
        assert pop_rtn.outcomes[uf.id].outcomes[nf].succeeded == False
        assert pop_rtn.outcomes[uf.id].outcomes[nf].failed == True
        assert pop_rtn.outcomes[uf.id].outcomes[nf].errored == False
        assert pop_rtn.outcomes[uf.id].outcomes[nf].message == m
        assert pop_rtn.outcomes[u1.id].outcomes[nf].succeeded == True
        assert pop_rtn.outcomes[u2.id].outcomes[nf].succeeded == True
        assert pop_rtn.failed_datasets == {uf.id: [nf]}
        assert pop_rtn.failed_outcomes[uf.id][nf].message == m
        assert pop_rtn.errored_datasets == {}
        assert pop_rtn.errored_outcomes == {}
        assert u1.datasets[nf].populated == True
        assert u1.datasets[nf].populate_attempted == True
        assert uf.datasets[nf].populated == False
        assert uf.datasets[nf].populate_attempted == True
        assert u2.datasets[nf].populated == True
        assert u2.datasets[nf].populate_attempted == True

        with pytest.raises(RuntimeError, match=exc_m):
            users.populate(stop_on_failure=True)

    def test_populating_users_with_errors(self, unload_users, users, dstore_n,
                                          cat_name):
        ne = "raise_exception_for_u_error"
        m = f"Error case raise_exception_for_u_error for user {self.to_user_id('error')} reached!"
        exc_m = f"Encountered Exception 'NameError' with message: {m}\nWith traceback"

        users.add_dataset(
            ne,
            self.user_dataset_config_class(
                data_store=dstore_n,
                category=cat_name,
            ))
        u1 = self.user(1)
        ue = self.user('error')
        u2 = self.user(2)

        assert u1.datasets[ne].populated == False
        assert u1.datasets[ne].populate_attempted == False
        assert ue.datasets[ne].populated == False
        assert ue.datasets[ne].populate_attempted == False
        assert u2.datasets[ne].populated == False
        assert u2.datasets[ne].populate_attempted == False

        with pytest.raises(RuntimeError, match=exc_m):
            users.populate()
        assert u1.datasets[ne].populated == True
        assert u1.datasets[ne].populate_attempted == True
        assert ue.datasets[ne].populated == False
        assert ue.datasets[ne].populate_attempted == True
        assert u2.datasets[ne].populated == False
        assert u2.datasets[ne].populate_attempted == False

        # Populate with an error but continue on regardless
        pop_rtn = users.populate(continue_on_error=True)
        assert bool(pop_rtn) == False
        assert pop_rtn.succeeded == False
        assert pop_rtn.failed == False
        assert pop_rtn.errored == True
        assert list(pop_rtn.outcomes.keys()) == [u1.id, ue.id, u2.id]
        assert pop_rtn.outcomes[ue.id].outcomes[ne].succeeded == False
        assert pop_rtn.outcomes[ue.id].outcomes[ne].failed == False
        assert pop_rtn.outcomes[ue.id].outcomes[ne].errored == True
        assert exc_m in pop_rtn.outcomes[ue.id].outcomes[ne].message
        assert pop_rtn.outcomes[u1.id].outcomes[ne] == None
        assert pop_rtn.outcomes[u2.id].outcomes[ne].succeeded == True
        assert pop_rtn.failed_datasets == {}
        assert pop_rtn.failed_outcomes == {}
        assert pop_rtn.errored_datasets == {ue.id: [ne]}
        assert exc_m in pop_rtn.errored_outcomes[ue.id][ne].message
        assert u1.datasets[ne].populated == True
        assert u1.datasets[ne].populate_attempted == True
        assert ue.datasets[ne].populated == False
        assert ue.datasets[ne].populate_attempted == True
        assert u2.datasets[ne].populated == True
        assert u2.datasets[ne].populate_attempted == True

    def test_autopopulating_users(self, unload_users, users, dstore_n,
                                  cat_name):
        n = "ds0"
        users.add_dataset(
            n,
            self.user_dataset_config_class(
                data_store=dstore_n,
                category=cat_name,
                auto_populate=True,
            ))
        u = self.user()
        d = u.datasets[n]
        assert d.populated == True
        assert d.populate_attempted == True
        assert d.other["populated_from"] == self.DSPopUser.__qualname__
        assert d.other["pop-ed?"] == True

        n = "ds1"
        u.add_dataset(
            n,
            self.user_dataset_config_class(
                data_store=dstore_n,
                category=cat_name,
                auto_populate=True,
            ))
        d = u.datasets[n]
        assert d.populated == True
        assert d.populate_attempted == True
        assert d.other["populated_from"] == self.DSPopUser.__qualname__
        assert d.other["pop-ed?"] == True

    def test_errors_and_failures_during_autopopulate_raises_an_exception(
            self, unload_users, u, dstore_n, cat_name):
        ne = "exception_raised"
        with pytest.raises(
                RuntimeError,
                match=
                "Encountered Exception 'NameError' with message: Error case exception_raised reached!"
        ):
            u.add_dataset(
                ne,
                self.user_dataset_config_class(
                    data_store=dstore_n,
                    category=cat_name,
                    auto_populate=True,
                ))

        # Dataset should exists but not be populated
        d = u.datasets[ne]
        assert d.populated == False
        assert d.populate_attempted == True

        nf = "failed_outcome"
        with pytest.raises(RuntimeError,
                           match="Failure case failed_outcome reached!"):
            u.add_dataset(
                nf,
                self.user_dataset_config_class(
                    data_store=dstore_n,
                    category=cat_name,
                    auto_populate=True,
                ))
        d = u.datasets[nf]
        assert d.populated == False
        assert d.populate_attempted == True

    # TEST_NEEDED
    @pytest.mark.skip
    def test_recovery_options_when_errors_occur_autopopulating(
            self, unload_users, users):
        raise NotImplementedError()

    def test_adding_dataset_from_dict(self, unload_users, users, dstore_n,
                                      cat_name, ddk):
        dset = {
            "data_store": dstore_n,
            "category": cat_name,
            "auto_populate": False
        }
        users.add_dataset("n0", dset)
        users.register_dataset("r0", dset)
        assert list(users.datasets.keys()) == [ddk, "n0", "r0"]

        u = self.user()
        u.add_dataset("n1", dset)
        u.add_dataset("r1", dset)
        assert list(u.datasets.keys()) == [ddk, "n0", "r0", "n1", "r1"]

    def test_populating_an_ldap_dataset(self, unload_users, users, cat,
                                        cat_name):
        from tests.utils.test_ldap import INIT_PARAMS
        params = copy.deepcopy(INIT_PARAMS)
        params[-2] = {
            "data_id": "uid",
            "mapping": {
                "email": "mail",
                "last_name": "sn",
                "full_name": "cn"
            }
        }
        cat.add(
            "test_ldap",
            om.utils.ldap.LDAP,
            ["test_ldap", *params],
        )

        users.add_dataset(
            "forumsys",
            self.user_dataset_config_class(
                data_store="test_ldap",
                category=cat_name,
            ))
        # By the config, dataset is not populated automoatically
        u = users.add("euler")
        d = u.datasets["forumsys"]
        assert d.populated == False

        # Set the username before populating
        d.username = "euler"

        # Populate
        outcome = d.populate()
        assert outcome.succeeded == True
        assert d.populated == True

        # Check some items
        assert d.email == "euler@ldap.forumsys.com"
        assert d.last_name == "Euler"
        assert d.username == "euler"

        # Check that other data fields were populated
        assert d.data_store["full_name"] == "Leonhard Euler"

    # TEST_NEEDED
    @pytest.mark.skip
    def test_populating_all(self):
        raise NotImplementedError()

    # TEST_NEEDED
    @pytest.mark.skip
    def test_populating_with_required(self):
        raise NotImplementedError()

    # TEST_NEEDED
    @pytest.mark.skip
    def test_populating_with_custom_filters(self):
        raise NotImplementedError()

    # TEST_NEEDED
    @pytest.mark.skip
    def test_populating_from_empty_mapping(self):
        raise NotImplementedError()

    # TEST_NEEDED
    @pytest.mark.skip
    def test_populating_with_custom_attribute_fields(self):
        raise NotImplementedError()

    # TODO needs to be part of larger frontend tests
    def test_populating_dummy_datasets_workout(self, fresh_frontend,
                                               unload_users, users, cat_name):
        cat = fresh_frontend.data_stores.add_category(cat_name)
        n = "pop_test_ds"
        cat.add(n, self.DSPopUser)
        cat.add("pop_test_ds_child", self.DSPopUserChild)
        cat.add("pop_test_ds_ovr", self.DSPopUserChildOverrideMethod)
        cat.add("pop_test_ds_ovr_no_dec",
                self.DSPopUserChildOverrideMethodNoDecorator)
        cat.add("pop_test_ds_new_method", self.DSPopUserChildNewMethod)

        users.add_dataset(
            "popped_test_ds",
            self.user_dataset_config_class(
                data_store=n,
                category=cat_name,
                auto_populate=True,
            ))
        users.add_dataset(
            "popped_test_ds_child",
            self.user_dataset_config_class(
                data_store="pop_test_ds_child",
                category=cat_name,
                auto_populate=True,
            ))
        users.add_dataset(
            "popped_test_ds_ovr",
            self.user_dataset_config_class(
                data_store="pop_test_ds_ovr",
                category=cat_name,
                auto_populate=True,
            ))
        users.add_dataset(
            "popped_test_ds_ovr_no_dec",
            self.user_dataset_config_class(
                data_store="pop_test_ds_ovr_no_dec",
                category=cat_name,
                auto_populate=True,
            ))
        users.add_dataset(
            "popped_test_ds_new_method",
            self.user_dataset_config_class(
                data_store="pop_test_ds_new_method",
                category=cat_name,
                auto_populate=True,
            ))
        u = self.user()

        assert u.datasets["popped_test_ds"].other[
            "populated_from"] == self.DSPopUser.__qualname__
        assert u.datasets["popped_test_ds_child"].other[
            "populated_from"] == self.DSPopUserChild.__qualname__

        assert u.datasets["popped_test_ds_ovr"].other[
            "populated_from"] == self.DSPopUserChildOverrideMethod.__qualname__
        assert u.datasets["popped_test_ds_ovr"].other[
            "populated_from_2"] == "override"
        assert u.datasets["popped_test_ds_ovr_no_dec"].other[
            "populated_from"] == self.DSPopUserChildOverrideMethodNoDecorator.__qualname__
        assert "populated_from_2" not in u.datasets[
            "popped_test_ds_ovr_no_dec"].other
        assert "populated_from" not in u.datasets[
            "popped_test_ds_new_method"].other
        assert u.datasets["popped_test_ds_new_method"].other[
            "populated_from_2"] == "new_method"
