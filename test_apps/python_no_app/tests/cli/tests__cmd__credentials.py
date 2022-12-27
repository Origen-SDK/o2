# FOR_PR clean up
import pytest, origen, origen_metal
from .shared import CLICommon

class T_Credentials(CLICommon):
    @pytest.mark.skip
    def test_set_passwords(self, monkeypatch):
        # import io
        # monkeypatch.setattr('sys.stdin', io.StringIO('test_pw_updated'))

        u = origen.current_user
        assert set(u.datasets.keys()) == {
            "test", "backup", "dummy_ldap_ds", "test2", "git"
        }
        assert u.data_lookup_hierarchy == ["test", "backup"]
        u.datasets["dummy_ldap_ds"].should_validate_password = False

        u.datasets["test"].password = "test_pw"
        u.datasets["backup"].password = "backup_pw"
        u.datasets["dummy_ldap_ds"].password = "dummy_ldap_ds_pw"
        u.datasets["test2"].password = "test2_pw"
        u.datasets["git"].password = "git_pw"

        assert u.password == "test_pw"
        assert u.datasets["test"].password == "test_pw"
        assert u.datasets["backup"].password == "backup_pw"
        assert u.datasets["dummy_ldap_ds"].password == "dummy_ldap_ds_pw"
        assert u.datasets["test2"].password == "test2_pw"
        assert u.datasets["git"].password == "git_pw"

        self.global_cmds.cred.run(["set", "test_pw_updated"])

        self.global_cmds.cred.run(["set", "--dataset", "git" , "git_pw_updated", "--dataset", "test2", "test2_pw_updated"])
        # import subprocess
        # proc = subprocess.Popen(["origen", "credentials", "set"], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        # proc.stdin.flush()
        # proc.stdout.flush()
        # print(proc.stdout.readline().decode("UTF-8").strip())
        # proc.stdin.write(("test_pw_updated" + "\n").encode('UTF-8'))
        # proc.stdin.flush()
        # print(proc.stdout.readline().decode("UTF-8").strip())

        assert u.password == "test_pw_updated"

    @pytest.mark.skip
    def test_verify_password(self):
        fail

    def test_clearing_passwords(self):
        u = origen.current_user
        assert set(u.datasets.keys()) == {
            "test", "backup", "dummy_ldap_ds", "test2", "git"
        }
        assert u.data_lookup_hierarchy == ["test", "backup"]
        u.datasets["dummy_ldap_ds"].should_validate_password = False

        u.datasets["test"].password = "test_pw"
        u.datasets["backup"].password = "backup_pw"
        u.datasets["dummy_ldap_ds"].password = "dummy_ldap_ds_pw"
        u.datasets["test2"].password = "test2_pw"
        u.datasets["git"].password = "git_pw"

        assert u.password == "test_pw"
        assert u.datasets["test"].password == "test_pw"
        assert u.datasets["backup"].password == "backup_pw"
        assert u.datasets["dummy_ldap_ds"].password == "dummy_ldap_ds_pw"
        assert u.datasets["test2"].password == "test2_pw"
        assert u.datasets["git"].password == "git_pw"
        # import time
        # time.sleep(5)

        # Should only clear the 'test' pw
        # TODO need to clear all hierarchy
        # origen_metal.users.unload()
        # origen_metal.sessions.unload()
        # origen_metal.frontend.reset()
        out = self.global_cmds.creds.run("clear")
        # import os
        # os.remove(r"C:\Users\nxa13790\Documents\origen\o2_2nd\test_apps\python_no_app\.o2\.session\__user__nxa13790__\__user__")
        # import time
        # time.sleep(5)
        assert "Clearing cached password for topmost dataset..." in out
        origen_metal.sessions.unload()
        origen_metal.users.unload()
        origen._origen.boot_users()
        u.datasets["dummy_ldap_ds"].should_validate_password = False
        u = origen.current_user
        # origen.boot_users()
        # origen.users.remove(u.id)
        # origen.users.lookup_current_id(update_current=True)
        # u.session.refresh()
        # print(u.password)

        # TODO update this with "prompt=False" option or something better than expecting an error
        prompt_err = "Error encountered prompting for password: The handle is invalid"
        with pytest.raises(RuntimeError, match=prompt_err):
            assert u.password is None
        with pytest.raises(RuntimeError, match=prompt_err):
            assert u.datasets["test"].password is None
        with pytest.raises(RuntimeError, match=prompt_err):
            assert u.datasets["backup"].password == None
        assert u.datasets["dummy_ldap_ds"].password == "dummy_ldap_ds_pw"
        assert u.datasets["test2"].password == "test2_pw"
        assert u.datasets["git"].password == "git_pw"

        out = self.global_cmds.creds.run("clear", "--datasets", "git", "test2")
        origen_metal.sessions.unload()
        origen_metal.users.unload()
        origen._origen.boot_users()
        u.datasets["dummy_ldap_ds"].should_validate_password = False
        u = origen.current_user

        assert "Clearing cached password for dataset 'git'" in out
        assert "Clearing cached password for dataset 'test2'" in out
        with pytest.raises(RuntimeError, match=prompt_err):
            assert u.datasets["test"].password is None
        with pytest.raises(RuntimeError, match=prompt_err):
            assert u.datasets["backup"].password is None
        assert u.datasets["dummy_ldap_ds"].password == "dummy_ldap_ds_pw"
        with pytest.raises(RuntimeError, match=prompt_err):
            assert u.datasets["test2"].password is None
        with pytest.raises(RuntimeError, match=prompt_err):
            assert u.datasets["git"].password is None

        u.datasets["test"].password = "test_pw"
        u.datasets["backup"].password = "backup_pw"
        u.datasets["dummy_ldap_ds"].password = "dummy_ldap_ds_pw"
        u.datasets["test2"].password = "test2_pw"
        u.datasets["git"].password = "git_pw"
        assert u.password == "test_pw"

        out = self.global_cmds.creds.run("clear", "--all")
        origen_metal.sessions.unload()
        origen_metal.users.unload()
        origen._origen.boot_users()
        u.datasets["dummy_ldap_ds"].should_validate_password = False
        u = origen.current_user

        assert "Clearing all cached passwords..." in out
        with pytest.raises(RuntimeError, match=prompt_err):
            assert u.password is None
        with pytest.raises(RuntimeError, match=prompt_err):
            assert u.datasets["test"].password is None
        with pytest.raises(RuntimeError, match=prompt_err):
            assert u.datasets["backup"].password is None
        with pytest.raises(RuntimeError, match=prompt_err):
            assert u.datasets["dummy_ldap_ds"].password is None
        with pytest.raises(RuntimeError, match=prompt_err):
            assert u.datasets["test2"].password is None
        with pytest.raises(RuntimeError, match=prompt_err):
            assert u.datasets["git"].password is None
