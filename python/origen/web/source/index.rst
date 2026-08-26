Origen 2
========

The semiconductor developer's kit, rebuilt for modern Python and Rust
workflows. O2 brings device modeling, pattern generation, test-program
generation, plugins, and documentation into one cohesive toolkit.

Start building
--------------

.. container:: o2-link-grid

   .. container:: o2-link-card

      **Get started**

      Install Origen, create an application, and understand the workspace.

      :doc:`Open the getting-started guide <guides/getting_started>`

   .. container:: o2-link-card

      **Model a device**

      Define hierarchy, registers, pins, timesets, and reusable model APIs.

      :doc:`Explore device modeling <guides/device_modeling>`

   .. container:: o2-link-card

      **Generate tester content**

      Build patterns and programs for supported ATE platforms, including
      UltraFLEX.

      :doc:`Explore tester workflows <guides/testers>`

   .. container:: o2-link-card

      **Extend Origen**

      Package reusable commands, models, services, and documentation as
      plugins.

      :doc:`Build an Origen plugin <guides/plugins>`

Documentation
-------------

Use the navigation and search to browse task-oriented guides, the Python and
Rust API references, release notes, and contributor documentation.

.. toctree::
   :maxdepth: 2
   :titlesonly:
   :hidden:

   Guides <guides/index>
{% if not origen_sphinx_app.config.origen_no_api %}
   API Reference <interbuild/autoapi/origen/origen>
{% endif %}
   Release Notes <guides/release_notes/index>
   Community <community>
