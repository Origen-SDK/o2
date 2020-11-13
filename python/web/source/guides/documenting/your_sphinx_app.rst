Your Sphinx App
===============

Writing docs for your *Origen application* amounts to writing docs as you would for any other *Sphinx app*.

You may or may not have experience using Sphinx, but if you're an experienced or intermediate
Sphinx user, you could skip to the |ose| which covers the heart of the Origen-Sphinx connection.
Although this section covers some Origen-specifics, the changes described here will be apparent
to any seasoned Sphinx users.

----

From :link-to:`the previous section <documenting:core_concepts>`:

.. raw:: html

  <div class="card text-white bg-primary mb-3">
    <div class="card-body">
      <blockquote class="quote-card">
        <p>
          At its heart, your <i>Origen application's</i> documentation 'engine' is just
          a <i>Sphinx app</i> with a custom extension thrown in.
        </p>
        <cite>{{ anchor_to('documenting:core_concepts', 'Core Concepts') }}</cite>
      </blockquote>
    </div>
  </div>

.. raw:: html

  <div class="card text-white bg-primary mb-3">
    <div class="card-body">
      <blockquote class="quote-card">
        <p>
          Writing docs for your <i>Origen application</i> amounts to writing
          docs like you would any other <i>Sphinx app</i>.
        </p>
        <cite>{{ anchor_to('documenting:core_concepts', 'Core Concepts') }}</cite>
      </blockquote>
    </div>
  </div>

This first point may sound a bit ominous and make you skeptical the second point. This page aims to put
that to rest while delegating most of the *actual sphinx usage* back to the :sphinx_docs:`Sphinx docs <>`
themselves to cover items that are best learned from the source, rather than paraphrased here.

Origen's Sphinx App
-------------------

Origen applications include a pre-configured setup for :sphinx_homepage:`Sphinx <>`. Before delving into
the Sphinx docs though, its important to know what customizations Origen has made, placing your
pre-configured *Sphinx app* commensurate with an app from Sphinx's *quickstart*,
which the latter's documentation will assume.

Sphinx And The Origen CLI
^^^^^^^^^^^^^^^^^^^^^^^^^

It is important to note that Sphinx has its own :sphinx_manpages:`command-line interface <>`. The ``origen web`` command wraps
Sphinx's commands and invokes them such that Origen's requirements are met. That's really all that needs
to be known for now: the Origen CLI automates calls to the Sphinx CLI.

For the curious, see the :sphinx_manpages:`Sphinx CLI <>` for details on Sphinx's commands.

Sphinx Quickstart
^^^^^^^^^^^^^^^^^

Sphinx's CLI includes a ``quickstart`` command which will build some default files for you. When you ran
``origen new``, some of the files built mimic those from Sphinx's quickstart.
One key difference is *where* those files are built.

Origen places the *Sphinx app* a bit further removed than where ``quickstart`` would: in
``web/source``, as opposed to just ``./source``. Navigating to ``web/source`` though, you'll see the
same files ``sphinx-quickstart`` would have given you: most notably, |conf.py| and ``index.rst``.
These are the same files which are referenced frequently in Sphinx's docs. Even though ``origen new`` fills
in some content for you, these still function as Sphinx describes them and all the same options
remain available.

To quickly define these files:

* |conf.py| is the configuration file for your *Sphinx app* and will include things like extensions,
  extension setup, custom functions, etc. See the :sphinx_conf:`sphinx topic` for more information.
* ``index.rst`` is the default homepage for the resulting website and is also the page launched when
  the ``--view`` switch is used with ``origen web build``.

A key addition to note is the automatic inclusion of the |ose| as the
first (topmost) extension in |conf.py|. This extension will be
:link-to:`covered in much more detail later <ose>`, but just know for now that it is responsible
for all of the *Origen specifics* which separate a standard Sphinx app from one used in
an Origen application.

Other extensions, such as :autoapi_home:`autoapi` and :autodoc_home:`autodoc` are also included,
but those are more for convenience.
:link-to:`Notes on these will also be covered later <documenting:api_generation>`.

