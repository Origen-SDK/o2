from operator import truediv
import pytest, importlib, subprocess
from origen_metal._origen_metal import __test__
from origen_metal.utils.revision_control.supported import Git
from origen_metal import utils
from origen_metal.frontend import DataStoreView
from tests.framework.users.shared import clean_users, get_users


def test_modules_revision_control_and_rc_aliased():
    assert utils.revision_control == utils.rc


def test_git_can_instantiated():
    driver = Git({"local": "./", "remote": "test.git"})
    assert isinstance(driver, Git)
    assert driver.system() == "Git"


def test_pyapi_internal_git_path_is_valid():
    git_mod = importlib.import_module(__test__.python_git_mod_path())
    assert git_mod.Git == Git


class TestGitAsDataStore(DataStoreView):
    ''' Git's only data store feature is populating users'''
    def parameterize(self):
        return {
            "init_args": [{
                "local": "../../",
                "remote": "test.git"
            }],
        }

    @property
    def data_store_class(self):
        return Git

    def test_populating_users(self):
        n = "test_git_pop"

        cmd = ['git', 'config', 'user.name']
        r = subprocess.run(cmd, capture_output=True)
        if r.returncode == 0:
            git_name = r.stdout.decode("utf-8").strip()
        elif r.returncode == 1:
            git_name = "origen"
            subprocess.run(
                ["git", "config", "user.name", git_name],
                capture_output=True,
                check=True
            )
        else:
            raise RuntimeError(f"Failed to run command: {cmd}: {r}")

        cmd = ['git', 'config', 'user.email']
        r = subprocess.run(cmd, capture_output=True)
        if r.returncode == 0:
            git_email = r.stdout.decode("utf-8").strip()
        elif r.returncode == 1:
            git_email = "o2@origen.com"
            subprocess.run(
                ["git", "config", "user.email", git_email],
                capture_output=True,
                check=True
            )
        else:
            raise RuntimeError(f"Failed to run command: {cmd}: {r}")

        origen_repo_name = 'origenrepo'
        origen_global_name = 'origenglobal'
        origen_repo = f'user.{origen_repo_name}'
        origen_global = f'user.{origen_global_name}'
        origen_repo_val = 'test_repo'
        origen_global_val = 'test_global'
        try:
            subprocess.run(['git', 'config', origen_repo, origen_repo_val],
                           capture_output=True,
                           check=True,
                           shell=True)
            subprocess.run([
                'git', 'config', '--global', origen_global, origen_global_val
            ],
                           capture_output=True,
                           check=True,
                           shell=True)

            # This one should be overwritten by the repo's setting
            subprocess.run(
                ['git', 'config', '--global', origen_repo, origen_global_val],
                capture_output=True,
                check=True,
                shell=True)

            clean_users()
            get_users().add_dataset("git", {
                "data_store": self.ds_test_name,
                "category": self.cat_test_name
            })
            u = get_users().add(n)
            ds = u.datasets["git"]
            ds.populate()
            assert ds.username == git_name
            assert ds.email == git_email
            assert ds.display_name == git_name
            assert ds.other[origen_repo_name] == origen_repo_val
            assert ds.other[origen_global_name] == origen_global_val
        finally:
            subprocess.run(['git', 'config', '--unset', origen_repo],
                           capture_output=True,
                           check=True,
                           shell=True)
            subprocess.run(
                ['git', 'config', '--global', '--unset', origen_global],
                capture_output=True,
                check=True,
                shell=True)
