Documenting
=============

"Documenting" can apply to several aspects of Origen. You could document:

* Patterns
* Programs
* Flows
* APIs
* Applications

In this section, we'll talk about documenting the applications themselves.

Any application created with ``origen new`` will already have the pieces in place to generate a
`static website <https://en.wikipedia.org/wiki/Static_web_page>`_.
Static websites allow us to generate self-contained webpages which can easily by served by a web server or
packaged up and distributed without introducing any dependencies on the readers.

The 'web' command
-------------------

Origen ships with the ``web`` command, which handles parameterizing and calling the underlying tools used to generate the webpages.

The ``web`` command is actually a group of subcommands, the foremost being: ``build``:

Building The Webpages
^^^^^^^^^^^^^^^^^^^^^

.. code-block:: none

  origen web build

If you run this command in your application, you'll see it running something called `Sphinx <https://www.sphinx-doc.org/en/master/>`_.
Sphinx is the main *underlying tool* hinted at above and is a widely used Python library used to generate static webpages, including
the `Python documentation itself! <https://docs.python.org/3.8/index.html>`_ 
(check the footer to see "Created using Sphinx 2.3.1", at least for Python 3.6 through 3.8).

When you built your Origen application with ``origen new``, some of the resulting structures were specifically geared towards running Sphinx and yielding
a very basic ``sphinx app`` for you to build atop of. This *app* will the *boiler plate* items filled out already, allowing you to jump right into
content writing.

For a new application, the built webpages will be located in the application's output directory, ``output/``, at offset ``web/sphinx_build``. (This, actually,
is configurable, but that will be a later topic.) Navitaging to the folder and opening the ``index.html`` page will take you to your application's homepage.
Or, you can use the ``origen web view`` command to launch your machine's default browser at that location. Alternatively, the build command also has a switch 
to view the resulting webpages after the build: ``origen web build --view``.

``Sphinx`` is *makefile-like*, in that it will only recompile changed files, leading to faster build times. A side-effect of this, however,
is that the web browser may still be launched, even on a failing build, giving the illusion that the build succeeded. ``origen web view`` will also launch
the browser as normal, but with it being a separate command, errors resulting from the build are less likely to go unnoticed in this flow.

An easy way to get around the above is just to wipe out the results and rebuild from scratch. The ``origen web clean`` command will do just that. Running this
command will remove any webpages from a previous build, forcing a full reocmpilation. It will also run ``clean`` on any
`extensions <https://www.sphinx-doc.org/en/master/usage/extensions/index.html>`_ which supports cleaning.

``origen web build`` in an Origen application is all that's needed to get some initial webpages off the ground. See the web command itself,
``origen web -h``, for a full list of the available commands. See each command's own help message, e.g. ``origen web build -h``,
for options specific to that command.

About Sphinx
------------

As stated, Origen applications include a pre-configured setup for `Sphinx <https://www.sphinx-doc.org/en/master/>`_. In the next sections, many topics will refer you to
the `Sphinx Documentation <https://www.sphinx-doc.org/en/master/contents.html>`_ to cover items best learned from the source, rather than being paraphrased here.

Before delving into the Sphinx docs though, its important to know *what* customizations Origen has made to the Sphinx app, placing your pre-configured
Sphinx app commensurate with an app from Sphinx's *quickstart*, which the latter's documentation will assume.

Sphinx includes a ``quickstart`` command which will build some default files for you. When you ran ``origen new``, some of the files built
mimic those which Sphinx's quickstart would have built. The key difference is *where*.

Origen places Sphinx apps a bit more out-of-the-way than where ``quickstart`` would: in ``web/source``, instead of just ``source``. Navigating to
``web/source`` though, you'll see the same files ``sphinx-quickstart`` would have given you: most notably, ``config.py`` and ``index.rst``. These
are the same files which are referenced frequenntly in Sphinx's docs. Even though some content is already filled in, these still function
as Sphinx describes and all options remain available to you.

This may sound a bit ominous, but there's not too much content added. The key addition is the automatic inclusion of ``origen_sphinx_ext`` as the
first (topmost) extension. This extension will be `covered in much more detail later <#the-origen-extension>`_, but it is responsible for all of the *Origen specifics*
which separate a run-of-the-mill Sphinx app from one used in an Origen application.

