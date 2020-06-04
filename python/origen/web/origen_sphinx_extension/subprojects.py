import origen, copy, pathlib, shutil, subprocess
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
    self.final_output_dir = config.pop("output_dir", origen.web.static_dir).joinpath("origen_sphinx_extension").joinpath(proj)
    self.subproject_output_dir = self.get_subproject_output_dir()

  def get_subproject_output_dir_cmd(self):
    return "poetry run python -c \"from origen.web import output_build_dir; print(str(output_build_dir))\""

  def get_subproject_output_dir(self):
    out = subprocess.run(self.get_subproject_output_dir_cmd(), cwd=self.source, capture_output=True)
    out = pathlib.Path(out.stdout.decode('utf-8').strip())
    return out

  def build_cmd(self):
    return "poetry run python -c \"from origen.web import run_cmd; run_cmd('build', {})\""

  def build(self):
    logger.info(f"Building docs for subproject '{self.proj}' - {self.build_cmd()}")
    subprocess.run(self.build_cmd(), cwd=self.source)
    self.mv_docs()

  def mv_docs(self):
    if self.subproject_output_dir.exists():
      if not self.final_output_dir.exists():
        # shutil will get mad if the directory doesn't exists.
        self.final_output_dir.mkdir(parents=True)
      else:
        # shutil will also get mad if the directory does exists.
        shutil.rmtree(str(self.final_output_dir))
        self.final_output_dir.mkdir(parents=True)
      logger.info(f"Moving docs from {self.subproject_output_dir} to {self.final_output_dir}...")
      shutil.move(str(self.subproject_output_dir), str(self.final_output_dir))
    else:
      logger.error(f"Could not find resulting docs for project {self.proj} at {self.subproject_output_dir}")
  
  def clean(self):
    if self.final_output_dir and self.final_output_dir.exists():
      logger.info(f"  Cleaning subproject {self.proj}")
      shutil.rmtree(str(self.final_output_dir))