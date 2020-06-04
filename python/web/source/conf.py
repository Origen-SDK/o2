# Configuration file for the Sphinx documentation builder.
#
# This file only contains a selection of the most common options. For a full
# list see the documentation:
# https://www.sphinx-doc.org/en/master/usage/configuration.html

# -- Path setup --------------------------------------------------------------

# If extensions (or modules to document with autodoc) are in another directory,
# add these directories to sys.path here. If the directory is relative to the
# documentation root, use os.path.abspath to make it absolute, like shown here.
#
import os, sys, pathlib
import origen
import origen.web

sys.path.insert(0, os.path.abspath('../../'))

# -- Project information -----------------------------------------------------

project = 'origen'
copyright = '2020, Origen Core Team'
author = 'Origen Core Team'

# -- General configuration ---------------------------------------------------
import rustdoc

# Add any Sphinx extension module names here, as strings. They can be
# extensions coming with Sphinx (named 'sphinx.ext.*') or your custom
# ones.
extensions = [
  'origen.web.origen_sphinx_extension',
  'origen.web.rst_shared_defs',
  'rustdoc',
  'sphinx.ext.autodoc',
  'sphinx.ext.autosectionlabel',
  'sphinx.ext.inheritance_diagram',
  'autoapi.sphinx',
  'recommonmark',
  'sphinx.ext.napoleon',
  'sphinx.ext.extlinks'
]

# Makes the references more verbose, but fixes a lot duplicated references resulting from autodoc.
# Silver lining to the verbosity is they're also crystal clear as to where they're going.
autosectionlabel_prefix_document = True

