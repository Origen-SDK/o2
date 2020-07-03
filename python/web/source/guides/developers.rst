.. Guides and notes on developing either the frontend or backend applications.

Developers
==========

.. toctree::
  
  developers/installation
  developers/architecture_overview
  developers/doc_gen_arch
  developers/documenting_the_core

The pages here will introduce you to the implementation and architectural aspects required to develop
the :origen_github_home:`origen_core <>` itself. Although
geared more towards existing or future developers, users which want a more *under-the-hood* view of Origen will find this section
enlightening.

For future developers, please see the |community_contributions| section for an overview of contributing to the core.

Development Installation
------------------------

Follow as shown the |dev_install| to setup your development environment.

User API
--------

The :link-to:`origen module <origen_api>` is the user-facing frontend.

Development APIs
----------------

Not disclosed to the users is the |_origen_api| which functions as a
controller connecting the frontend |origen_api| to the Rust-powered backend, which
includes the actual device modeling and generation.

This API should not be used by users directly and is entirely derived from the Rust backend.
Rather, functions are exposed to users through the `origen` module and should encapsulate all calls
to ``_origen``. This includes simple ``getters``, such as :attr:`origen.application.Base.output_dir`, whose implementation
is merely a call to :func:`_origen.output_directory`.

The backend is split into two Rust libraries: ``PyAPI`` and ``Origen``. ``PyAPI`` is a PyO3 library which
implements ``_origen``.

APIs
----
* The user/frontend API: :link-to:`Origen <origen_api>`
* The frontend controller to the backend model: :link-to:`_origen <_origen_api>`
* The frontend controller's development environment: |rustdoc_pyapi|
* The backend models' development environment, also called :link-to:`origen <rustdoc_origen>`
* Origen's :link-to:`command-line interface development environment <rustdoc_cli>`
* An |example_application_docs|