.. raw:: html

  <div class="alert alert-info" role="alert">
    When we say "it is responsible for all of the <i>Origen specifics</i>" we mean it! Removing this extension
    will return you to a default application, as constructed by <i>sphinx-quickstart</i>. This may be what you want, if you want complete control from
    the ground up, but you will lose some of the interactions available from Origen in the broader sense.

    For example, many of the <code>origen web build</code> switches and features rely on functionality from the <code>origen_sphinx_ext</code>. Removing this extension
    without implementing that functionality yourself will cause those items to not function correctly.

    The <code>origen_sphinx_ext</code> has a number of customizations available and can be inherited or overriden like any other Sphinx extension. This
    will be <a href="#the-origen-extension">covered in more detail later</a>, but this should be preferred to removing the extension entirely.
  </div>

  <div class="alert alert-danger" role="alert">
    Moving this extension around in the load order will have unknown effects, almost all of which will be bad. For utmost compatability, it should
    remain as the first extension enabled.
  </div>

Adding Content
--------------

Understanding now that your Origen application's documentation is really just a pre-configured Sphinx app with the ``origen_sphinx_ext`` pre-included,
you can beginning adding content. Origen includes some additions here but it also does not get in the way of Sphinx's regularl flow.

Sphinx content primarily uses `restructured text (RST) <https://www.sphinx-doc.org/en/master/usage/restructuredtext/index.html>`_, which serves both to link documents together and format the actual content.
Tutorials on restructured text are out of scope here, as Sphinx and the RST official website is abound with `primers <https://www.sphinx-doc.org/en/master/usage/restructuredtext/basics.html>`_,
`tutorials <https://docutils.sourceforge.io/docs/user/rst/quickstart.html>`_, and more in-depth `documentation <https://docutils.sourceforge.io/docs/ref/rst/restructuredtext.html>`_ 
that will cover more ground than we ever could.

The important thing is that, even though we have a customized application, it is still a Sphinx app and, as such, the content there is applicable here.

Markdown
^^^^^^^^

Adjacent to *restructured text* is another popular markup language: `markdown <https://www.markdownguide.org/>`_. Depending on your background,
you may already have experience using Markdown but none using RST and wish to continue using Markdown to write content.
A Sphinx extension, `recommonmark <https://recommonmark.readthedocs.io/en/latest/>`_ is available to build Markdown content for Sphinx apps and Origen comes with this
pre-installed. Additionally, the *origen_sphinx_ext* will configure your Markdown to accept
`embedded RST <https://recommonmark.readthedocs.io/en/latest/auto_structify.html#embed-restructuredtext>`_, allowing for you to place 
`RST directives Sphinx uses <https://www.sphinx-doc.org/en/master/usage/restructuredtext/directives.html>`_  inside your Markdown documents.

See the `recommonmark <https://recommonmark.readthedocs.io/en/latest/>`_ docs for more information.

Templates
^^^^^^^^^

You may have already come across `templating <https://www.sphinx-doc.org/en/master/templating.html>`_ in your Sphinx reading. In case you haven't, *templates* allow for content to be
dynamically added into your documentation through `Jinja <https://palletsprojects.com/p/jinja/>`_, Sphinx's templating language of choice. Like RST, Markdown, and Sphinx
in general, tutorials on Jinja will not be covered here, but head over to the `Jinja documentation <https://jinja.palletsprojects.com/en/master/>`_ to learn all about it.

Origen applications come pre-configured to invoke the Jinja processor on RST templates, as well as any of the content in the ``_templates``, or other added *template directories*.

.. Templates inside of your pre-configured Sphinx app work just the same as any other Sphinx app. Origen does, however, through some
  additional items available in your templates. By default, Sphinx tosses in `these items <>`_ when building templates. For general
  apps, these are usually sufficient, but we may need additional context. The `origen_sphinx_ext` will also provide you with `origen`,
  booted up as normal, which you can use to dynamically place content in your applications.
  With the `origen` module at your disposable, you can, for instance, `instantiate targets <>`_ and dynamically add content
  such as `pins <>`_, `registers <>`_, or anything else!

