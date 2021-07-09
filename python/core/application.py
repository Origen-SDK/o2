import origen
from origen.application import Base
from origen.utility.publishers.poetry import Poetry
from origen.utility.results import BuildResult, UploadResult
from origen.utility import github


# This class represents this application and is automatically instantiated as origen.app
# It is required by Origen and should not be renamed or removed under any circumstances
class Application(Base):
    def __init__(self):
        Base.__init__(self)
        self.rust_dir = self.app_dir.joinpath("../../rust").absolute()


class Publisher(Poetry):
    def __init__(self, **config):
        self.cargo_release_cmd = ["cargo", "build", "--release"]
        Poetry.__init__(self, **config)
        self.build_package_command_opts["add_env"] = {
            "ORIGEN__COPY_BUILD_TARGET": "0"
        }

    def _call_build_package_cmd(self):
        return origen.utility.exec(self.pkg_cmd,
                                   capture=False,
                                   add_env={"ORIGEN__COPY_BUILD_TARGET": "0"})

    def build_package(self):
        origen.logger.info(
            f"Building Origen Release Libraries {origen.app.rust_dir}")
        origen.logger.info(
            "Building Origen Release Libraries (origen executable)")
        r = origen.utility.exec(self.cargo_release_cmd + ["--workspace"],
                                capture=False,
                                cd=str(origen.app.rust_dir.joinpath("origen")))
        if r.failed():
            msg = "Failed to build Origen release"
            origen.logger.error(msg)
            return BuildResult(succeeded=False, message=msg)

        origen.logger.info("Building Origen Release Libraries (pyapi)")
        r = origen.utility.exec(self.cargo_release_cmd,
                                capture=False,
                                cd=str(origen.app.rust_dir.joinpath("pyapi")))
        if r.failed():
            msg = "Failed to build PyAPI release"
            origen.logger.error(msg)
            return BuildResult(succeeded=False, message=msg)

        return Poetry.build_package(self)

    def upload(self, build_result, dry_run):
        # The mechanism to actually build, load, and publish the libraries is
        # a github action.
        # Running this action with no arguments essentially does nothing - it'll
        # build everything but won't actually publish any libraries.
        # When running for real, all inputs will be given to publish the libraries.
        if dry_run:
            inputs = {}
        else:
            # Need to fill these out yet when final publishing action is done
            inputs = {
                "publish_pypi_test": "true",
                # "publish_pypi": "true"
            }
        res = github.dispatch_workflow("Origen-SDK", "o2", "publish.yml",
                                       "master", inputs)
        if res.succeeded:
            msg = "Publish action successfully started. Check https://github.com/Origen-SDK/o2/actions/workflows/publish.yml for further status"
        else:
            msg = f"Encountered errors starting the publish action. Message from Github: {res.message}"
        return UploadResult(res.succeeded, message=msg)