.. raw:: html

  <div class="alert alert-info" role="alert">
    When we say "it is responsible for all of the <i>Origen specifics</i>", we mean it! Removing this
    extension will return you to a default app, as constructed by <i>sphinx-quickstart</i>.
    This may be what you want - if you want complete control from the ground up - but you will lose
    the interactions available from Origen in the broader sense.
    <br><br>

    For example, many of the <code>origen web build</code> switches and features rely on the
    <code>origen_sphinx_extension</code>. Removing this extension without implementing the associated
    functionality yourself will cause those items to not behave properly (if at all).
    <br><br>

    The <code>origen_sphinx_extension</code> has a number of customizations available and can be inherited
    or overridden like any other Sphinx extension. This will be
    {{ anchor_to('ose', 'covered in more detail later') }} but this mentality should be
    preferred over removing the extension entirely.
  </div>

  <div class="alert alert-danger" role="alert">
    Moving this extension around in the load order will have unknown effects, almost all of which
    will be bad. For utmost compatibility, it should remain as the first extension enabled.
  </div>

Adding Content
--------------

Understanding now that your *Origen application's* documentation is really just a pre-configured
*Sphinx app* with the |ose| already included, you can begin adding content.
Origen includes some additions here but it also does not get in the way of Sphinx's regular flow.

Sphinx content primarily uses :sphinx_rst:`restructured text (RST) <>`, which serves both to link
documents together and format the actual content. Tutorials on ``restructured text`` are out of scope here,
as Sphinx and the RST official website are abound with :sphinx_rst_primer:`primers <>`,
:rst_quickstart:`tutorials <>`, and more in-depth :rst_docs:`documentation <>`
that will cover more ground than we ever could.

To restate once again, even though we have a customized *Sphinx app*, all the content there
is applicable here. That said, your *Sphinx app* has some bonus items thrown in by default...

Markdown
^^^^^^^^

Adjacent to *restructured text* is another popular markup language: :markdown_home:`markdown <>`.
Depending on your background, or how involved you are in blogs and social media websites
(Markdown is popular in those spaces), you may
already have experience using Markdown but none using RST and wish to continue using Markdown to
write content. A Sphinx extension, :recommonmark_home:`recommonmark <>` is available to build
Markdown content for Sphinx apps and Origen comes with this already installed and configured.
The |ose| will configure your Markdown to accept
:recommonmark_embedded_rst:`embedded RST <>`, allowing you to place 
:sphinx_rst_directives:`RST directives Sphinx uses <>` inside your Markdown documents as well.

See the :recommonmark_home:`recommonmark <>` docs for more information.

Templates
^^^^^^^^^

You may have already come across :sphinx_templating:`templating <>` in your Sphinx reading.
In case you haven't, *templates* allow for content to be
dynamically resolved in your documentation. :jinja_home:`Jinja <>`, Sphinx's templating language of
choice, comes already installed as well. Like RST, Markdown, and Sphinx in general, tutorials on
Jinja will not be covered here, but head over to the :jinja_docs:`Jinja documentation <>` to get started.

.. raw:: html

  <div class="alert alert-primary" role="alert">
    Origen applications come pre-configured to invoke the Jinja processor on all RST templates,
    as well as any of the content in the <code>_templates</code>, or other added
    <code>template directories</code>.
    <br><br>

    Default Sphinx only runs the template engine on the latter.
  </div>

.. Templates inside of your pre-configured Sphinx app work just the same as any other Sphinx app. Origen does, however, through some
  additional items available in your templates. By default, Sphinx tosses in `these items <>`_ when building templates. For general
  apps, these are usually sufficient, but we may need additional context. The `origen_sphinx_ext` will also provide you with `origen`,
  booted up as normal, which you can use to dynamically place content in your applications.
  With the `origen` module at your disposable, you can, for instance, `instantiate targets <>`_ and dynamically add content
  such as `pins <>`_, `registers <>`_, or anything else!

Extensions
----------

As has been alluded to several times, Sphinx has the concept of :sphinx_extensions:`extensions <>`, which are
additional libraries that are plugged into Sphinx to give increased functionality, additional features,
or offer more customization. We've brought up the |ose| a few times, and its definition
is coming up shortly but we've also described the :recommonmark_home:`recommonmark extension <>`
extension, which is brought in and configured automatically. 

Section Labels
^^^^^^^^^^^^^^

