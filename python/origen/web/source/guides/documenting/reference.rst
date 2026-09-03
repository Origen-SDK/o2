Reference
=========

Origen Web
-----------

* Build the webpages:

  ``origen web build``
* Build the webpages and launch your browser to view the results:

  ``origen web build --view``
* Clean the artifacts of a previous run, rebuild the webpages, and launch your browser:

  ``origen web build --clean --view``
* Build, watch, and serve with live reload on Python 3.11+:

  ``origen web serve --open``
* Skip generated API, Rustdoc, and subproject content for a faster edit loop:

  ``origen web serve --fast``

Run ``origen web --help`` or see the |web_cmd| section for further details.

Origen's Shorthand Defs
-----------------------

Origen includes a set of :link-to:`Shorthand Defs <shorthand~basic_usage>` (namespaced as ``origen``) for quick
referencing to common places your application documentation may point to, especially where new-user or
installation guides are concerned:

{% set m = importlib.import_module('origen.web.origen_sphinx_extension.shorthand_defs') %}

.. code:: python

  {{ m.defs | pprint }}

Reference Materials
-------------------

Sphinx Guides
^^^^^^^^^^^^^

* :sphinx_homepage:`Sphinx Homepage <>`
* :sphinx_app:`Sphinx App <>`
* :sphinx_conf:`Sphinx's conf.py <>`
* :sphinx_extensions:`Sphinx Extensions <>`
* |origen-s_sphinx_app|
* |documenting:origen_included_extensions|

   * :autodoc_home:`Autodoc <>`
   * :autoapi_home:`AutoAPI <>`
* :link-to:`The origen Sphinx Extension <ose>`
* :sphinx_themes:`Sphinx Themes <>`
* |ose_theme|

Jinja
^^^^^

* :sphinx_templating:`Templating In Sphinx <>`
* :jinja_home:`Jinja <>`

.. * Templating In Origen
.. * Invoking Origen's Compiler
.. * Standard Templating Context

RST Guides
^^^^^^^^^^

* :sphinx_rst_primer:`Sphinx's RST Primer <>`
* :rst_quickstart:`RST Quickstart <>`
* :rst_spec:`RST Docs <>`
* Other useful RST/Sphinx guides

   * :rst_cheatsheet:`RST cheatsheet <>`

..  RST In Origen Cheatsheet
.. """"""""""""""""""""""""

.. The below examples show some quick RST examples for common items and any Sphinx or Origen accommodations.

.. * Adding a new page
..   * Header
..   * Toctree
.. * Links
..   * Internal Page Links

..     Example

..   * External Links

..     Example

..   * Linking to APIs

..     Example

Customizations
^^^^^^^^^^^^^^

* Adding a :py:data:`favicon`
* Adding :py:data:`logos`
* :link-to:`Adding a subproject <ose_subprojects>`
* :link-to:`Configuration Variables <ose_config_vars>`

Other
^^^^^

* :link-to:`Markdown <markdown>`

   * :markdown_home:`Markdown Introduction <>`
   * `MyST Parser <https://myst-parser.readthedocs.io/>`_
* :link-to:`The Napoleon Extension <napoleon>`

   * |google_docstring_spec|
   * |numpy_docstring_spec|
* :link-to:`The Autosectionlabel Extension <autosectionlabel>`

   * :link-to:`Prefix document config <autosectionlabel_prefix_document>`
