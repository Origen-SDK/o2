import origen
import pytest
from tests.shared import *
from origen_metal.utils.version import Version, pep440

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

def test_plugin_versions():
    pyplug_ver = origen.plugin('python_plugin').version
    assert pyplug_ver.__class__ == Version
    assert str(pyplug_ver) == str(pep440('0.1.0'))

    test_apps_pl_ver = origen.plugin('test_apps_shared_test_helpers').version
    assert test_apps_pl_ver.__class__ == Version
    assert str(test_apps_pl_ver) == str(pep440('0.1.1'))