Your |sphinx_app| will automatically include and enable Sphinx's |autosectionlabel| extension,
which creates |sphinx_refs| for each section within your documentation. These references can
then be used as normal |sphinx_app| and/or integrated with |shorthand|.

API generation can induce conflicts in the section labeling. The |autosectionlabel| extension has
a |sphinx_config_var| to append the full file path to the section, resolving these conflicts. The
variable, |autosectionlabel_prefix_document|, is enabled by default. This setup can be altered or
removed entirely in your :link-to:`sphinx app's <sphinx_app>` |conf.py|.

Automatic API Generation
^^^^^^^^^^^^^^^^^^^^^^^^^

Your |sphinx_app| includes two more extensions: :autoapi_home:`AutoAPI <>`, which will cycle
through your top-level module searching for Python objects and |docstrings| - building RST files out of them,
and :autodoc_home:`autodoc <>` which will parse the resulting RST files from *AutoAPI* into viewable content.

.. raw:: html

  <div class="alert alert-warning" role="alert">
    AutoAPI works by iterating through the <b>built module</b>, not just by parsing the files. This means
    that your application, and all connected modules, classes, etc., must at least load correctly in
    Python for AutoAPI to run to completion.
  </div>

When your *Origen application* is built, AutoAPI will be automatically added as an extension, with your
application's namespace as a target. This setup, though automatic, is done by during
*Origen application* creation and can be easily customized, or removed entirely, from
your |conf.py|. See the :autoapi_usage:`usage section <>` present in its documentation
for more on ``AutoAPI``.

.. raw:: html

  <div class="alert alert-info" role="alert">
    APIs can take some time to parse and build, especially for larger projects. For quicker turnaround,
    the <code>--no-api</code> switch can be given to the build command to bypass running this extension
    for that particular build.
    <br><br>

    AutoAPI will always rebuild the APIs by default, but contents from a previous run will persist from
    run to run. Assuming no changes to the source, <code>--no-api</code> can be used after an initial
    build without any adverse effects to these extensions.
    <br><br>
  </div>

Docstring Formatting
^^^^^^^^^^^^^^^^^^^^

Your |sphinx_app| also comes with |napoleon| already enabled, which allows you write |docstrings| according
to either the |numpy_docstring_spec| or |google_docstring_spec|.
:link-to:`Napoleon <napoleon>` can be further configured, or removed entirely, in your |conf.py|.

Themes
------

:sphinx_extensions:`Extensions <>` are geared towards adding *functionality* to your Sphinx app and
to give you better tools with which to write content.
However, very little has been said about what gives your generated webpages their look, style, or flair.
For this, Sphinx uses :sphinx_themes:`themes <>` and, like so many other aspects, Origen has a hook for that.

Before jumping into Origen's theme, take a moment to glance through some of
:sphinx_builtin_themes:`Sphinx's built-in themes <>`. Although we've already
chosen one for you, the examples there should show you exactly what is meant by the *look and feel* of
your webpages. You can also view the vast amount of :sphinx_available_themes:`third-party themes <>`
Sphinx has available.

Recap
-----

* Your *Sphinx app* in your *Origen application* is a standard *Sphinx app* with some setup already done for you.
* Most notably, the inclusion of the |ose|.
* However, writing docs for your *Sphinx app* is no different than writing docs for any other *Sphinx app*.
* *Extensions* allow for other libraries to plug into Sphinx and offer additional features.
* Some other extensions included automatically are |recommonmark|, |autoapi|, and |autodoc|.
* Sphinx also has themes, which focus on the look and feel of your website.

Reference Material
^^^^^^^^^^^^^^^^^^

The following reference material will help you understand *Sphinx*, *RST*,
*extensions* and other material pertinent to writing content for your project.

* :sphinx_app:`Sphinx Tutorial <>`
* :sphinx_rst_primer:`Sphinx's RST Primer <>`
* :rst_docs:`RST Reference <>`
* :sphinx_extensions:`Sphinx Extensions <>`
* :autoapi_home:`AutoAPI <>`
* :autodoc_home:`Autodoc <>`
* :sphinx_themes:`Sphinx Themes <>`
* :sphinx_available_themes:`Example Themes <>`
