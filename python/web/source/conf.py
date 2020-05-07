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
import os
import sys, pathlib
import sphinxbootstrap4theme
import origen
import origen.web

sys.path.insert(0, os.path.abspath('../../'))

# -- Project information -----------------------------------------------------

project = 'origen'
copyright = '2020, Origen Core Team'
author = 'Origen Core Team'


# -- General configuration ---------------------------------------------------
import rustdoc
import recommonmark
from recommonmark.transform import AutoStructify

# Add any Sphinx extension module names here, as strings. They can be
# extensions coming with Sphinx (named 'sphinx.ext.*') or your custom
# ones.
extensions = [
  'origen.web.sphinx_ext.sphinx',
  'rustdoc',
  'sphinx.ext.autodoc',

  # Causes a bunch of warnings with the APIs. Need to look into this first
  # It works, but the build output is ugly
  #'sphinx.ext.autosectionlabel',

  'sphinx.ext.inheritance_diagram',
  'autoapi.sphinx',
  'recommonmark',
]

rustdoc_output_dir = origen.web.static_dir.joinpath('rustdoc')
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
  'origen': None,
  '_origen': {'orphan': True}
}
autoapi_output_dir = origen.web.interbuild_dir.joinpath('autoapi')
autodoc_default_flags = ['members', 'undoc-members', 'inherited-members']
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
    ('Example App', '_static/build/origen_sphinx_ext/example/sphinx_build/index', False)
  ],
}
# List of patterns, relative to source directory, that match files and
# directories to ignore when looking for source files.
# This pattern also affects html_static_path and html_extra_path.
exclude_patterns = []

# -- Options for HTML output -------------------------------------------------

# The theme to use for HTML and HTML Help pages.  See the documentation for
# a list of builtin themes.
#
html_theme = 'origen'
#html_theme = 'sphinxbootstrap4theme'
#html_theme_path = [sphinxbootstrap4theme.get_path()]
#html_logo = "_static/origen-device.png" #"dummy.png"
#html_favicon =  None #"https://origen-sdk.org/favicon-32x32.png"

# Add any paths that contain custom static files (such as style sheets) here,
# relative to this directory. They are copied after the builtin static files,
# so a file named "default.css" will overwrite the builtin "default.css".
html_static_path = ['_static']

# github_doc_root = 'https://github.com/rtfd/recommonmark/tree/master/doc/'
# def setup(app):
#     app.add_config_value('recommonmark_config', {
#             'url_resolver': lambda url: github_doc_root + url,
#             'auto_toc_tree_section': 'Contents',
#             }, True)
#     app.add_transform(AutoStructify)
