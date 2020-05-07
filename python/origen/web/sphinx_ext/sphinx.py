from sphinx.errors import ExtensionError
from sphinx.util.logging import getLogger
import origen, origen.web, shutil, copy, subprocess, pathlib
import sphinxbootstrap4theme
from recommonmark.transform import AutoStructify

''' The defaults here can merged item-by-item by Python '''
ORIGEN_THEME_DEFAULTS = {
  'bypass_main_logo': True
}

ORIGEN_FAVICON = 'https://origen-sdk.org/favicon-32x32.png'

''' Default navbar links which must be merged manually '''
ORIGEN_THEME_NAVBAR_LINKS = [
  # release notes
]

''' Default logos which must be merged manually '''
ORIGEN_THEME_LOGOS = [
  {
    'src': 'https://origen-sdk.org/img/origen-device.png',
    'href': 'https://origen-sdk.org/',
    'alt': 'o1',
    'rel_src': False,
  },
  {
    'src': '_static/o2_zero_effort_logo.png',
    'href': 'https://origen-sdk.org/o2',
    'alt': 'o2',
    'rel_src': True,
  },
]

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
  sphinx.config.html_theme = 'origen'

  # Note: Origen includes the recommomark module, so even if the user removes it from the extensions list in their own config,
  #  this will still be safe. It'll just have no usage.
  # Setup taken from: https://recommonmark.readthedocs.io/en/latest/auto_structify.html
  # Adding the config here so users get it for free - its not particularly obvious what this does so don't want to risk them messing with it.
  # It can be overriden in the their own 'setup' method
  github_doc_root = 'https://github.com/rtfd/recommonmark/tree/master/doc/'
  sphinx.add_config_value('recommonmark_config', {
            'url_resolver': lambda url: github_doc_root + url,
            'auto_toc_tree_section': 'Contents',
            }, True)
  sphinx.add_transform(AutoStructify)
  sphinx.connect("source-read", jinja_integrator)

'''
  The 'setup' method will be run before the user's config, allowing us to 'preconfigure' several
    items and make their config simpler, less error pronne, and, well, preconfigured.
  The setup below is run AFTER their config was been run. We can take their settings/overrides and conmbine
    them with what we have here.
  
  Regarding theme options:
    We'll set the 'html_theme' to 'origen' by default, but if its overriden then all our theme stuff is skipped.
'''
def apply_origen_config(sphinx, config):
  if config.origen_no_api:
    if "rustdoc_projects" in config.__dict__:
      config.rustdoc_projects.clear()
    if "autoapi_modules" in config.__dict__:
      config.autoapi_modules.clear()
    if "autodoc_modules" in config.__dict__:
      config.autodoc_modules.clear()

  # Theme specific setup - assuming Origen's theme is used (set by default, but overridable in their config)
  # Aside: if they set the theme to 'None', Sphinx's default (Alabastar) will be used
  if ('html_theme' in config) and (config.html_theme == 'origen'):
    # Add our needed JS and CSS
    # Bootstrap 4 setup: https://getbootstrap.com/docs/4.0/getting-started/introduction/
    # However, the bootstrap4 theme that we're extending ships with the distributable bootstrap source.
    # Since we've got it, might as well use it. 
    # If we throw out the distributable package, the following three items must be included
    # sphinx.add_js_file("https://cdnjs.cloudflare.com/ajax/libs/popper.js/1.12.9/umd/popper.min.js")
    # sphinx.add_js_file("https://maxcdn.bootstrapcdn.com/bootstrap/4.0.0/js/bootstrap.min.js")
    # sphinx.add_css_file("https://maxcdn.bootstrapcdn.com/bootstrap/4.0.0/css/bootstrap.min.css")

    # JS files
    # Use distributable package
    sphinx.add_js_file('bootstrap-4.3.1-dist/js/bootstrap.min.js')
    # sphinx.add_js_file('https://stackpath.bootstrapcdn.com/bootstrap/4.4.1/js/bootstrap.bundle.min.js')
    sphinx.add_js_file('sphinxbootstrap4.js')
    sphinx.add_js_file('origen.js')

    # CSS Files
    sphinx.add_css_file('bootstrap-4.3.1-dist/css/bootstrap.min.css')
    # sphinx.add_css_file("https://stackpath.bootstrapcdn.com/bootstrap/4.4.1/css/bootstrap.min.css")
    # Experimenting with some Dark themes - personally, I like darkly the most, but some other good candidates are below
    # sphinx.add_css_file("https://stackpath.bootstrapcdn.com/bootswatch/4.3.1/cyborg/bootstrap.min.css")
    # sphinx.add_css_file("https://stackpath.bootstrapcdn.com/bootswatch/4.3.1/slate/bootstrap.min.css")
    sphinx.add_css_file("https://stackpath.bootstrapcdn.com/bootswatch/4.4.1/darkly/bootstrap.min.css")
    sphinx.add_css_file('sphinxbootstrap4.css')
    sphinx.add_css_file('https://gitcdn.link/repo/dracula/pygments/master/dracula.css') # ('dracula.css')
    sphinx.add_css_file('origen.css')

    # Merge the user's theme setup with Origen's
    if 'html_theme_options' in config:
      # Merge single items with the current config
      config.html_theme_options = {
        **ORIGEN_THEME_DEFAULTS,
        **config.html_theme_options
      }
      theme = config.html_theme_options
      if not 'favicon' in theme:
        theme['favicon'] = ORIGEN_FAVICON
        theme['favicon_raw_src'] = True

      # If the config as navbar links. prepend ours to theirs
      if 'navbar_links' in theme:
        theme['navbar_links'] = ORIGEN_THEME_NAVBAR_LINKS + theme['navbar_links']
      else:
        theme['navbar_links'] = ORIGEN_THEME_NAVBAR_LINKS

      # Same with the logos
      if 'logos' in theme:
        theme['logos'] = ORIGEN_THEME_LOGOS + theme['logos']
      else:
        theme['logos'] = ORIGEN_THEME_LOGOS
    else:
      config.html_theme_options = {
        **{
          'navbar_links': ORIGEN_THEME_NAVBAR_LINKS,
          'logos': ORIGEN_THEME_LOGOS
        },
        **ORIGEN_THEME_DEFAULTS
      }

# Setup taken from here: https://www.ericholscher.com/blog/2016/jul/25/integrating-jinja-rst-sphinx/
def jinja_integrator(app, docname, source):
  src = source[0]
  import builtins, types, inspect
  rendered = app.builder.templates.render_string(
    src, {
      **builtins.__dict__,
      **types.__dict__,
      **inspect.__dict__,
      **{
        'origen': origen,
        'origen_sphinx_ext': origen.web.sphinx_ext,
      }
    } # app.config.html_context
  )
  source[0] = rendered

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