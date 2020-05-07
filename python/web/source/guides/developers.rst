.. Guides and notes on developing either the frontend or backend applications.

.. toctree::
  :hidden:

  developers/installation

Developers
==================================

The Origen framework exists in two parts: a Python-based `frontend`, which users will build their applications upon, and a compiled `backend`,
which maintains the device, tester, and other various models.

Development Installation
------------------------

Follow as shown `the installation guide here <developers/installation.html>`_ to setup your
development environment.

User API
--------

The :doc:`origen module <../interbuild/autoapi/origen/origen>` is the user-facing frontend.

Development APIs
----------------

Not disclosed to the users is the :doc:`_origen API <../interbuild/autoapi/_origen/_origen>` which functions as an
adapter connecting the frontend user API :doc:`origen module <../interbuild/autoapi/origen/origen>` to the
Rust-powered backend.

This API should not be used by users directly and is entirely derived from the Rust backend.
Rather, functions exposed to the users through the `origen` module should encapsulate all calls
to `_origen`. This includes simple `getters`, such as `origen.app.output_dir`, whose implementation
is merely a call to the `_origen.output_directory()` function.

The backend is split into two Rust libraries: `PyAPI` and `Origen`. `PyAPI` is a PyO3 library which
implements `_origen`.

APIs
---------
* The user API: :doc:`origen module <../interbuild/autoapi/origen/origen>`
* The frontend adapter to the backend, :doc:`_origen <../interbuild/autoapi/_origen/_origen>`
* The backend listener, `pyapi <../_static/build/rustdoc/pyapi/doc/_origen/index.html>`_
* The compiled backend model, also called `origen <../_static/build/rustdoc/origen/doc/origen/index.html>`_
* The `command line interface <../_static/build/rustdoc/cli/doc/origen/index.html>`_
* An `example application <../_static/build/origen_sphinx_ext/example/sphinx_build/index.html>`_
