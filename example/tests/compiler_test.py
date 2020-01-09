import origen
import pytest

def boot_falcon():
    return origen.app.instantiate_dut("dut.falcon") if origen.dut is None else origen.dut

def test_compiler_inits():
    boot_falcon()
    assert isinstance(origen.app.compiler, origen.compiler.Compiler) == True
    assert origen.app.compiler.stack == []
    assert origen.app.compiler.renders == []
    assert origen.app.compiler.output_files == []
    assert isinstance(origen.app.compiler.syntax, origen.compiler.Compiler.MakoSyntax) == True
    
def test_compiler_renders_text(capfd):
    origen.app.compile("hello, ${name}!", name='jack')
    assert len(origen.app.compiler.renders) == 1
    assert len(origen.app.compiler.stack) == 0
    assert origen.app.compiler.renders[0] == 'hello, jack!'
    origen.app.compile("${name} is a ${adj} boy!", name='jack', adj='good')
    assert len(origen.app.compiler.renders) == 2
    assert len(origen.app.compiler.stack) == 0
    assert origen.app.compiler.renders[1] == "jack is a good boy!"

   