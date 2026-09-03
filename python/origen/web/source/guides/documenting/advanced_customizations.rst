Advanced Customizations
=======================

This section will cover some advanced customizations - the concept being that the *Sphinx app* and
the |ose| are flexible enough to allow overriding aspects which are not to your
liking without having to start from scratch or abandoning all of what Origen provides.

----

The removal or deactivation of certain things will have consequences, but if those consequences
can be enumerated and understood, then it'll make advanced customization all the easier.

Themes
------

You may not like the theme Origen has chosen for you - and that's quite alright.
This section will cover what Origen's theme contains, how it can be extended, but also the consequences
of axing it entirely out of your *Sphinx app*.

The Origen Theme's Parent
^^^^^^^^^^^^^^^^^^^^^^^^^

On current Python versions, Origen uses the `PyData Sphinx Theme
<https://pydata-sphinx-theme.readthedocs.io/>`_ and layers its branding,
navigation defaults, logo, favicon, and CSS on top. Consult the PyData theme's
``html_theme_options`` documentation when extending this mode.

Older supported Python versions that cannot install the modern theme use the
bundled ``origen`` theme, which extends the
:bootstrap4_sphinx_theme:`Sphinx Bootstrap4 Theme <>`. Its Bootstrap options
remain available only in that compatibility mode.

Overriding Origen's Theme
^^^^^^^^^^^^^^^^^^^^^^^^^

If something in Origen's theme is not to your liking, your Sphinx app can
:sphinx_templating:`override templates <>` used by Origen's theme. To inherit from Origen's
templates, only overriding aspects given by your project's template,
:sphinx_creating_themes:`inherit from <>` ``origen/web/sphinx_ext/theme/<template>.html``.

The active parent templates come from ``pydata_sphinx_theme`` on current
Python versions and from :bootstrap4_sphinx_theme:`sphinxbootstrap4 <>` in the
legacy compatibility mode. Keep overrides limited to stable Sphinx template
blocks where possible, since parent-theme internals can change between releases.

Extending Themes
^^^^^^^^^^^^^^^^

Sphinx allows for :sphinx_creating_themes:`one theme to extend another <>`. The Origen theme is
no exception, allowing for alterations without entirely breaking away from it - maintaining
|ose| features which rely on the theme.

Choosing A Different Theme
^^^^^^^^^^^^^^^^^^^^^^^^^^

Although picked for you during the Origen application creation, the Origen theme is completely optional.
:sphinx_using_a_theme:`Setting the theme <>` in your config will override and decouple your webpages
from the Origen theme entirely but at the expense of the aforementioned
|ose_theme_opts| (or at least in their current form).

All of the Origen-theme specifics are contained within the ``html_theme_options`` configuration setting,
so breaking away from the Origen theme will only impact those options. 

Though a bit more involved, it is encouraged to inherit from Origen's theme where possible rather
than break away from it entirely, the former of which will maintain the highest degree of functionality.

.. note::

   Setting the theme to ``None`` in your ``conf.py`` will revert your
   theme to Sphinx's default: :sphinx_alabaster_theme:`the Alabaster theme <>`.

Recap
-----

* Origen and the |ose| aim to allow for as much flexibility as possible while maintaining the highest
  degree of functionality.
* The |ose_theme| can be extended, allowing for a project-specific vibe without having to entirely
  ditch the features Origen's theme provides.
* On current Python versions the extension configures and brands the PyData
  Sphinx Theme; the bundled Bootstrap-derived theme remains the fallback for
  older supported Python versions.
* If you do opt to move away from Origen's theme entirely, some of the |ose| features will not
  work properly but the general integration between the
  *Origen application*, *Sphinx app*, and Origen CLI will still be maintained.

  That is to say, ``origen web build`` will still work.
