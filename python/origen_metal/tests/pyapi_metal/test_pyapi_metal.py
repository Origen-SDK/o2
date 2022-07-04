'''
    Tests for internal PyAPI features and/or helper functions that are easier or more applicable to 
    Python-side testing than Rust
'''

import pytest
import origen_metal._origen_metal as om

class TestContextManagerWrapper:
    @property
    def get_test_mod(self):
        return om.__test__._helpers.contextlib

    @pytest.fixture
    def tm(self):
        return self.get_test_mod

    @property
    def get_test_class(self):
        return om.__test__._helpers.contextlib.TestClass

    @pytest.fixture
    def tc(self):
        return self.get_test_class

    def test_context_manager_function(self, tm):
        l = ["hi"]
        m = ["hello"]
        assert tm.testing_context_wrapped_function == False
        with tm.context_wrapped_function(l) as t:
            m.append("from_context")
            assert tm.testing_context_wrapped_function == True
            assert t == "test_wrapped_function"
            assert l == ["hi", "added_perm", "added_temp"]
        assert m == ["hello", "from_context"]
        assert l == ["hi", "added_perm"]
        assert tm.testing_context_wrapped_function == False

    def test_context_manager_function_no_scope(self, tm):
        l = ["no_scope"]
        assert tm.testing_context_wrapped_function_no_scope == False
        with tm.context_wrapped_function_no_scope(l) as t:
            assert tm.testing_context_wrapped_function_no_scope == True
            assert t == "test_wrapped_function_no_scope"
            assert l == ["no_scope", "added_perm", "added_temp"]
        assert l == ["no_scope", "added_perm"]
        assert tm.testing_context_wrapped_function_no_scope == False

    def test_context_manager_class_method(self, tm):
        tc = tm.TestClass()
        assert tm.testing_context_wrapped_cls_method == False
        with tc.context_wrapped_cls_method() as (t, count):
            assert tm.testing_context_wrapped_cls_method == True
            assert t == "test_wrapped_cls_method"
            assert count == 1
        assert tm.testing_context_wrapped_cls_method == False

        with tm.TestClass.context_wrapped_cls_method() as (t, count):
            assert t == "test_wrapped_cls_method"
            assert count == 2

    def test_context_manager_instance_method(self, tm):
        tc1 = tm.TestClass()
        tc2 = tm.TestClass()
        assert tm.testing_context_wrapped_ins_method == False
        with tc1.context_wrapped_ins_method() as (t, count):
            assert tm.testing_context_wrapped_ins_method == True
            assert t == "test_wrapped_ins_method"
            assert count == 1
        assert tm.testing_context_wrapped_ins_method == False

        with tc2.context_wrapped_ins_method() as (t, count):
            assert t == "test_wrapped_ins_method"
            assert count == 1

        with tc1.context_wrapped_ins_method() as (t, count):
            assert t == "test_wrapped_ins_method"
            assert count == 2

        with tc1.context_wrapped_ins_method() as (t, count):
            assert t == "test_wrapped_ins_method"
            assert count == 3

        with tc2.context_wrapped_ins_method() as (t, count):
            assert t == "test_wrapped_ins_method"
            assert count == 2

    def test_context_manager_recovers_from_errors(self, tm):
        l = ["error_test"]
        assert tm.testing_context_wrapped_function == False
        with pytest.raises(AssertionError, match="0 == 1"):
            with tm.context_wrapped_function(l) as t:
                assert tm.testing_context_wrapped_function == True
                assert t == "test_wrapped_function"
                assert l == ["error_test", "added_perm", "added_temp"]
                assert 0 == 1
        assert l == ["error_test", "added_perm", "Found exception for error test: assert 0 == 1"]
        assert tm.testing_context_wrapped_function == False

    # TODO add some additional robustness tests
    # def test_context_manager_no_args(self):
    #     fail
    # def test_context_manager_two_args(self):
    #     fail
    # def test_context_manager_multiple_args(self):
    #     fail
    # def test_context_manager_kwargs(self):
    #     fail
    # def test_context_manager_full(self):
    #     # "(arg, *args, **kwargs)"
    #     fail
