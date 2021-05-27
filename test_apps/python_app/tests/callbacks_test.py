import origen, pytest
from tests.shared import clean_eagle, clean_bald_eagle


class TestCallbacks:
    before_tester_reset_called = False
    after_tester_reset_called = False

    @pytest.mark.xfail
    def test_available_callbacks(self):
        origen.callbacks.available == [
            # "on_app_init",
            # "on_target_loaded"
        ]

    def test_before_after_tester_reset(self):
        @origen.callbacks.listen_for("before_tester_reset")
        def pytest_before_tester_reset():
            TestCallbacks.before_tester_reset_called = True

        @origen.callbacks.listen_for("after_tester_reset")
        def pytest_after_tester_reset():
            TestCallbacks.after_tester_reset_called = True

        assert TestCallbacks.before_tester_reset_called == False
        assert TestCallbacks.after_tester_reset_called == False
        resets = origen.app.__class__.tester_resets
        origen.tester.reset()

        assert TestCallbacks.before_tester_reset_called == True
        assert TestCallbacks.after_tester_reset_called == True
        # This will increase from the callback at in the example application
        assert origen.app.__class__.tester_resets == resets + 1


class TestStartupShutdownCallbacks:
    def test_toplevel_startups_and_shutdowns_occur(self, clean_eagle):
        origen.callbacks.emit("toplevel__startup")
        assert origen.dut.startups_called == 1
        assert origen.dut.startup_source == "eagle"

        origen.callbacks.emit("toplevel__shutdown")
        assert origen.dut.shutdowns_called == 1
        assert origen.dut.shutdown_source == "eagle"

    def test_toplevel_startup_can_override(self, clean_bald_eagle):
        origen.callbacks.emit("toplevel__startup")
        assert origen.dut.startups_called == 1
        assert origen.dut.startup_source == "bald_eagle"

        origen.callbacks.emit("toplevel__shutdown")
        assert origen.dut.shutdowns_called == 1
        assert origen.dut.shutdown_source == "eagle"

    def test_startups_and_shutdowns_occur(self, clean_bald_eagle):
        origen.callbacks.emit("toplevel__startup")
        blk = origen.dut.generic_clk_ctrl
        assert len(blk.callbacks) == 0

        origen.callbacks.emit("controller__startup")
        assert blk.callbacks == [
            "base__enable_cc_called",
            "base__wait_for_enable_called",
        ]

        origen.callbacks.emit("controller__shutdown")
        assert blk.callbacks == [
            "base__enable_cc_called", "base__wait_for_enable_called",
            "base__disable_cc_called", "base__wait_for_disable_called"
        ]

    def test_startups_and_shutdowns_can_be_overridden(self, clean_bald_eagle):
        origen.callbacks.emit("toplevel__startup")
        blk = origen.dut.fast_clk_ctrl
        assert len(blk.callbacks) == 0
        origen.callbacks.emit("controller__startup")
        assert blk.callbacks == [
            "fast_clk_ctrl__enable_cc_called",
            "fast_clk_ctrl__check_enabled_called",
            "base__wait_for_enable_called",
        ]

        origen.callbacks.emit("controller__shutdown")
        assert blk.callbacks == [
            "fast_clk_ctrl__enable_cc_called",
            "fast_clk_ctrl__check_enabled_called",
            "base__wait_for_enable_called",

            # Note the order here
            "fast_clk_ctrl__wait_for_disable_called",
            "base__disable_cc_called"
        ]
