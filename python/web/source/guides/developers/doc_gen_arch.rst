Documentation Generation Architecture
=====================================

The |documenting:core_concepts| and |origen-s_sphinx_app| sections in the |guides| go into
discuss the |sphinx_app| and how using it in Origen differs from a standard one.
This section will discuss more of what *drives* the |sphinx_app| from an Origen application.

When a |web_cmd| is run from Origen, the :link-to:`CLI <rustdoc_cli>` will parse and kick off the
Python environment - which will then pick up the command and route it appropriately. That
routing will end up in the :func:origen.web.run_cmd function, which handles all web-oriented
commands. From there, ``sphinx-build`` (or whatever else) is kicked off in the context of the
user's Origen application.

The brunt of the web-drivers will be in the :mod:origen.web module, including the
:mod:origen.web.origen_sphinx_extension`origen_sphinx_extension` and, inside of that,
the templates, CSS, Javascript, etc. for the ``Origen theme``.

Other extensions maintained by the |core_team| but not necessarily tied to Origen, such as
|documenting:shorthand| and |documenting:rustdoc|, are also present within the ``origen.web``
module's namespace.
