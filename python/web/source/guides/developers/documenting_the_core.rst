Documenting The Core
====================

Documenting the |origen_api| is much like
:link-to:`documenting any other Origen Application <documenting:introduction>`. The same
principles and commands applicable there apply here as well. When running commands inside of the
core itself, it functions as a dummy application named ``{{ origen.app.name }}`` - complete with its own
``config/application.toml`` and ``application.py``.

As with any existing application, Origen already has some content and other extensions configured outside
of the :link-to:`defaults <documenting:origen_included_extensions>` yielded by ``origen new``.

For details on the *implementation* of the documentation components, such as the |ose| or |shorthand|,
see the :link-to:`notes on the documentation generation implementation <developers:doc_gen_arch>`.

Documentation Overview
----------------------

Origen's documentation primarily uses :rst_quickstart:`RST <>`, though |markdown| is also
acceptable and should have much of the same features available thanks to
:recommonmark_embedded_rst:`Recommonmark's embedded RST <>`.

Configured Extensions
---------------------

The Origen core activates a few other extensions not necessarily activated in a standard
``origen new`` workspace:

*  |shorthand| - Provides *shorthand* notation for shorter and single-source referencing within Sphinx apps.

   * In addition to |documenting:origen_shorthand_defs|, the core application has its
     :link-to:`own set available <shorthand~project_namespace>` for internal use
     defined in :origen_app_shorthand_defs_src:`web/source/_conf/shorthand.py <>`.

*  :autosectionlabel_home:`autosectionlabel <>` - Automatically creates references to sections.

   * To prevent clashing (which **will** happen with the API generation), the configuration variable
     |autosectionlabel_prefix_document| is set to ``True`` and influences how |sphinx_refs| are resolved.
     For example, this section is referenced as
     :ref:`guides/developers/documenting_the_core:Configured Extensions <guides/developers/documenting_the_core:Configured Extensions>`
     instead of just as ``documenting_the_core:Configured Extensions``.

*  |inheritance_diagram| - Generates an :inheritance_diagram_example:`inheritance diagram <>` in the generated API.

   * This extension uses :graphviz_home:`Graphviz <>` under the hood,
     which is another third-party tool for visualizing and displaying graphs.
   * At least on Windows (and at the time of this writing), this tool must be installed manually from
     :graphviz_download:`Graphviz's website <>`. Since most classes are simplistic,
     using the extension is not required for release docs, but the extension is available otherwise.

*  :napoleon_home:`napoleon <>` - Allows for |docstrings| formatted according to the |google_docstring_spec|
   or |numpy_docstring_spec|, or any combinations of the two (along with standard |autodoc| or |py_domain| tags,
   which are automatically support by Origen applications).
*  :extlinks_home:`extlinks <>` - Allows for tracking and referencing of external links and should
   be used over direct references.

   * The :extlinks_config_var:`extlinks config variable <>` references the full dictionary defined in
     ``web/source/conf/extlinks.py``.

*  |documenting:rustdoc| - A home-brewed |sphinx_ext| that automates generating and moving Rust documentation into
   the core website's ``_static`` directory. See the |rustdoc_api| for additional details.

Internal Referencing
--------------------

Referencing sections of Origen's documentation within Origen should use either
:sphinx_ref_role:`Sphinx's :ref: role <>` or :link-to:`Shorthand referencing <shorthand~basic_usage>`, which
wraps aforementioned role. Both will itself launch ``consistency checks``
when building to ensure the references are valid and flag any broken ones. Using |shorthand| is preferred
as any shared references can be updated in bulk if needed, but using |sphinx_refs| directly is
acceptable otherwise.

:sphinx_ref_role:`Sphinx references <>` unfortunately do not work outside of RST sources, or parts of sources
parsed as RST. |Shorthand| provides some utilities for retrieving references usable outside of just RST sources. See
the :link-to:`template helpers <shorthand~templating>` for details.

Various examples can also be found within :link-to:`Origen's 'guides/' source <src_code:guides_root>`.

External Referencing
--------------------

The usage of direct links is discouraged in favor of |extlinks|, which allows tracking of external
links and (eventually) some more robust link-checking over :sphinx_nitpicky:`Sphinx's nitpicky option <>`.
External links in any non-user facing places is alright, as they won't be checked anyway, such as
internal code comments (i.e, comments not in the docstring) or ``READMEs`` that are not
included by the Sphinx app.

The ``extlinks`` dictionary is split from the main |conf.py| to help with organization and is located,
along with the :origen_app_shorthand_defs_src:`Origen core shorthand defs <>`, in |src_code:_conf_dir|.

Documenting The View (python/origen/origen)
-------------------------------------------

Documenting the user-facing ``origen`` module comes in two flavors: the API, and the guides
(this site!).

All user-facing methods and classes should have a |docstring|, which is picked up by
|autoapi| and |autodoc|, and keeps the API complete and available as a quick reference for
method prototypes, general usage, etc.

Larger and more complex features, especially widely used, or core, ones (such as
|timesets|, |pins|, |bits|, or |registers|) will need more detail than what can be provided easily by
just a |docstring|. For these features, pages can be added to this site. Its a judgement call
as to whether just ``docstrings`` are sufficient by themselves, but some form of API record should
be present for all user-facing methods, even for features which will have dedicated guides. Features
that do have dedicated guides should also have links to the appropriate API locations.

The output generated by |autodoc| will be parsed as normal RST, meaning |extlinks|, |shorthand|,
and |jinja| are all available inside docstrings - and many user-facing features take advantage of this.
See the :link-to:`origen mod <src_code:origen_init>` or the :link-to:`origen_sphinx_extension <src_code:ose_init>`
sources for examples.

:napoleon_home:`Sphinx's Napoleon extension <>` is available
for docstrings formatted per the |google_docstring_spec| or |numpy_docstring_spec| as well.

Documenting The Controller (rust/pyapi)
---------------------------------------

The notes above apply here as well, just with the caveat that the docstrings reside in
Rust instead of Python. |Autoapi| will create RST files from
the compiled code, but there's no difference between those and ones generated from the ``origen`` Python
module. All of the same features are available - including |extlinks|, |shorthand|, |jinja|, and
:napoleon_home:`Google/Numpy formatted docstrings`.

|autoapi| will pick up docstrings defined using :rust_docstrings:`Rust's syntax <>` on any
:pyo3_pyclass:`pyclass <>`, :pyo3_pyfunction:`pyfunction <>`, :pyo3_pymodule:`pymodule <>`, etc.

The only caveat known so far is that |autoapi| cannot discern method or function signatures
from compiled code and instead shows these as accepting no arguments, which obviously
isn't always the case. |Autodoc| will, however, take the first line of the docstring, if it is
the function/method name, as the *signature override* and we can use this to fill in the gap ourselves.
See :docstring_sig_override_so:`this stack-overflow question <>`
for details or the :link-to:`tester controller source <src_code:pytester>` for examples within the Origen core.

Note: this requires the :docstring_sig_override_cv:`appropriate config variable <>` to be set, which it is
by default.

Although the controller isn't meant to be user-facing, it is still *available*. Its not as imperative to document
as user-facing features but is still good to have some kind of |docstrings| in place to make it easier on
those developing core features.

Documenting The Model (rust/origen)
-----------------------------------

The ``model`` does not have any user-facing API, though documentation is still available for
developers to reference. Documenting the model should follow any Rust documentation standards.
Like the frontend though, complex concepts may be more easily documented on this site. For these
though, documentation should reside in :link-to:`the developers section <src_code:dev_guides_root>`.
