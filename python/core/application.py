import origen
from origen.application import Base
from origen.utility.publishers.poetry import Poetry
from origen.utility.results import BuildResult

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
        self.build_package_command_opts["add_env"] = {"ORIGEN__COPY_BUILD_TARGET": "0"}

    def _call_build_package_cmd(self):
        return origen.utility.exec(self.pkg_cmd, capture=False, add_env={"ORIGEN__COPY_BUILD_TARGET": "0"})

    def build_package(self):
        origen.logger.info(f"Building Origen Release Libraries {origen.app.rust_dir}")
        origen.logger.info("Building Origen Release Libraries (origen executable)")
        r = origen.utility.exec(self.cargo_release_cmd + ["--workspace"], capture=False, cd=str(origen.app.rust_dir.joinpath("origen")))
        if r.failed():
            origen.logger.error("Failed to build Origen release")
            return BuildResult(False)

        origen.logger.info("Building Origen Release Libraries (pyapi)")
        r = origen.utility.exec(self.cargo_release_cmd, capture=False, cd=str(origen.app.rust_dir.joinpath("pyapi")))
        if r.failed():
            origen.logger.error("Failed to build PyAPI release")
            return BuildResult(False)

        return Poetry.build_package(self)

    def upload(self, _build_result):
        ...