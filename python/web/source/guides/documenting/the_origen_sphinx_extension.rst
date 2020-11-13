The Origen Sphinx Extension
===========================

Now that you've been exposed to :sphinx_extensions:`extensions <>`, we can talk about the
|inline_ose|, which bridges the gap between the *Origen application* and the *Sphinx app*.

----

As you may have seen on the Sphinx docs, extensions are capable of making certain customizations, including:

* :sphinx_add_config_var:`Registering config variables <>`
* Setting up config values for itself **and** for other extensions
* :sphinx_connect:`Hooking into the build in various phases <>`, such as when the
  :sphinx_event_config_inited:`config is initialized <>`
  or when :sphinx_event_builder_inited:`the builder is first initialized <>`
* :sphinx_add_theme:`Setting up themes <>`

All of the Origen specifics, or the 'behind the scenes work,' is done by this extension - hooked
into your *Sphinx app* in various phases.

The |inline_ose| is responsible for, but not limited to:

* Listening for and enacting options from ``origen build``.
* Setting up the |ose_theme|
* Configuring :recommonmark_home:`recommonmark <>` to accept embedded RST.
* Allowing Jinja templating in all RST files.
* Building and including |ose_subprojects|

All of that is 'behind-the-scenes' though. The |inline_ose| also provides some helpful
features and additional config variables.

Configuration Variables
^^^^^^^^^^^^^^^^^^^^^^^

The Origen extension adds these configuration variables:

.. py:data:: origen_subprojects

  Any Origen applications whose documentation should be built and encompassed in this. See
  :link-to:`Subproject <ose_subprojects>` for more details.

  .. versionadded:: 0.0.0

.. py:data:: origen_no_api

  Indicates whether the API should be built. This is set automatically when the ``--no-api``
  switch is used.

  .. versionadded:: 0.0.0

SubProjects
^^^^^^^^^^^

Some Origen workspaces are actually a collection of applications. Or, some more functional (or plugin)
applications may have a smaller, development application built into, or adjacent to, it - the
latter being how the :o2_github_root:`Origen project is actually setup <>`.
In applications where it is expected that one application will encompass some others,
it may be more fluid for the documentation to do the same, but without actually *containing*
that other's *Sphinx app* (or *Origen application* for that matter).

The |inline_ose| has a means for one project to automate building and capturing the static
webpages of one *Origen application* into another. This is setup
by registering to-be-encompassed projects in the ``origen_subprojects`` configuration variable.
Registering a project here will, during the top application's
:sphinx_build_phases:`build phase <>`, run ``origen web build`` for the given subprojects
and copy its resulting webpages into ``_static/origen_sphinx_ext/<subproject name>``. This
subproject's webpages can now be referenced in the current app like any other static entity and
shipped along with the top *Origen's application's* webpages.

.. code-block:: python

  # Register an Origen application which this application should build webpages for.
  # The subprojects are a nested dictionary where the key is the subproject name and the source
  # points to that Origen application's root (note: not the Sphinx app's root).
  #  (The nested dictionary structure is to allow for future customizations to individual subprojects)
  origen_subprojects = {
    'example': {
      'source': 'path/to/example/application/root,
    }
  }

The Origen Theme
^^^^^^^^^^^^^^^^

Unless :sphinx_using_a_theme:`the configuration says otherwise <>`, Origen will set the current theme to
``origen``. Origen's theme is a modified :bootstrap4:`bootstrap4 <>` theme with
:darkly:`darkly <>` overlaid atop and dark-themed syntax highlighting from :dracula_pygments:`dracula <>`.

Origen's theme also includes some items not easily reachable by extensions. The *origen theme options*
section below will give a tour of what options Origen's theme has available.

Origen Theme Options
""""""""""""""""""""