.. raw:: html

  <div class="alert alert-primary" role="alert">
    Origen's template engine (invoked via <code>origen compile <...></code> is currently <u><b>not available</b></u> (at least not directly) for Sphinx documentation. This, however,
    is on the roadmap. Check back soon!
  </div>

Extensions
----------

As has been alluded to several times, Sphinx has the concept of `extensions <https://www.sphinx-doc.org/en/master/usage/extensions/index.html>`_, which are additions that can be plugged into Sphinx
to give increased functionality or customization. We've brought up the ``origen_sphinx_ext`` a few times, and its definition is coming up. We've also described `recommonmark extension <#markdown>`_, which
is brought in and configured automatically.

Automatic API Generation
^^^^^^^^^^^^^^^^^^^^^^^^^

Origen also includes two other extensions: `AutoAPI <https://autoapi.readthedocs.io/>`_, which will parse your top-level module for Python objects and doc strings, and
build RST files, and `autodoc <https://www.sphinx-doc.org/en/master/usage/extensions/autodoc.html>`_ which will parse those resulting RST files into viewable content.

.. raw:: html

  <div class="alert alert-warning" role="alert">
    AutoAPI works by iterating through the <b>built module</b>, not just by parsing the files. This means that your application, and all
    connected modules, classes, etc., must at least load correctly in Python for AutoAPI to run to completion.
  </div>

When your Origen application is built, AutoAPI will be automatically added as an extension, with your application's namespace as a target.
This setup, though automatic, is done by during the Origen app creation and can be easily customized, or removed entirely, from
your Sphinx's ``conf.py``. See the `usage section <https://autoapi.readthedocs.io/#usage>`_ present in its documentation.

.. raw:: html

  <div class="alert alert-info" role="alert">
    APIs can take some time to parse and build, especially for larger projects. For quicker turnaround, the <code>--no-api</code> switch can be
    given to the build command to bypass running this extension for that particular build.

    AutoAPI will always rebuild the APIs, but contents from a previous run will persist from run to run. Assuming no changes to the source,
    <code>--no-api</code> can be used after an initial build without any adverse effects.

    This feature requires that the <code>origen_sphinx_ext</code> is present.
  </div>

The Origen Extension
---------------------

Now that you've been exposed to `extensions <https://www.sphinx-doc.org/en/master/usage/extensions/index.html>`_, we can talk about the *origen_sphinx_ext*, which bridges the gap
between the Origen application and the Sphinx app.

As you may have seen on the Sphinx docs, extensions are capable of making certain customization including:

* `Registering config variables <https://www.sphinx-doc.org/en/master/extdev/appapi.html#sphinx.application.Sphinx.add_config_value>`_
* Setting up config values, either for itself or other extensions
* `Hooking into the build in various phases <https://www.sphinx-doc.org/en/master/extdev/appapi.html#sphinx.application.Sphinx.connect>`_, such as when the
  `config is initialized <https://www.sphinx-doc.org/en/master/extdev/appapi.html#event-config-inited>`_
  or when `the builder is first initialized <https://www.sphinx-doc.org/en/master/extdev/appapi.html#event-builder-inited>`_
* `Setting up themes <https://www.sphinx-doc.org/en/master/extdev/appapi.html#sphinx.application.Sphinx.add_html_theme>`_ (`covered a bit later <#themes>`_)

Most of the 'behind the scenes work' is done by this extension, hooked into Sphinx at various phases.

SubProjects
^^^^^^^^^^^^^^^^

Some Origen workspaces are actually a collection of applications. Or, some more functional (or plugin) applications may have a smaller, development
application built into, or adjacent to, it - the latter being how the `Origen project is actually setup <https://github.com/Origen-SDK/o2>`_. In applications where it is expected
that one application should encompass some others, it may be more fliud for the documentation of one to encompass the documentation of another, without
actually *containing* that other's Sphinx app.

The Origen extensions has a means for one project to automate building and capturing the static webpages of one application into another. This is setup
by registering to-be-encompassed projects in the ``origen_subprojects`` configuration variable. Registering a project here will, during the top application's
`build phase <https://www.sphinx-doc.org/en/master/extdev/index.html#build-phases>`_, run the ``origen web build`` command for the given subprojects
and copy its resulting webpages into ``_static/origen_sphinx_ext/<subproject name>``. This subproject's built webpages can then be referenced in the current app.

