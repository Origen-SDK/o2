from . import Publisher
import origen
from origen.utility.results import BuildResult, UploadResult


class UV(Publisher):
    """Build and publish Python distributions with UV."""

    username_env_var = "UV_PUBLISH_USERNAME"
    password_env_var = "UV_PUBLISH_PASSWORD"

    def __init__(self, **config):
        Publisher.__init__(self, **config)
        self.pkg_cmd = config.get("pkg_cmd", ["uv", "build", "--wheel"])
        self.build_package_command_opts = {"capture": False}
        self.upload_cmd = config.get("upload_cmd", ["uv", "publish"])
        self.upload_package_command_opts = {"capture": False}

    def build_package(self):
        result = origen.utility.exec(
            self.pkg_cmd,
            **self.build_package_command_opts,
        )
        if result.succeeded():
            return BuildResult(succeeded=True, metadata={"format": "wheel"})
        return BuildResult(succeeded=False)

    def upload(self, build_result, dry_run):
        repo_url = origen.config["pkg_server_push"]
        command = [*self.upload_cmd, "--publish-url", repo_url]
        if dry_run:
            command.append("--dry-run")

        dataset = origen.current_user().dataset_for("pkg_server_push")
        if dataset is None:
            dataset = origen.current_user()

        options = dict(self.upload_package_command_opts)
        options["add_env"] = {
            self.username_env_var: dataset.username,
            self.password_env_var: dataset.password_for(
                "pkg_server_push", default=None
            ),
        }
        result = origen.utility.exec(command, **options)
        if result.succeeded():
            message = f"Successfully pushed package to {repo_url}"
        else:
            message = f"Failed to push package to repository {repo_url}"
        return UploadResult(result.succeeded(), message=message)
