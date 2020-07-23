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

``--no-api`` was mentioned when discussing |documenting:api_generation| but to recap: this option will bypass
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

Releasing
^^^^^^^^^

When the documentation is complete, it can be *released* by using the ``-r``, or ``--release`` switch.
The release procedure and location is dependent on options in the *Origen application*.

.. raw:: html

  <div class="alert alert-warning" role="alert">
  <i>Releasing</i> is a feature still in development. Pieces are working, but documentation is
  purposefully left scarce as certain aspects are either still in development or subject to change.
  </div>

When building your docs, you may see various *warnings* pop up. In general, it is not good practice
to leave build warnings hanging around for released content. *Releasing* will interpret all warnings
as errors and will **fail to release** the docs, even if the build previously succeeded without the
``--release`` switch. However, this can be overridden by also using the ``release-with-warnings`` switch.

*Releasing* will also add other, long-running checks into the mix - such as checking for the validity of
external links, which can take several minutes to complete for large projects. These checks can be
run during a development build by applying the switch ``--as-release`` to the build command.

Archiving
^^^^^^^^^

In conjunction with, or as an alternative to, releasing your docs, you can choose to *archive* them instead,
the intent being to provide a snapshot of the documentation corresponding to a particular
*Origen application* version.

*Archiving* is very similar to *releasing* except that the resulting build is released as a "sub-site*,
meaning that the *latest* content, as well as other *archives* are unchanged and the resulting build
is instead placed somewhere within the currently released site.

For example, using the ``archive <archive_id>`` option during ``origen web build`` will place the built docs
at ``<release_path>/archive/<archive_id>`` but keep the remaining ``<release_path>`` unaffected.

Recap
-----

* ``origen web build`` is actually a subcommand of the larger ``origen web`` command.
* Cleaning, building, and viewing your webpages can all be streamlined with the single
  command ``origen web build --clean --view``.
* In some circumstances, the webpages can still be viewed even on a failing build.
* Once the docs are complete, the ``-r``, or ``--release``, switch can be used to release the documentation.
* Similarly, the ``--archive <archive_id>`` switch can be used to instead release a snapshot of the current documentation
  with a particular ``archive id`` without affecting the *latest* or other *archives*.