extlinks = {
  'sphinx_homepage': ('https://www.sphinx-doc.org/en/master/index.html%s', ''),
  'sphinx_app': ('https://www.sphinx-doc.org/en/master/usage/quickstart.html#getting-started%s', ''),
  'sphinx_docs': ('https://www.sphinx-doc.org/en/master/contents.html%s', ''),
  'sphinx_extensions': ('https://www.sphinx-doc.org/en/master/usage/extensions/index.html%s', ''),
  'sphinx_themes': ('https://www.sphinx-doc.org/en/master/usage/theming.html#themes%s', ''),
  'sphinx_using_a_theme': ('https://www.sphinx-doc.org/en/master/usage/theming.html#using-a-theme%s', ''),
  'sphinx_builtin_themes': ('https://www.sphinx-doc.org/en/master/usage/theming.html#builtin-themes%s', ''),
  'sphinx_add_theme': ('https://www.sphinx-doc.org/en/master/extdev/appapi.html#sphinx.application.Sphinx.add_html_theme%s', ''),
  'sphinx_creating_themes': ('https://www.sphinx-doc.org/en/master/theming.html#creating-themes%s', ''),
  'sphinx_available_themes': ('https://sphinx-themes.org/%s', ''),
  'sphinx_project_examples': ('https://www.sphinx-doc.org/en/master/examples.html#projects-using-sphinx%s', ''),
  'sphinx_conf': ('https://www.sphinx-doc.org/en/master/usage/configuration.html#module-conf%s', ''),
  'sphinx_add_config_var': ('https://www.sphinx-doc.org/en/master/extdev/appapi.html#sphinx.application.Sphinx.add_config_value%s', ''),
  'sphinx_confval_html_logo': ('https://www.sphinx-doc.org/en/master/usage/configuration.html#confval-html_logo%s', ''),
  'sphinx_confval_html_favicon': ('https://www.sphinx-doc.org/en/master/usage/configuration.html#confval-html_favicon%s', ''),
  'sphinx_confval_html_theme_options': ('https://www.sphinx-doc.org/en/3.x/usage/configuration.html#confval-html_theme_options%s', ''),
  'sphinx_rst': ('https://www.sphinx-doc.org/en/master/usage/restructuredtext/index.html%s', ''),
  'sphinx_rst_primer': ('https://www.sphinx-doc.org/en/master/usage/restructuredtext/basics.html%s', ''),
  'sphinx_rst_directives': ('https://www.sphinx-doc.org/en/master/usage/restructuredtext/directives.html%s', ''),
  'sphinx_templating': ('https://www.sphinx-doc.org/en/master/templating.html#templating%s', ''),
  'sphinx_manpages': ('https://www.sphinx-doc.org/en/master/man/index.html%s', ''),
  'sphinx_build_phases': ('https://www.sphinx-doc.org/en/master/extdev/index.html#build-phases%s', ''),
  'sphinx_connect': ('https://www.sphinx-doc.org/en/master/extdev/appapi.html#sphinx.application.Sphinx.connect%s',''),
  'sphinx_event_config_inited': ('https://www.sphinx-doc.org/en/master/extdev/appapi.html#event-config-inited%s', ''),
  'sphinx_event_builder_inited': ('https://www.sphinx-doc.org/en/master/extdev/appapi.html#event-builder-inited%s', ''),
  'sphinx_alabaster_theme': ('https://alabaster.readthedocs.io/en/latest/%s', ''),
  'rst_quickstart': ('https://docutils.sourceforge.io/docs/user/rst/quickstart.html%s', ''),
  'rst_cheatsheet': ('https://docutils.sourceforge.io/docs/user/rst/cheatsheet.txt%s', ''),
  'rst_docs': ('https://docutils.sourceforge.io/rst.html%s', ''),
  'rst_spec': ('https://docutils.sourceforge.io/docs/ref/rst/restructuredtext.html%s', ''),
  'rst_cokelaer_cheatsheet': ('https://thomas-cokelaer.info/tutorials/sphinx/rest_syntax.html#contents-directives%s', ''),
  'rst_guide_zephyr': ('https://docs.zephyrproject.org/latest/guides/documentation/index.html%s', ''),
  'jinja_home': ('https://palletsprojects.com/p/jinja/%s', ''),
  'jinja_docs': ('https://jinja.palletsprojects.com/en/master/%s', ''),
  'recommonmark_home': ('https://recommonmark.readthedocs.io/en/latest/%s', ''),
  'recommonmark_embedded_rst': ('https://recommonmark.readthedocs.io/en/latest/auto_structify.html#embed-restructuredtext%s', ''),
  'markdown_home': ('https://www.markdownguide.org/%s', ''),
  'autoapi_home': ('https://autoapi.readthedocs.io/%s', ''),
  'autoapi_usage': ('https://autoapi.readthedocs.io/#usage%s', ''),
  'autodoc_home': ('https://www.sphinx-doc.org/en/master/usage/extensions/autodoc.html%s', ''),
  'bootstrap4': ('https://getbootstrap.com/docs/4.5/getting-started/introduction/%s', ''),
  'bootstrap4_widgets': ('https://getbootstrap.com/docs/4.0/components/alerts/%s', ''),
  'bootstrap4_sphinx_theme': ('http://myyasuda.github.io/sphinxbootstrap4theme/%s', ''),
  'bootstrap4_sphinx_theme_options': ('http://myyasuda.github.io/sphinxbootstrap4theme/setup.html#html-theme-options%s', ''),
  'bootstrap4_sphinx_theme_templates': ('https://github.com/myyasuda/sphinxbootstrap4theme/tree/master/themes/sphinxbootstrap4theme%s', ''),
  'darkly': ('https://bootswatch.com/darkly/%s', ''),
  'dracula_pygments': ('https://draculatheme.com/pygments%s', ''),
  'o2_github_root': ('https://github.com/Origen-SDK/o2%s', ''),
  'static_website': ('https://en.wikipedia.org/wiki/Static_web_page%s', ''),
  'python_docs': ('https://docs.python.org/3.8/index.html%s', ''),
  'python_exception_hierarchy': ('https://docs.python.org/3/library/exceptions.html#exception-hierarchy%s', ''),
  'ticket_mako_multiple_newlines': ('https://stackoverflow.com/questions/22558067/how-to-convert-multiple-newlines-in-mako-template-to-one-newline%s', ''),
}

origen_content_header = {
  "insert-rst-shared-defs": True
}

