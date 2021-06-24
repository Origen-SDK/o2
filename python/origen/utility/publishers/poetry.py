from . import Publisher
import origen
from origen.utility.results import BuildResult, UploadResult
import subprocess


class Poetry(Publisher):
    def __init__(self, **config):
        Publisher.__init__(self, **config)
        self.pkg_cmd = config.get("pkg_cmd",
                                  ["poetry", "build", "--format", "wheel"])
        self.build_package_command_opts = {"capture": False}

    def build_package(self):
        r = origen.utility.exec(self.pkg_cmd,
                                **self.build_package_command_opts)
        if r.succeeded():
            return BuildResult(succeeded=True, metadata={
                "format": "wheel",
            })
        else:
            return BuildResult(succeeded=False, )

    def upload(self, build_result, dry_run):
        # Kicks off the build
        ...
