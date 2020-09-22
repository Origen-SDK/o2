import origen
import pytest
from tests.shared import *

#@pytest.fixture(autouse=True)
#def run_around_tests():
#    global dut
#    # Code that will run before each test
#    instantiate_dut("dut.falcon")
#    dut = origen.dut
#    yield
#    # Code that will run after each test


def test_dut_from_a_plugin():
    origen.target.load("dut/hawk")
    assert "python_plugin.blocks.dut.derivatives.hawk.controller.Controller" in str(
        type(origen.dut))
    assert (origen.dut.hawk_reg1)


def test_block_from_a_plugin():
    origen.target.load("dut/falcon")
    assert (origen.dut.dac)
    assert "python_plugin.blocks.dac.controller.Controller" in str(
        type(origen.dut.dac))
    assert (origen.dut.dac.my_dac_reg1)
    assert (origen.dut.dac.bist.bist_reg1)


def test_plugin_app_access():
    assert (origen.has_plugin("python_plugin"))
    assert origen.plugin("python_plugin").name == "python_plugin"
