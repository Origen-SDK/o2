The Shorthand Extension
=======================

``Shorthand`` is an custom |sphinx_ext| which enables easier referencing and tracking of
|sphinx_refs|, |rst_subs|, static assets, and external links.

Basic Usage
-----------

``Shorthand`` is configured via the ``shorthand_defs`` |sphinx_config_var|, which defaults to ``None``.
The configuration maps ``the shorthand definitions`` - or the user-friendly names - to ``targets``.
``Shorthand`` supports various target interpretations, which are denoted by placing the
``definition`` into enumerated :link-to:`categories <shorthand~categories>` - a commonly used
``category`` being ``refs``, which provides shorthand syntaxes for |sphinx_refs|.

|sphinx_refs| are themselves already a type of shortcut, but using something like |autosectionlabel| 
with |autosectionlabel_prefix_document| in a large project can yield long, difficult to track paths.
``Shorthand`` can help by shortening these paths and providing a single source where large scale
updates can be easily made.

In its simplest form, the |shorthand~config_var| can just be a |dict| of nested |dicts|, where the nested
|dict| corresponds to a specific :link-to:`category <shorthand~categories>`:

.. code:: python

  ''' Shorthand Refs Example '''
  shorthand_defs = {
    'refs': {
      'my_page': 'root/section1/section2/my_page:My Title',
    }
  }

The example definition ``my_page`` will be interpreted as a |sphinx_ref| by residing in the ``refs category``.
When resolved, the target ``root/section1/section2/my_page:My Title`` will be resolved as
``:ref:`my_page <root/section1/section2/my_page:My Title>```. Notice that is just a |sphinx_ref| - nothing
special has been done - just a substitution.

When running ``sphinx-build``, the |shorthand~config_var| will be parsed and transformed into an ``RST source``
containing these definitions as |rst_subs|. The resulting output file (which will be in your
``sphinx source directory`` by default) will look like:

.. code::

  .. Substitution definitions derived from Shorthand

  :orphan:

  .. start-content

  .. |my_page| replace:: :ref:`my_page <root/section1/section2/my_page:My Title>`

Using the |rst_include_directive|, these substitutions will be available in your RST sources. Then,
the above can be invoked from your source as just ``|my_page|``.

Adding Text
^^^^^^^^^^^

The text associated with ``my_page`` isn't the most formal and you'll probably want *better* titles in your
references. A ``refs`` definition can also point to a |tuple| where the first element is the text, or title,
to be used, and the second element is the target. Updating the above to:

.. code:: python

  ''' Shorthand Refs Example '''
  shorthand_defs = {
    'refs': {
      'my_page': (
        'My Page',
        'root/section1/section2/my_page:My Title',
      )
    }
  }

Yields a substitution:

.. code::

  .. |my_page| replace:: :ref:`My Page <root/section1/section2/my_page:My Title>`

Which resolves to ``:ref:`My Page <root/section1/section2/my_page:My Title>``` when invoked.

Multiple Text Entries Per Target
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

As the |shorthand~config_var| is just a |dict| and |conf.py| is just a Python file, multiple substitutions
can be easily stacked up while maintaining the same *single-source* mentality:

.. code:: python

  ''' Shorthand Refs Example '''
  my_page = 'root/section1/section2/my_page:My Title'
  shorthand_defs = {
    'refs': {
      'my_page': ('my page', my_page),
      'my_page_caps': ('My Page', my_page),
    }
  }

The :link-to: Role
^^^^^^^^^^^^^^^^^^

|RST_subs| are nice, but they are limited in that small variations are not possible. The above
showed how multiple substitutions can be easily created, but this also means more substitutions to keep
track of.

Shorthand also provides a ``:link-to:`` role which works much like :sphinx_ref_role:`Sphinx's ref role <>`.
Sphinx allows for text to passed to a ``:ref:`` invocation and ``:link-to:`` does the same, just accepting
a *shorthand target* instead a reference target. To give the ``my_page`` reference custom text:

.. code::

  :link-to:`Custom Text <my_page>`

Categories
----------

``Shorthand`` supports more than just ``refs``. The following categories are
also available:

* ``statics`` - Static items in Sphinx require paths relative to the current
  document. This makes reuse a bit tricky as the path can change depending on
  which doc is being processed. Shorthand's ``statics`` will handle the
  relative linking for you, as well as provide some consistency checks ensuring
  that the static target is valid. When defining statics, the *target* should
  be defined relative to the ``Sphinx app root``. In most cases this will be
  something like ``./_static/my_static_item.ext``.
* ``docs`` - Like statics, the |sphinx_doc_role| requires paths relative to the current
  document and, like statics, a ``doc`` definition will
  resolve to the path relative to the current document when processing.
  ``docs`` are defined similarly to ``statics`` and should be defined
  relative to the ``Sphinx app root.``
