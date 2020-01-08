import origen
import pytest

def boot_falcon():
    return origen.app.instantiate_dut("dut.falcon") if origen.dut is None else origen.dut

def test_compiler(capfd):
    boot_falcon()
    compiler = origen.app.compile("file1", "file2")
    assert isinstance(compiler, origen.compiler.Compiler) == True
    out, err = capfd.readouterr()
    assert out == 'Added files to the compiler stack on init:\n  file1\n  file2\nRunning the compiler with these options:\n'
   