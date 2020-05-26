.. include:: ../../_common_defs.rst
  :start-after: start_content

Further Customizing Your Sphinx App
===================================

This section covers other customizations which are at either the *Origen application* level,
or from the workspace.

----

Application Customizations
--------------------------

The settings below reside in the *Origen application* but are used by |ref_ose|.
The settings below can be set in the application's `.toml` configuration file.

* ``website_output_directory``: Directory where the final built webpages should reside,
  relative to the application's root directory. Defaults to ``output/web``.
* ``website_source_directory``: Directory of the *Sphinx app*, relative to the application's root.
  Defaults to ``web/source``.

Recap
-----

* Some settings for the *Sphinx app*, or for how Origen's CLI runs Shpinx, are derived from outside
  of the *Sphinx app* itself.
* These settings can be set in the *Origen application's* ``.toml`` config file.

Up Next
-------

The next section will cover the ``origen web`` command in more detail.
