Architecture Overview
=====================

The Origen framework exists in two parts within a single package: a Python-based ``frontend``, which
users will build their applications upon, and a compiled ``backend``, which maintains the device,
tester, and other models. This is reminiscent of the :mvc_dp_wiki:`Model-View-Controller Design Pattern <>`
where the Rust-based backend functions as the ``model``, the user-facing |origen_api| API as the ``view``,
and the hidden |_origen_api| module as the ``controller``, bridging the two.

Development APIs
----------------

Not disclosed to the end users is the |_origen_api| which functions as a
controller connecting the frontend |origen_api| to the Rust-powered backend and
includes the actual device modeling and generation.

This API should not be used by end users directly and is entirely derived from the Rust backend.
Rather, functions are exposed to them through the ``origen`` module and should encapsulate all calls
to ``_origen``. This includes simple ``getters``, such as :attr:`origen.application.Base.output_dir`,
whose implementation is merely a call to :func:`_origen.output_directory`.

The backend is split into two Rust libraries: ``PyAPI`` and ``Origen``. ``PyAPI`` utilizes
:pyo3_crate_home:`PyO3 <>` which creates the Python module ``_origen``.

APIs
----
* The user/frontend API: :link-to:`Origen <origen_api>`
* The frontend controller to the backend model: :link-to:`_origen <_origen_api>`
* The frontend controller's development environment: |rustdoc_pyapi|
* The backend models' development environment, also called :link-to:`origen <rustdoc_origen>`
* Origen's :link-to:`command-line interface development environment <rustdoc_cli>`
* An |example_application_docs|