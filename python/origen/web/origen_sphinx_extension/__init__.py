# from sphinx.errors import ExtensionError
import origen, origen.web, shutil, copy, subprocess, pathlib
from sphinx.util.logging import getLogger
logger = getLogger('Origen Sphinx Extension')

import sphinxbootstrap4theme
from recommonmark.parser import CommonMarkParser
from recommonmark.transform import AutoStructify

from . import templating, subprojects, misc

''' The defaults here can merged item-by-item by Python '''
ORIGEN_THEME_DEFAULTS = {
  'bypass_main_logo': True
}

ORIGEN_FAVICON_URL = 'https://origen-sdk.org/favicon-32x32.png'

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

root = ""
static_root = ""
templates_root = ""
theme_dir = origen.frontend_root.joinpath("web/origen_sphinx_extension/theme")

def setup(sphinx):
  '''
    Set up the |ose|

    This will:

      * Register various config variables
      * Add the Origen and sphinxbootstrap4 themes
      * Set the theme to 'origen' (overridable by the user later)
      * Configure :recommonmark_home:`recommonmark <>`
  
  '''
  sphinx.add_config_value("origen_subprojects", {}, '')
  sphinx.add_config_value("origen_no_api", None, 'env')
  sphinx.add_config_value("origen_templates", None, '')
  sphinx.add_config_value("origen_api_module_data_clashes", {}, '')
  sphinx.add_config_value("origen_refs_for", {}, '')
  sphinx.add_config_value("origen_content_header", {}, '')

  sphinx.connect("config-inited", apply_origen_config)
  sphinx.connect("builder-inited", subprojects.build_subprojects)
  sphinx.config.html_theme_path += [sphinxbootstrap4theme.get_path()]
  sphinx.add_html_theme('origen', str(theme_dir))

  for ext in origen.app.compiler.supported_extensions:
    #  supported extensions include the '.'
    sphinx.add_source_suffix(f'.rst{ext}', 'restructuredtext')
    sphinx.add_source_suffix(f'.md{ext}', 'markdown')

  # We'll set the theme here to Origen, but the user's config can override in their conf.py
  sphinx.config.html_theme = 'origen'

  # Note: Origen includes the recommonmark module, so even if the user removes it from the extensions list in their own config,
  #  this will still be safe. It'll just have no usage.
  # Setup taken from: https://recommonmark.readthedocs.io/en/latest/auto_structify.html
  # Adding the config here so users get it for free - its not particularly obvious what this does so want to abstract this as much as possible.
  # It can be overridden in the their own 'setup' method
  github_doc_root = 'https://github.com/rtfd/recommonmark/tree/master/doc/'
  sphinx.add_config_value('recommonmark_config', {
            'url_resolver': lambda url: github_doc_root + url,
            'enable_eval_rst:': True,
            }, True)
  sphinx.add_transform(AutoStructify)

  # There a bug floating around in recommonmark that show links generating warning like:
  #   '...\recommonmark\parser.py:75: UserWarning: Container node skipped: type=document'
  # Some Tickets:
  #   * https://github.com/readthedocs/recommonmark/issues/177
  #   * https://github.com/readthedocs/recommonmark/pull/185
  #   * https://github.com/readthedocs/recommonmark/issues/130
  # Looks like fixes may have been merged from the first ticket, but not released.
  # For now, monkeypatching the 'visit_document' definition based on this:
  #   https://github.com/readthedocs/recommonmark/issues/177#issuecomment-555553053
  # which just gets rid of the method body. Haven't noticed anything ill-effects yet.
  setattr(CommonMarkParser, 'visit_document', lambda *args : None)

  sphinx.connect("source-read", templating.insert_header)
  sphinx.connect("source-read", templating.preprocess_src)

def apply_origen_config(sphinx, config):
  '''
    The :meth:`setup` method Sphinx looks for will be run before the user's config, allowing us to 'preconfigure' several
    items and make their config simpler, more static, and less error prone.

    This method is hooked into :sphinx_event_config_inited:`sphinx's config-inited event <>` and
    is run **AFTER** their config has been processed.
    We can take their settings and overrides and combine them with what we have here.
    
    Regarding theme options:
      We'll set the 'html_theme' to 'origen' by default, but if its overridden then all our theme stuff is skipped.
  '''

  if config.origen_no_api:
    # Skip all the API generation by just clearing the appropriate config variables.
    if "rustdoc_projects" in config.__dict__:
      config.rustdoc_projects.clear()
    if "autoapi_modules" in config.__dict__:
      config.autoapi_modules.clear()
    if "autodoc_modules" in config.__dict__:
      config.autodoc_modules.clear()
  sphinx.connect("autodoc-process-docstring", templating.process_docstring)

  if len(config.origen_api_module_data_clashes) > 0:
    sphinx.connect('autoapi-process-node', misc.no_index_clashes)
    vars = []
    for v in config.origen_api_module_data_clashes.values():
      vars += [f'"{n}"' for n in v]
    sphinx.config.html_context['origen_module_pydata_clashes_js'] = "[" + ', '.join(vars) + "]"

  # Theme specific setup - assuming Origen's theme is used (set by default, but overridable in their config)
  if ('html_theme' in config) and (config.html_theme == 'origen'):
    # Add needed JS and CSS
    # Bootstrap 4 setup: https://getbootstrap.com/docs/4.0/getting-started/introduction/
    # However, the bootstrap4 theme that we're extending ships with the distributable bootstrap source.
    # Since we've got it, might as well use it. 
    # If we throw out the distributable package, the following three items must be included
    # sphinx.add_js_file("https://cdnjs.cloudflare.com/ajax/libs/popper.js/1.12.9/umd/popper.min.js")
    # sphinx.add_js_file("https://maxcdn.bootstrapcdn.com/bootstrap/4.0.0/js/bootstrap.min.js")
    # sphinx.add_css_file("https://maxcdn.bootstrapcdn.com/bootstrap/4.0.0/css/bootstrap.min.css")

    # JS files
    # Use distributable package since its there.
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
    sphinx.add_css_file('quote_card.css')
    sphinx.add_css_file('origen.css')

    sphinx.config.html_context['origen_version'] = origen.version

    # Merge the user's theme setup with Origen's
    if 'html_theme_options' in config:
      # Merge single items with the current config
      config.html_theme_options = {
        **ORIGEN_THEME_DEFAULTS,
        **config.html_theme_options
      }
      theme = config.html_theme_options
      if not 'favicon' in theme:
        theme['favicon'] = ORIGEN_FAVICON_URL
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

def clean(partial_config):
  logger.info("Cleaning origen_sphinx_extension...")
  if hasattr(partial_config, 'origen_subprojects'):
    for subp, _config in partial_config.origen_subprojects.items():
      subprojects.SubProject(subp, _config).clean()