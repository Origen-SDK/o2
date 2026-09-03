import origen, copy, pathlib, shutil, subprocess, os
from sphinx.errors import ExtensionError
from . import logger


def build_subprojects(sphinx):
    '''
    Builds any Origen projects whose documentation should be built within
    this project
  '''
    for subp, config in sphinx.config.origen_subprojects.items():
        SubProject(subp, config).build()


class SubProject:
    def __init__(self, proj, config):
        config = copy.deepcopy(config)
        self.proj = proj
        self.source = config.pop("source", None)
        self.final_output_dir = config.pop(
            "output_dir", origen.web.unmanaged_static_dir).joinpath(
                "origen_sphinx_extension").joinpath(proj)
        self.subproject_output_dir = self.get_subproject_output_dir()

    def get_subproject_output_dir_cmd(self):
        return "uv run --no-sync --no-editable python -c \"from origen.web import output_build_dir; print(str(output_build_dir))\""

    def get_subproject_output_dir(self):
        env = os.environ.copy()
        env.pop('VIRTUAL_ENV', None)
        out = subprocess.run(self.get_subproject_output_dir_cmd(),
                             shell=True,
                             cwd=self.source,
                             stdout=subprocess.PIPE,
                             stderr=subprocess.PIPE,
                             env=env)
        content = pathlib.Path(out.stdout.decode('utf-8').strip())
        if out.returncode == 0:
            return content
        else:
            raise ExtensionError(
                f"Unable to get output directory for subproject '{self.proj}'.\n"
                f"Stdout: {content}\n"
                f"Stderr: {out.stderr.decode('utf-8').strip()}"
            )

    def build_cmd(self):
        return "uv run --no-sync --no-editable python -c \"from origen.web import run_cmd; run_cmd('build', {})\""

    def build(self):
        if self.subproject_output_dir is not False:
            logger.info(
                f"Building docs for subproject '{self.proj}' - {self.build_cmd()}"
            )
            env = os.environ.copy()
            env.pop('VIRTUAL_ENV', None)
            out = subprocess.run(self.build_cmd(),
                                 shell=True,
                                 cwd=self.source,
                                 env=env)
            if out.returncode == 0:
                self.mv_docs()
            else:
                raise ExtensionError(
                    f"Failed to build subproject '{self.proj}' with exit code {out.returncode}"
                )

    def mv_docs(self):
        if self.subproject_output_dir.exists():
            if not self.final_output_dir.exists():
                # shutil will get mad if the directory doesn't exists.
                self.final_output_dir.mkdir(parents=True)
            else:
                # shutil will also get mad if the directory does exists.
                shutil.rmtree(str(self.final_output_dir))
                self.final_output_dir.mkdir(parents=True)
            logger.info(
                f"Moving docs from {self.subproject_output_dir} to {self.final_output_dir}..."
            )
            shutil.move(str(self.subproject_output_dir),
                        str(self.final_output_dir))
        else:
            raise ExtensionError(
                f"Could not find resulting docs for project '{self.proj}' at "
                f"{self.subproject_output_dir}"
            )

    def clean(self):
        if self.final_output_dir and self.final_output_dir.exists():
            logger.info(f"  Cleaning subproject {self.proj}")
            shutil.rmtree(str(self.final_output_dir))
