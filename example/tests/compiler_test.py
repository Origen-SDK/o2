import origen
import pytest
import pathlib
import os
import stat
from os import access, W_OK, X_OK, R_OK

def boot_falcon():
    return origen.app.instantiate_dut("dut.falcon") if origen.dut is None else origen.dut

def test_compiler_inits():
    boot_falcon()
    assert isinstance(origen.app.compiler, origen.compiler.Compiler) == True
    assert origen.app.compiler.stack == []
    assert origen.app.compiler.renders == []
    assert origen.app.compiler.output_files == []
    assert isinstance(origen.app.compiler.syntax, origen.compiler.Compiler.MakoSyntax) == True
    assert str(origen.app.compiler.templates_dir()) == "/mnt/c/o2/compiler/example/example/templates"

def test_compiler_understands_global_context():
    assert origen.app.compile("dut's name is ${dut.name}").renders[0] == "dut's name is dut"
    assert origen.app.compile("tester is ${tester}").renders[1] == "tester is None"
    assert origen.app.compile("origen version is of type '${type(origen.version)}'").renders[2] == "origen version is of type '<class 'str'>'"

def test_compiler_can_clear_itself():
    origen.app.compiler.clear()
    assert origen.app.compiler.stack == []
    assert origen.app.compiler.renders == []
    assert origen.app.compiler.output_files == []

def test_compiler_renders_text():
    origen.app.compile("hello, ${name}!", name='jack')
    assert len(origen.app.compiler.renders) == 1
    assert len(origen.app.compiler.stack) == 0
    assert origen.app.compiler.renders[0] == 'hello, jack!'
    origen.app.compile("${name} is a ${adj} boy!", name='jack', adj='good')
    assert len(origen.app.compiler.renders) == 2
    assert len(origen.app.compiler.stack) == 0
    assert origen.app.compiler.renders[1] == "jack is a good boy!"
    assert origen.app.compiler.renders[-1] == origen.app.compiler.last_render()
    
def test_compiler_renders_files():
    templates_dir = f"{origen.root}/../python/templates"
    origen.app.compile('dut_info.txt.mako', templates_dir=templates_dir)
    assert len(origen.app.compiler.stack) == 0
    assert len(origen.app.compiler.output_files) == 1
    compiled_file = origen.app.compiler.output_files[0]
    compiled_file_status = os.stat(compiled_file)
    assert isinstance(compiled_file, pathlib.PurePath) == True
    assert compiled_file.exists() == True
    assert access(compiled_file, R_OK) == True
    # Check file permissions
    assert bool(compiled_file_status.st_mode & stat.S_IRUSR) == True
    assert bool(compiled_file_status.st_mode & stat.S_IWUSR) == True
    assert bool(compiled_file_status.st_mode & stat.S_IWUSR) == True
    assert compiled_file_status.st_size == 33

   