.. code-block:: python

  # Register another Origen application which this application should build
  # The subprojects are a nested dictionary where the key is the subproject name and the source
  # points to that applications root.
  # Note: this should be the Origen application's root, not the Sphinx application's root.
  #  (The nested dictionary structure is to allow for future customizations to individual subprojects)
  origen_subprojects = {
    'example': {
      'source': 'path/to/example/application/root,
    }
  }

Configuration Variables
^^^^^^^^^^^^^^^^^^^^^^^

The Origen extension adds these configuration variables:

.. py:data:: origen_subprojects

  Any Origen applications whose documentation should be built and encompassed in this.

  .. versionadded:: 0.0.0

.. py:data:: origen_no_api

  Indicates whether the API should be built. This is set automatically when the ``--no-api`` switch is used.

  .. versionadded:: 0.0.0

Application Customizations
^^^^^^^^^^^^^^^^^^^^^^^^^^

The settings below reside in the Origen application but are used by the Origen extension. Most of these reside in the application's `.toml` configuration.

* ``website_output_directory``: Directory where the final built webpages should reside, relative to the application's root directory. Defaults to ``output/web``.
* ``website_source_directory``: Directory of the *Sphinx app*, relative to the application's root. Defaults to ``web/source``.

Themes
------

`extensions <https://www.sphinx-doc.org/en/master/usage/extensions/index.html>`_ are geared towards adding *functionality* to your Sphinx app, to give you better tools with which to write content.
However, very little has been said about what gives your generated webpages their look, style, or flair. For this, Sphinx uses
`themes <https://www.sphinx-doc.org/en/master/usage/theming.html#themes>`_ and, like so many other aspects, Origen has a hook for that.

Before jumping into Origen's theme, take a moment to glance through some of `Sphinx's built-in themes <https://www.sphinx-doc.org/en/master/usage/theming.html#builtin-themes>`_. Although we've already
chosen one for you, the examples there should show you exactly what is meant by the *look and feel* of your webpages. You can also view the vast amount of
third-party themes Sphinx has `many themes available <https://sphinx-themes.org/>`_ available.

The Origen Theme
^^^^^^^^^^^^^^^^

Unless `the configuration says otherwise <https://www.sphinx-doc.org/en/master/usage/theming.html#using-a-theme>`_, Origen will set the current theme to 'origen'.
Origen's theme is a modified `bootstrap4 <https://pypi.org/project/sphinxbootstrap4theme/>`_ theme with `darkly <https://bootswatch.com/darkly/>`_ overlayed atop.

Origen's theme also includes some items not easily reachable by extensions. The *origen theme options* section below will give a tour of what options Origen's theme
has available.

Origen Theme Options
""""""""""""""""""""

.. code-block:: python

  html_theme_options = {
    # Given logos will line the top of the Navbar, starting on the left-hand side.
    # Base Sphinx only allows a single logo that must reside in '_static' and can only link to
    # the project's homepage.
    # (See: https://www.sphinx-doc.org/en/master/usage/configuration.html#confval-html_logo)
    #
    # Origen's theme offers the ability to use multiple logos with more flexibility per logo.
    # The logos will be appear in the order in which they are inserted
    logos: [
      # Add a logo from an external source
      {
        'src': 'https://link_to_my_logo.png',
        'href': 'https://link_my_logo_points_to',
        'alt': 'alternative text to display if the logo cannot be found',
        'rel_src': False
      },

      # Add a logo from a source relative to the project (such as at '_static')
      {
        'src': '_static/my_other_logo.png',
        'href': 'https://link_my_other_logo_points_to',
        'alt': 'alternative text',
        'rel_src': True
      }
    ],

    # If a 'config.html_logo' is not give, the project's name and a link to the homepage is inserted
    # instead. If this option is set to True, then this logo Sphinx normally adds
    # will be surpressed, leaving only the theme's logos given above.
    bypass_main_logo = False

    # Sphinx's favicon (shown here: https://www.sphinx-doc.org/en/master/usage/configuration.html#confval-html_favicon)
    # doesn't allow for a favicon located outside of '_static'
    # The favicon provided here can reside outside of '_static', or
    # as a URL if 'favicon_raw_src' (shown below) is set to True.
    # Websites can only display one favicon and Sphinx's 'config.html_favicon' takes priority.
    # If 'config.html_favicon' is set to anything other than 'None' (its default value) or False
    # (or, technically, anything else that resolve to False),
    # it will be used in place of the favicon given here.
    favicon = "path/to/favicon/in/_static/directory.png"
      # or "https://url/for/your/favicon.png"
      # or "path/to/wherever.parsing
      # if 'favicon_raw_src' is set (see below)

    # If set to True, the favicon 'src' will be whatever value, verbatim.
    # If False, then the favicon is assumed to reside in '_static/'.
    # This item has no effect if 'config.html_favicon' is set
    favicon_raw_src = False
  }

