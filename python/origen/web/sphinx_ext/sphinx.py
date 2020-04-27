from sphinx.errors import ExtensionError
from sphinx.util.logging import getLogger
import origen, origen.web, shutil, copy, subprocess, pathlib
import sphinxbootstrap4theme
from recommonmark.transform import AutoStructify


logger = getLogger('Origen Sphinx Ext')

root = ""
static_root = ""
templates_root = ""
theme_dir = origen.frontend_root.joinpath("web/sphinx_ext/theme")

class SubProject:
  def __init__(self, proj, config):
    config = copy.deepcopy(config)
    self.proj = proj
    self.source = config.pop("source", None)
    self.final_output_dir = config.pop("output_dir", origen.web.static_dir).joinpath("origen_sphinx_ext").joinpath(proj)
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
    if self.final_output_dir:
      logger.info(f"  Cleaning subproject {self.proj}")
      shutil.rmtree(str(self.final_output_dir))

def setup(sphinx):
  sphinx.add_config_value("origen_deploy_function", None, '')
  sphinx.add_config_value("origen_archive_id", None, '')
  sphinx.add_config_value("origen_subprojects", {}, '')
  sphinx.add_config_value("origen_no_api", None, 'env')
  sphinx.add_config_value("origen_templates", None, '')

  sphinx.connect("config-inited", apply_origen_config)
  sphinx.connect("builder-inited", build_subprojects)
  sphinx.config.html_theme_path += [sphinxbootstrap4theme.get_path()]
  sphinx.add_html_theme('origen', str(theme_dir))

  # Bootstrap 4 setup: https://getbootstrap.com/docs/4.0/getting-started/introduction/
  # Note that JS is already included by Sphinx so the jquery-3.2.1.slim.min.js is not included here
  # If it is included, it doesn't seem to have any conspicuous impact, but it'll produce errors in the browser console
    # sphinx.add_js_file("https://code.jquery.com/jquery-3.2.1.slim.min.js")
  # Same goes for hte below items. The sphinxbootstrap extensions brings in the distributable package, so we'll have those.
  #  Might as well use them.
  #sphinx.add_js_file("https://cdnjs.cloudflare.com/ajax/libs/popper.js/1.12.9/umd/popper.min.js")
  #sphinx.add_js_file("https://maxcdn.bootstrapcdn.com/bootstrap/4.0.0/js/bootstrap.min.js")
  #sphinx.add_css_file("https://maxcdn.bootstrapcdn.com/bootstrap/4.0.0/css/bootstrap.min.css")

  sphinx.add_js_file('bootstrap-4.3.1-dist/js/bootstrap.min.js')
  sphinx.add_js_file('sphinxbootstrap4.js')
  sphinx.add_css_file('bootstrap-4.3.1-dist/css/bootstrap.min.css')
  sphinx.add_css_file('sphinxbootstrap4.css')
  # Experimenting with some Dark themes
  #sphinx.add_css_file("https://stackpath.bootstrapcdn.com/bootswatch/4.4.1/cyborg/bootstrap.min.css")
  #sphinx.add_css_file("https://stackpath.bootstrapcdn.com/bootswatch/4.4.1/darkly/bootstrap.min.css")
  sphinx.add_css_file('origen.css')

  # Note: Origen includes the recommomark module, so even if the user removes it from the extensions list in their own config,
  #  this will still be safe. It'll just have no usage.
  # Setup taken from: https://recommonmark.readthedocs.io/en/latest/auto_structify.html
  github_doc_root = 'https://github.com/rtfd/recommonmark/tree/master/doc/'
  sphinx.add_config_value('recommonmark_config', {
            'url_resolver': lambda url: github_doc_root + url,
            'auto_toc_tree_section': 'Contents',
            }, True)
  sphinx.add_transform(AutoStructify)

def apply_origen_config(sphinx, config):
  if config.origen_no_api:
    if "rustdoc_projects" in config.__dict__:
      config.rustdoc_projects.clear()
    if "autoapi_modules" in config.__dict__:
      config.autoapi_modules.clear()
    if "autodoc_modules" in config.__dict__:
      config.autodoc_modules.clear()

'''
  Builds any Origen projects whose documentation should be built within
  this project
'''
def build_subprojects(sphinx):
    for subp, config in sphinx.config.origen_subprojects.items():
      SubProject(subp, config).build()

'''
  Launches the Origen compiler for the given templates, placing them in the
  web output directory.
'''
def compile():
  raise NotImplementedError("Web-compile is not implemented yet!")

'''
  The default deploy function. This function uses the built-in application
  parameters to discern where the final output should be located on the
  site.

  This can be overriden by supplying a difference function to the config
  value 'origen_deploy_function'
'''
def deploy():
  raise NotImplementedError("Web-deploy is not implemented yet!")

def archive():
  raise NotImplementedError("Web-archive is not implemented yet!")

def clean(partial_config):
  logger.info("Cleaning origen_sphinx_ext...")
  if hasattr(partial_config, 'origen_subprojects'):
    for subp, _config in partial_config.origen_subprojects.items():
      SubProject(subp, _config).clean()