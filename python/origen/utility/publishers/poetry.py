from . import Publisher
import origen
from origen.utility.results import BuildResult, UploadResult
import subprocess


class Poetry(Publisher):
    poetry_repo_env_var = "POETRY_REPOSITORIES_ORIGEN"
    repo_name = "origen_pkg_repo"
    username_env_var = "POETRY_HTTP_BASIC_ORIGEN_PKG_REPO_USERNAME"
    password_env_var = "POETRY_HTTP_BASIC_ORIGEN_PKG_REPO_PASSWORD"

    def __init__(self, **config):
        Publisher.__init__(self, **config)
        self.pkg_cmd = config.get("pkg_cmd",
                                  ["poetry", "build", "--format", "wheel"])
        self.build_package_command_opts = {"capture": False}
        self.upload_cmd = config.get(
            "upload_cmd", ["poetry", "publish", "-r", self.repo_name])
        self.upload_package_command_opts = {"capture": False}

    def build_package(self):
        r = origen.utility.exec(self.pkg_cmd,
                                **self.build_package_command_opts)
        if r.succeeded():
            return BuildResult(succeeded=True, metadata={
                "format": "wheel",
            })
        else:
            return BuildResult(succeeded=False)

    def add_repo(self, repo, url):
        origen.log.trace(
            f"Adding repo '{repo}' to poetry config at URL '{url}'")
        return origen.utility.exec(
            ["poetry", "config", f"repositories.{repo}", url],
            **{"capture": False})

    def upload(self, build_result, dry_run):
        repo_url = origen.config["pkg_server_push"]
        r = self.add_repo(self.repo_name, repo_url)
        if r.succeeded():
            origen.log.trace(
                f"Added poetry repository {self.repo_name} ({repo_url})")
        else:
            return BuildResult(succeeded=False,
                               message="Failed to add poetry repository")

        cmd = self.upload_cmd
        opts = self.upload_package_command_opts

        d = origen.current_user().dataset_for("pkg_server_push")
        if d is None:
            d = origen.current_user()

        opts["add_env"] = {
            self.username_env_var: d.username,
            self.password_env_var: d.password_for("pkg_server_push",
                                                  default=None),
        }
        if dry_run:
            self.upload_cmd.append("--dry-run")
        r = origen.utility.exec(cmd, **opts)
        if r.succeeded:
            m = f"Successfully pushed package to {repo_url}"
        else:
            m = f"Failed to push package to repository {repo_url}"
        return UploadResult(r.succeeded(), message=m)
