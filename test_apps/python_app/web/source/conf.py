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
# import os
# import sys
# sys.path.insert(0, os.path.abspath('.'))

import sys, os
sys.path.insert(0, os.path.abspath('../../'))

import origen
import origen.web

# -- Project information -----------------------------------------------------

project = 'example'
copyright = '2020, Origen Core Team'
author = 'Origen Core Team'

# The full version, including alpha/beta/rc tags
release = '0.0.0'

# -- General configuration ---------------------------------------------------

# Add any Sphinx extension module names here, as strings. They can be
# extensions coming with Sphinx (named 'sphinx.ext.*') or your custom
# ones.
extensions = [
    'origen.web.origen_sphinx_extension', 'origen_autoapi.sphinx',
    'sphinx.ext.autodoc', 'sphinx.ext.napoleon', 'sphinx.ext.autosectionlabel',
    'recommonmark', 'origen.web.shorthand'
]

autosectionlabel_prefix_document = True
autoapi_modules = {
    'example.application': {
        'module-members': ['undoc-members'],
        'class-members': ['members', 'undoc-members']
    }
}
autoapi_output_dir = origen.web.interbuild_dir.joinpath('autoapi')

# Add any paths that contain templates here, relative to this directory.
templates_path = ['_templates']

# List of patterns, relative to source directory, that match files and
# directories to ignore when looking for source files.
# This pattern also affects html_static_path and html_extra_path.
exclude_patterns = []

# -- Options for HTML output -------------------------------------------------

# The theme to use for HTML and HTML Help pages.  See the documentation for
# a list of builtin themes.
html_theme_options = {
    'navbar_links':
    [('Github', 'https://github.com/Origen-SDK/o2/tree/master/example', True)],
    'logos': [{
        'src': '_static/example_logo.png',
        'alt': 'Example',
        'rel_src': True,
    }]
}

# Add any paths that contain custom static files (such as style sheets) here,
# relative to this directory. They are copied after the builtin static files,
# so a file named "default.css" will overwrite the builtin "default.css".
html_static_path = ['_static']