.. py:data:: logos

  Given logos will line the top of the navbar, starting on the left-hand side.

  Base Sphinx only allows a single logo that must reside in ``_static`` and can only link to
  the project's homepage. See: :sphinx_confval_html_logo:`config.html_logo <>`

  Origen's theme offers the ability to use multiple logos with more flexibility per logo.
  The logos will be appear in the order in which they are inserted

  .. code-block:: python
  
    html_theme_options['logos'].append({
      # Add a logo from an external source
      'src': 'https://link_to_my_logo.png',
      'href': 'https://link_my_logo_points_to',
      'alt': 'alternative text to display if the logo cannot be found',
      'rel_src': False
    })
  
  .. code-block:: python

    html_theme_options['logos'].append({
      # Add a logo from a source relative to the project (such as in '_static')
      'src': '_static/my_other_logo.png',
      'href': 'https://link_my_other_logo_points_to',
      'alt': 'alternative text',
      'rel_src': True
    })

  .. versionadded:: 0.0.0

.. py:data:: bypass_main_logo

  In addition to the logos above, if a :sphinx_confval_html_logo:`config.html_logo <>` is not given,
  the project's name with a reference pointing to the homepage is inserted as the foremost logo.

  This *project-level* logo can be suppressed by setting ``bypass_main_logo`` to ``True``,
  leaving only the theme's logos given above.

  Default:
    False

  .. code-block:: python

    html_theme_options['bypass_main_logo'] = False

  .. versionadded:: 0.0.0

.. py:data:: favicon_raw_src

  If set to ``True``, the favicon ``src`` will be whatever the value given is, verbatim.
  If ``False``, then the favicon is assumed to reside in ``_static/``, which is Sphinx's default.
  This item has no effect if ``config.html_favicon`` is set

  Default:
    False

  .. code-block:: python

    html_theme_options['favicon_raw_src'] = False

  .. versionadded:: 0.0.0

.. py:data:: favicon

  Sphinx's favicon, :sphinx_confval_html_favicon:`shown here <>`,
  doesn't allow for one located outside of ``_static``
  
  The favicon provided here can reside outside of ``_static``, or
  as a URL if ``favicon_raw_src`` is set to ``True``.
  
  Websites can only display one favicon and Sphinx's ``config.html_favicon`` takes priority.
  If ``config.html_favicon`` is set to anything other than ``None`` or ``False``
  (or, technically, anything else that *resolves* to ``False``),
  it will be used in place of anything given here.

  Default:
    None

  .. code-block:: python

    # if html_theme_options['favicon_raw_src'] is set

    # Direct path
    html_theme_options['favicon'] = "path/to/favicon/in/_static/directory.png"

    # URL
    html_theme_options['favicon'] = "https://url/for/your/favicon.png"
  
  **Note:** Be sure not tp confuse ``html_theme_options['favicon']``, the *theme config*, with ``conf.html_favicon``,
  which is a Sphinx global config variable and takes precedence over ``html_theme_options['favicon']``.

  .. versionadded:: 0.0.0

----

To maintain some semblance across Origen applications, a few logos will be prefixed automatically.
The settings for these logos are shown below:

{% set json = importlib.import_module('json') %}
{% set ext = ose %}

.. code:: json

  {{ eval("'\\n'.join(f'  {s}' for s in json.dumps(ext.ORIGEN_THEME_LOGOS, indent=4).split('\\n'))", {'json': json, 'ext': ext}) }}

Recap
-----

The |inline_ose|:

* is the bridge between your *Origen application*, Origen itself, and your *Sphinx app*.
* will setup several aspects of your *Sphinx app* for you at runtime.
* has its own set of :link-to:`configuration variables <ose_config_vars>`.
* also comes with support for :link-to:`SubProjects <ose_subprojects>`
* contains the |ose_theme|

* The |ose_theme|, though part of the |inline_ose|, contains its own
  :link-to:`configuration options <ose_theme_opts>`, which follows Sphinx's rules for
  configuring themes, but is also accessible in your ``config.py`` via
  :sphinx_confval_html_theme_options:`html_theme_options`.