doc_root = "guides/documenting"
pattern_api_root = 'guides/testers/pattern_generation/pattern_api'
program_api_root = 'guides/testers/program_generation/program_api'
utility_root = 'guides/runtime/utilities'
rst_shared_defs = {
  'output_dir': origen.web.interbuild_dir.joinpath('rst_shared_defs'),
  # 'api': {
  #   'class': {
  #     "origen-compiler": (
  #       "Origen's Compiler",
  #       'origen.compiler.Compiler'
  #     ),
  #     "origen-translator": (
  #       "Origen's Translator",
  #       'origen.translator.Translator'
  #     )
  #   }
  # },

  'refs': {
    'documenting_introduction': (
      'Documenting: Introduction',
      f"{doc_root}/introduction:Introduction"
    ),
    'documenting_core_concepts': (
      'Documenting: Core Concept',
      f"{doc_root}/core_concepts:Core Concepts"
    ),
    'pattern_api_comments': (
      'commenting pattern source',
      f'{pattern_api_root}:Comments'
    ),
    'program_api_comments': (
      'commenting program source',
      f'{program_api_root}:Comments'
    ),
    'logger': (
      'logger',
      f'{utility_root}/logger:Logger'
    ),

    'documenting': {
      'web_output_dir': (
        'web output directory',
        f"{doc_root}/further_customizing_your_sphinx_app:Application Customizations"
      ),
      'web_cmd': (
        'origen web',
        f"{doc_root}/building_your_webpages:Origen Web"
      ),
      'block_diagram': (
        'block diagram',
        f"{doc_root}/core_concepts:Doc System Block Diagram"
      ),
      'ose': (
        'origen sphinx extension',
        f"{doc_root}/the_origen_sphinx_extension:The Origen Sphinx Extension"
      ),
      'origen-s_sphinx_app': (
        'Origen\'s sphinx app',
        f"{doc_root}/your_sphinx_app:Origen's Sphinx App"
      ),
      'ose_theme': (
        'origen theme',
        f"{doc_root}/the_origen_sphinx_extension:The Origen Theme"
      ),
      'ose_theme_opts': (
        'origen sphinx extension theme options',
        f"{doc_root}/the_origen_sphinx_extension:Origen Theme Options"
      ),
      'ose_config_vars': (
        'origen sphinx extension config variables',
        f"{doc_root}/the_origen_sphinx_extension:Configuration Variables"
      ),
      'api_generation': (
        'API generation',
        f"{doc_root}/your_sphinx_app:Automatic API Generation"
      ),
      'ose_favicon': (
        'OSE favicon',
        f"{doc_root}/the_origen_sphinx_extension:Favicon"
      ),
      'ose_logos': (
        'OSE logos',
        f"{doc_root}/the_origen_sphinx_extension:Logos"
      ),
      'ose_subprojects': (
        'OSE SubProjects',
        f"{doc_root}/the_origen_sphinx_extension:SubProjects"
      ),
      'origen_included_extensions': (
        'Origen included extensions',
        f"{doc_root}/your_sphinx_app:Extensions"
      ),
    }
  },
}

rustdoc_output_dir = origen.web.static_dir.joinpath('rustdoc')
rustdoc_apply_svg_workarounds = True
rustdoc_projects = {
  'pyapi': {
    'source': origen.root.joinpath('../rust/pyapi'),
    'build_options': {}
  },
  'origen': {
    'source': origen.root.joinpath('../rust/origen'),
    'build_options': {'lib': None}
  },
  'cli': {
    'source': origen.root.joinpath('../rust/origen'),
    'build_options': {'bin': 'origen'}
  }
}

autoapi_modules = {
  'origen': {
    'module-members': ['undoc-members'],
    'class-members': ['members', 'undoc-members', 'inherited-members']
  },
  '_origen': {'orphan': True,
    'exclude-members': ['__doc__'],
    'class-members': ['members', 'undoc-members', 'inherited-members']
  }
}
autoapi_output_dir = origen.web.interbuild_dir.joinpath('autoapi')

# Build the example project's docs into this one.
origen_subprojects = {
  'example': {
    'source': origen.root.joinpath('../example'),
  }
}

# Add any paths that contain templates here, relative to this directory.
templates_path = ['_templates']

# Theme customizations
html_theme_options = {
  'navbar_links': [
    ('Github', 'https://github.com/Origen-SDK/o2', True),
    ('O1', 'https://origen-sdk.org/', True),
    ('Example App', '_static/build/origen_sphinx_extension/example/sphinx_build/index', False)
  ],
}

# List of patterns, relative to source directory, that match files and
# directories to ignore when looking for source files.
# This pattern also affects html_static_path and html_extra_path.
exclude_patterns = []

# -- Options for HTML output -------------------------------------------------

# The theme to use for HTML and HTML Help pages.  See the documentation for
# a list of builtin themes.
html_theme = 'origen'

# Add any paths that contain custom static files (such as style sheets) here,
# relative to this directory. They are copied after the builtin static files,
# so a file named "default.css" will overwrite the builtin "default.css".
html_static_path = ['_static']
origen_api_module_data_clashes = {
  'origen': [
    'origen.tester',
    'origen.producer'
  ]
}

def setup(app):
  # Since origen.logger is actually a reference to a module, AutoAPI will skip it.
  # Throw it back in as a variable.
  def replace_origen_logger(app, node):
    if node.name == 'origen':
      node.variables['logger'] = (getattr(origen, 'logger'), node.default_variable_opts())
  app.connect('autoapi-process-node', replace_origen_logger)