To maintain some semblance across Origen applications, a few logos will be prepended automically. The settings for these logos are shown
below.

{% set json = importlib.import_module('json') %}
{% set ext = importlib.import_module('origen.web.sphinx_ext.sphinx') %}

.. code:: json

  {{ eval("'\\n'.join(f'  {s}' for s in json.dumps(ext.ORIGEN_THEME_LOGOS, indent=4).split('\\n'))", {'json': json, 'ext': ext}) }}


The Origen Theme's Parent
^^^^^^^^^^^^^^^^^^^^^^^^^

Origen's theme `extends <https://www.sphinx-doc.org/en/master/theming.html#creating-themes>`_ the
`Sphinx Bootstrap4 Theme <http://myyasuda.github.io/sphinxbootstrap4theme/>`_, which not only gives the webpages their
look, but also enables `Boostrap4 widgets <https://getbootstrap.com/docs/4.0/components/alerts/>`_, out of the box.

The *Bootstrap4 theme* also has its own set of `html_them_options <http://myyasuda.github.io/sphinxbootstrap4theme/setup.html#html-theme-options>`_.

Overriding Origen's Theme
^^^^^^^^^^^^^^^^^^^^^^^^^

If something in Origen's theme is not to your liking, your Sphinx app can `override templates <https://www.sphinx-doc.org/en/master/theming.html#templating>`_ 
used by Origen's theme. To inherit from Origen's template, only overriding aspects given by your app's template,
`inherit from <https://www.sphinx-doc.org/en/master/theming.html#creating-themes>`_ ``origen/web/sphinx_ext/theme/<template>.html``.

.. Origen currently supplies these templates and these static files:

Since Origen inherits from the `sphinxbootstrap4 <http://myyasuda.github.io/sphinxbootstrap4theme/index.html>`_ theme,
templates to extend may `reside there as well <https://github.com/myyasuda/sphinxbootstrap4theme/tree/master/themes/sphinxbootstrap4theme>`_.

Extending Themes
^^^^^^^^^^^^^^^^

As shown above, Sphinx allows for `one theme to extend another <https://www.sphinx-doc.org/en/master/theming.html#creating-themes>`_.
The Origen theme is no exception, allowing for alterations without
entirely breaking away from it - maintaining `origen extension <#the_origen_extension>`_ features which rely on the theme.

Choosing A Different Theme
^^^^^^^^^^^^^^^^^^^^^^^^^^

Although picked for you during the Origen application creation, the Origen theme is completely optional.
`Setting the theme <https://www.sphinx-doc.org/en/master/usage/theming.html#using-a-theme>`_ in your config will override and decouple your webpages from the Origen theme entirely but at the expense
of the aforementioned `Origen Theme Options <#origen-theme-options>`_ (or at least in their current form).

All of the Origen-theme specifics are contained within the ``html_theme_options`` configuration setting, so breaking away from the
Origen theme will only impact those options. Though a bit more involved, it is encouraged to inherit from Origen's theme, rather
than breaking away from it entirely, to maintain the highest degree of functionality.

.. raw:: html

  <div class="alert alert-info" role="alert">
    Setting the theme to <code>None</code> in your <code>conf.py</code> will revert your Sphinx app's theme to Sphinx's default: 
    <a href='https://alabaster.readthedocs.io/en/latest/>'>the Alabaster theme</a>.
  </div>