* ``abslinks`` - Absolute URLs, usable in HTML anchors
  (see also :link-to:`usage in templating <shorthand~templating>`).
* ``extlinks`` - Similar to ``abslinks`` but targets are pulled from
  :extlinks_home:`Sphinx's extlinks extension <>` defined in the app.
* ``substitutions`` - Resolved as straight |rst_subs|.

The categories above will dictate how the |rst_subs| and the ``:link-to:`` roles are resolved:

.. code:: python

  ''' Shorthand Refs Example '''
  shorthand_defs = {
    'refs': {
      'my_page': ('my page', 'root/section1/section2/my_page:My Title'),
    },
    'docs': {
      'my_pdf': '_static/my_pdf.pfd'
    },
    'substitutions': {
      'title': 'My Page Title'
    }
  }

.. code::

  .. |my_page| replace:: :ref:`My Page <root/section1/section2/my_page:My Title>`
  .. |my_pdf| replace:: :link-to:`My Page <_static/my_pdf.pfd>`
  .. |title| replace:: My Page Title

See the :link-to:`API <shorthand~categories_var>` for a full list of the
available categories.

Definition Organization
-----------------------

A definition within a category can also be a |dict| of nested *definitions* to
more easily organize large lists:

.. code:: python

  ''' Shorthand Refs Example '''
  shorthand_defs = {
    'refs': {
      'my_page': ('my page', 'root/section1/section2/my_page:My Title'),
      'sub1': {
        'page1': (
          'Page 1',
          'root/sub1/page1:Page 1'
        ),
        'page2: (
          'Page 2',
          'root/sub2/page2:Page 2'
        )
      },
      'sub2': {
        'page1: (
          'Page 1',
          'root/sub1/page1:Page 1'
        )
      }
    }
  }

Nested definitions will have the key prefixed and can be addressed by
separating the prefixed name with a colon (``:``). For example,
the above definitions can be addressed as:

.. code::

  |my_page|
  |sub1:page1|
  |sub1:page2|
  |sub2:page1|

Definition Namespaces
---------------------

Adjacent to the definition organization, is ``namespacing``. Instead
of being isolated to a specific category, ``namespaces`` are an entirely
separate set of definitions.

The |shorthand~config_var| can also be a |list| of |dicts|, where each
individual |dict| in the |list| acts as it's own set of *shorthand definitions*. The
``namespace`` key gives the set of definitions their name and can be addressed
by prefixing the namespace and the tilde (``~``) character:

.. code:: python

  ''' Shorthand Refs Example '''
  shorthand_defs = [
    {
      'namespace': 'n1',
      'refs': {
        'my_page': ('my page', 'root/n1/my_page:My Title'),
      }
    },
    {
      'namespace': 'n2',
      'refs': {
        'my_page': ('my page', 'root/n2/my_page:My Title'),
      }
    }
  ]

These are addressed as:

.. code::

  |n1~my_page|
  |n2~my_page|

Project Namespace
^^^^^^^^^^^^^^^^^

A list of definitions can have only one unnamed, or project, set of definitions.
Subsequent definitions must be namespaced.

Some configuration elements for namespaced definitions will also act as children
of the project namespace. For example, if a namespaced definition |dict| does
not have an ``output_dir`` setting, it will inherit it from the project
namespace (see the :link-to:`next section <shorthand~config_keys>` for
details on this setting).

For another application of namespacing, see the |shorthand~multidefs| section.

Other Configuration Keys
------------------------

Outside of the |shorthand~categories| and ``namespace`` keys, the following
configuration options are also available:

* ``output_name`` - Indicates the filename with the resulting RST definitions.
* ``output_dir`` - Indicates the output directory for the resulting RST definitions.

Using Multiple Shorthand Definitions
------------------------------------

Building off of |shorthand~namespaces|, another useful application is to accept ``shorthand_defs`` from
other sources and have them available in the current project - preferably without clashing.

This could be done by other extensions or the |sphinx_app| trying to dynamically alter the
|shorthand~conf_var| but this can get clunky and load-order dependencies may introduce themselves when
a sort of ``definition inheritance`` is used.

``Shorthand`` provides the |shorthand~add_defs| to register definitions from an external source. See the
:link-to:`API entry <shorthand~add_defs>` for its usage.

Usage In Templating
-------------------

The |shorthand~basic_usage| section showed how to use |shorthand| in RST sources. However, ``shorthand``
provides some methods which may be helpful during templating or dynamically building RST sources. The methods
|shorthand~anchor_to|, |shorthand~href_to|, and |shorthand~link_to| provide an interface into the definitions
from outside RST sources. See the |shorthand~api| for more details.
