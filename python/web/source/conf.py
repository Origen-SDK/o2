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
import os, sys, pathlib, subprocess
sys.path.insert(0, os.path.abspath('../../'))

import origen
import origen.web
from origen.web.shorthand.dev import add_shorthand_dev_defs

from sphinx.util.logging import getLogger
logger = getLogger("origen")

# -- Project information -----------------------------------------------------

project = 'origen'
copyright = '2020, Origen Core Team'
author = 'Origen Core Team'

# -- General configuration ---------------------------------------------------
from web.source._conf import origen_extlinks, origen_shorthand_defs

# Add any Sphinx extension module names here, as strings. They can be
# extensions coming with Sphinx (named 'sphinx.ext.*') or your custom
# ones.
extensions = [
    'origen.web.origen_sphinx_extension',
    'origen.web.shorthand',
    'origen.web.rustdoc',
    'sphinx.ext.autodoc',
    'sphinx.ext.autosectionlabel',
    'autoapi.sphinx',
    'recommonmark',
    'sphinx.ext.napoleon',
    'sphinx.ext.extlinks'
]

if subprocess.run("dot -V",
                  shell=True,
                  stdout=subprocess.PIPE,
                  stderr=subprocess.PIPE).returncode == 0:
    extensions.append('sphinx.ext.inheritance_diagram')
else:
    logger.info(
        "INFO: dot command 'dot' cannot be run (needed for graphviz output), check the graphviz_dot setting"
    )

# Makes the references more verbose, but fixes a lot duplicated references resulting from autodoc.
# Silver lining to the verbosity is they're also crystal clear as to where they're going.
autosectionlabel_prefix_document = True

extlinks = origen_extlinks

origen_content_header = {"insert-rst-shared-defs": True}
shorthand_defs = origen_shorthand_defs

rustdoc_output_dir = origen.web.unmanaged_static_dir.joinpath('rustdoc')
rustdoc_apply_svg_workarounds = True
rustdoc_projects = {
    'pyapi': {
        'source': origen.root.joinpath('../rust/pyapi'),
        'build_options': {}
    },
    'origen': {
        'source': origen.root.joinpath('../rust/origen'),
        'build_options': {
            'lib': None
        }
    },
    'cli': {
        'source': origen.root.joinpath('../rust/origen'),
        'build_options': {
            'bin': 'origen'
        }
    }
}

autoapi_modules = {
    'origen': {
        'module-members': ['undoc-members'],
        'class-members': ['members', 'undoc-members', 'inherited-members']
    },
    '_origen': {
        'orphan': True,
        'exclude-members': ['__doc__'],
        'class-members': ['members', 'undoc-members', 'inherited-members']
    }
}
autoapi_output_dir = origen.web.interbuild_dir.joinpath('autoapi')

# Build the example project's docs into this one.
origen_subprojects = {
    'example': {
        'source': origen.root.joinpath('../test_apps/python_app'),
    }
}

# Add any paths that contain templates here, relative to this directory.
templates_path = ['_templates']

# Theme customizations
html_theme_options = {
    'navbar_links': [
        ('Github', 'https://github.com/Origen-SDK/o2', True),
        # Took this out, it's good to keep the page around for O2 developers, but adds
        # clutter that the majority of users will not care about
        #('Example App',
        # '_static/build/origen_sphinx_extension/example/sphinx_build/index',
        # False),
        ('Community', 'community', False)
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
    'origen': ['origen.tester', 'origen.producer']
}


def setup(app):
    # Since origen.logger is actually a reference to a module, AutoAPI will skip it.
    # Throw it back in as a variable.
    def replace_origen_logger(app, node):
        if node.name == 'origen':
            node.variables['logger'] = (getattr(origen, 'logger'),
                                        node.default_variable_opts())

    def preprocess_docstring(app, what, name, obj, options, lines):
        if name == "origen.compiler.Renderer.file_extensions":
            lines.clear()
            lines.append("File extensions which this compiler recognizes.")

    app.connect('autoapi-process-node', replace_origen_logger)
    app.connect('origen-preprocess-docstring', preprocess_docstring)

    # Insert shorthand's own defs for its guides.
    origen.web.shorthand.dev.add_shorthand_dev_defs()
