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

First, recall that Origen's theme is not built from scratch but an 
:sphinx_creating_themes:`extension <>` of another theme, the
:bootstrap4_sphinx_theme:`Sphinx Bootstrap4 Theme <>`, which partially gives the
webpages their look and enables :bootstrap4_widgets:`Bootstrap4 widgets <>`
out of the box.

The *Bootstrap4 theme* also has its own set of
:bootstrap4_sphinx_theme_options:`html_theme_options <>`.
Most of the user-facing ones have been hijacked by the *origen theme* (See the |ose_theme_opts| for more),
but others supported by the *Bootstrap4 theme* are also fair game for customization.

Overriding Origen's Theme
^^^^^^^^^^^^^^^^^^^^^^^^^

If something in Origen's theme is not to your liking, your Sphinx app can
:sphinx_templating:`override templates <>` used by Origen's theme. To inherit from Origen's
templates, only overriding aspects given by your project's template,
:sphinx_creating_themes:`inherit from <>` ``origen/web/sphinx_ext/theme/<template>.html``.

Since Origen inherits from the :bootstrap4_sphinx_theme:`sphinxbootstrap4 <>` theme,
templates to extend that may :bootstrap4_sphinx_theme_templates:`reside there as well <>`.

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

.. raw:: html

  <div class="alert alert-info" role="alert">
    Setting the theme to <code>None</code> in your <code>conf.py</code> will revert your
    theme to Sphinx's default: 
    <a href="{{ app.config.extlinks['sphinx_alabaster_theme'][0]|replace('%s', '') }}">the Alabaster theme</a>.
  </div>

Recap
-----

* Origen and the |ose| aim to allow for as much flexibility as possible while maintaining the highest
  degree of functionality.
* The |ose_theme| can be extended, allowing for a project-specific vibe without having to entirely
  ditch the features Origen's theme provides.
* The |ose_theme| itself :sphinx_creating_themes:`extends <>` the
  :bootstrap4_sphinx_theme:`sphinxbootstrap4 theme <>`.
* If you do opt to move away from Origen's theme entirely, some of the |ose| features will not
  work properly but the general integration between the
  *Origen application*, *Sphinx app*, and Origen CLI will still be maintained.

  That is to say, ``origen web build`` will still work.
