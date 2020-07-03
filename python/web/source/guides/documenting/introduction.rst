Introduction
============

"Documenting" can apply to several aspects of Origen. You could document:

* Patterns
* Programs
* Flows
* Applications

In this section, we'll talk about documenting the applications themselves. See 
:link-to:`the pattern API <patgen:comments>` and
:link-to:`the program API <prog-gen:comments>`
topics for documenting pattern and program sources, respectively.

Any application created with ``origen new`` will already have the pieces in place to generate a
:static_website:`static website <>`, which itself allows for self-contained
webpages which can easily by served by a web server or packaged up and distributed without introducing any
dependencies on the end users.

Your Origen application documentation engine features:

* A fully-functioning :sphinx_app:`Sphinx app <>` out-of-the-box.
* Integration of the |ose|, which ties into your *Origen application* and the |web_cmd| command.
* An fully-featured :bootstrap4:`bootstrap4 <>`-derived :sphinx_themes:`theme <>`
  with the :darkly:`darkly <>` overlay and :dracula_pygments:`dracula <>` syntax highlighting.

Basic Build Command
-------------------

The entry point into Origen's doc generation is the |web_cmd| command. Take a moment to run the
following in your workspace to get an idea of the initial state of your application's docs:

.. code-block:: none

  origen web build --view

This command will build the webpages, placing them in your |web_output_dir| and launch
your system's web-browser to view the resulting contents. This will yield webpages specific
to your application which have the same look and feel as the Origen documentation.

See the |web_cmd| section, or run ``origen web -h`` from your terminal, for more details.

Up Next
-------

The next section will introduce you to the core concepts of documenting your
*Origen application* - including the technologies used.
