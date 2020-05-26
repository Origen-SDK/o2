.. include:: ../../_common_defs.rst
  :start-after: start_content

Building Your Webpages
======================

This section will discuss the ``origen web`` command in more detail and cover some of the available
options for ``origen web build``.

----

At the very beginning, we introduced the build command: ``origen web build --view`` as a means to generate
and view your project's documentation. Now that we've covered more of what Origen and Sphinx have
to offer, we can revisit this command and take a closer look at some of its options.

Origen Web
----------

The ``build`` command is actually a subcommand in the larger ``origen web`` command.

Running ``origen web --help`` will show you what can be done with the *Sphinx app* from the CLI:

{% set origen_exec = '../rust/origen/target/debug/origen.exe' -%}

{{ insert_cmd_output(origen_exec + " web --help") }}

Building
^^^^^^^^

You should now be familiar with the ``build`` command's basic usage, but what about some of the other options?

Running ``build`` with ``--help`` gives us the following options:

{{ insert_cmd_output(origen_exec + " web build --help") }}

``--no-api`` was mentioned when discussing |ref_api_generation| but to recap: this option will bypass
generating any API contents. Existing contents will persist though, so this option can be used without
any ill-effects provided the API source hasn't changed. However, this also means that continuously running
with ``--no-api`` during development could result in stale API documentation.

Viewing And Cleaning
^^^^^^^^^^^^^^^^^^^^

``Sphinx`` is *makefile-like*, in that it will only recompile changed files, leading to faster build times.
A side-effect of this, however, is that the web browser may still be launched, even on a failing build,
giving the illusion that the build succeeded.

An easy way to get around the above is just to wipe out the results and rebuild from scratch. The
``origen web clean`` command will do just that. Running this command will remove any webpages from
a previous build, forcing a full recompilation. It will also run ``clean`` on any
:sphinx_extensions:`extensions <>` which supports cleaning.

The webpages can also be built with a clean *Sphinx app* using ``origen web build --clean``. This is the
same as running:

.. code:: none

  origen web clean
  origen web build

Likewise, the commands:

.. code:: none

  origen web clean
  origen web build
  origen web view

Can all be mashed into the same command as ``origen web build --clean --view``

Recap
-----

* ``origen web build`` is actually a subcommand of the larger ``origen web`` command.
* Cleaning, building, and viewing your webpages can all be streamlined with the single
  command ``origen web build --clean --view``.
* In some circumstances, the webpages can still be viewed even on a failing build.

Up Next
-------

The next section will cover some more advanced customization options.
