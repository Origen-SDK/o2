# FOR_PR clean up
# Use the dev version but actual tests should be done through 'eval'.
import sys, pathlib
p = pathlib.Path(__file__).parent.parent.parent.joinpath("python/origen")
sys.path.append(str(p))
sys.path.append(str(p.parent.joinpath("origen_metal")))

import origen, _origen, origen_metal

import pytest, pip, jinja2, shutil, subprocess
from origen.helpers.regressions.cli import CLI
from types import ModuleType
from pathlib import Path, PosixPath, WindowsPath

PyProjectSrc = _origen.infrastructure.pyproject.PyProjectSrc
toml = "pyproject.toml"
lockfile = "poetry.lock"
no_workspace_test_dir = pathlib.Path(__file__).parent
o2_root = no_workspace_test_dir.parent.parent
debug_cli_loc = o2_root.joinpath(f"rust/origen/target/debug/origen{'.exe' if origen.running_on_windows else ''}")
eval_scripts_dir = no_workspace_test_dir.joinpath("eval_scripts")
status_eval_script = eval_scripts_dir.joinpath("print_status.py")
pl_names_eval_script = eval_scripts_dir.joinpath("print_pl_names.py")

# Assume pip is installed in 'site-packages'
site_packages_dir =  pathlib.Path(pip.__file__).parent.parent

class T_InvocationBaseTests(CLI):
    templates_dir = no_workspace_test_dir.joinpath("templates")
    templates_out_dir = templates_dir.joinpath("output")
    debug_cli_loc = debug_cli_loc
    PyProjectSrc = PyProjectSrc

    @classmethod
    def setup(cls):
        cls.set_params()
        if cls.target_pyproj_dir:
            cls.target_pyproj_toml = cls.target_pyproj_dir.joinpath(toml)
            cls.target_poetry_lock = cls.target_pyproj_dir.joinpath(lockfile)
        else:
            cls.target_pyproj_toml = None
            cls.target_poetry_lock = None

        if not hasattr(cls, "file_based_evals"):
            cls.file_based_evals = False
        cls.cli_location = cls.cli_dir.joinpath(f"origen{'.exe' if origen.running_on_windows else ''}")

    @property
    def header(self):
        return "--Origen Eval--"

    def eval_and_parse(self, code):
        # out = CLI.global_cmds.eval.run(code, "-vv", run_opts={"return_details": True})
        if isinstance(code, str):
            code = [code]
        print(code)
        # out = CLI.global_cmds.eval.run(*code, run_opts={"return_details": True, "check": False})
        out = CLI.global_cmds.eval.run(*code)
        print(out)
        out = out.split("\n")
        idx = out.index(self.header)
        return eval(out[idx+1])

    def get_status(self):
        if self.file_based_evals:
            return self.eval_and_parse(["-f", status_eval_script])
        else:
            return self.eval_and_parse(f"print('{self.header}'); print(origen.status)")

    def test_invocation_from_pytest(self):
        assert origen.status["pyproject"] is None
        assert origen.status["invocation"] is None

    def test_pyproject_and_invocation_set(self):
        # code = f"print('{self.header}'); print(origen.status)"
        # code = r"print\(\\\"Origen\ Status:\\\"\) print\(origen.status\)"
        status = self.get_status()
        print(status)
        assert status["pyproject"] == self.target_pyproj_toml
        assert status["invocation"] == self.invocation

    def test_cli_location(self):
        # code = f"print('{self.header}'); print(origen.status)"
        # status = self.eval_and_parse(code)
        status = self.get_status()
        assert status['cli_location'] == self.cli_location

class T_InvocationEnv(T_InvocationBaseTests):
    @classmethod
    def setup_method(cls):
        super().setup()
        # cls.set_params()
        if cls.target_pyproj_dir:
            cls._pyproj_src_file = cls.gen_pyproj()
        if not hasattr(cls, "move_pyproject"):
            if cls.target_pyproj_dir:
                cls.move_pyproject = True
            else:
                cls.move_pyproject = False
        # TODO clear any existing pyproject/poetry.locks ?
        # cls._pyproj_lock = cls._pyproj_file.parent.joinpath("poetry.lock")
        # for d in origen_exe_loc.parents:
        #     f = d.joinpath(toml)
        #     if f.exists():
        #         target = f.parent.joinpath(f"{toml}.origen.regressions")
        #         print(f"Temporarily moving pyproject {f} to {target}")
        #         f.rename(target)
        if cls.move_pyproject:
            target = cls.target_pyproj_dir.joinpath(toml)
            print(f"Moving pyproject {cls._pyproj_src_file} to {target}")
            shutil.copy(cls._pyproj_src_file, target)
        if cls.target_pyproj_dir:
            subprocess.run(["poetry", "--version"], check=True, cwd=cls.target_pyproj_dir)
            subprocess.run(["poetry", "install"], check=True, cwd=cls.target_pyproj_dir)

    @classmethod
    def teardown_method(cls):
        if cls.move_pyproject:
            print(f"Cleaning pyproject and lockfile {cls.target_pyproj_toml}, {cls.target_poetry_lock}")
            cls.target_pyproj_toml.unlink()
            cls.target_poetry_lock.unlink()

    @classmethod
    def gen_pyproj(cls):
        env = jinja2.Environment(
            loader=jinja2.FileSystemLoader(searchpath="./templates")
        )
        t = env.get_template("pyproject.toml")
        cls.templates_out_dir.mkdir(exist_ok=True)
        pyproj = cls.templates_out_dir.joinpath(f"{cls.__name__}.{toml}")
        with open(pyproj, "w") as f:
            f.write(t.render(local_origen=cls.local_origen, name=cls.__name__, o2_root=o2_root))
        return pyproj

    # TEST_NEEDED invocation origen/metal package locations
    # class TestBareEnv(CLI):
    # @pytest.mark.parameterize(
    #         [origen, origen._origen, origen_metal, origen._origen_metal],
    #         ids=["origen", "_origen", "origen_metal", "_origen_metal"]
    # )
    # def test_origen_pkgs(self, mod, ext, ):
    #     # assert origen.__file__ == ?
    #     # assert origen._origen.__file__ == 
    #     # assert origen_metal.__file__ == ?
    #     # TEST_NEEDED CLI not sure why origen_metal._origen_metal has no filename
    #     # Just assert its a module fo now.
    #     # if id == "_origen_metal":
    #     assert isinstance(origen_metal._origen_metal, ModuleType)

    def get_plugin_names(self):
        if self.file_based_evals:
            return self.eval_and_parse(["-f", pl_names_eval_script])
        else:
            return self.eval_and_parse(f"print('{self.header}'); print(list(origen.plugins.keys()))")

    @pytest.mark.skip
    def test_origen_h(self):
        fail

    def test_plugins(self):
        # code = f"print('{self.header}'); print(list(origen.plugins.keys()))"
        pls = self.get_plugin_names()
        # pls = self.eval_and_parse(code)
        if self.has_pls:
            # TODO consistent plugin loading
            assert set(pls) == {'pl_ext_cmds', 'test_apps_shared_test_helpers', 'python_plugin'}
        else:
            assert pls == []
