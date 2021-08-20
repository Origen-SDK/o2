import pathlib, sys, requests
from typing import Any, Dict, List, Tuple
from sphinx.util.nodes import split_explicit_title, _make_id
from docutils import nodes, utils
from docutils.nodes import Node, system_message
from docutils.parsers.rst.states import Inliner
from sphinx.addnodes import pending_xref
from . import shorthand
from .shorthand import all_include_rsts

app = None
_unchecked_targets = set()


def check_link(link, *, warning_prefix=None):
    ''' Checks if the given link points to *somewhere* '''
    try:
        req = requests.get(link)  # = urllib.request.urlopen(link)
        if not (200 <= req.status_code < 300):
            if warning_prefix:
                shorthand.logger.warn(
                    f"{warning_prefix}: Invalid link '{link}' - Received non-2xx status code: {req.status_code}"
                )
            return (False, req.status_code)
    except requests.exceptions.BaseHTTPError as e:  # urllib.error.URLError as e:
        if warning_prefix:
            shorthand.logger.warn(
                f"{warning_prefix}: Invalid link '{link}' - Received error {e.__class__} with message {str(e)}"
            )
        return (False, e)
    return (True, req.status_code)


def update_unchecked_targets(target):
    _unchecked_targets.add(target)


def check_consistency(app, env):
    ''' 
    Targets here bypass Sphinx's ref/doc/whatever checkers and could potentially allow
    for bad targets to make its way into the resulting pages. Other methods will note
    these potential targets and they'll be checked here after all the documents have been read.

    Only 'refs', 'docs', or 'statics' should make there way here - unless others are added in the future.
  '''
    for target in _unchecked_targets:
        t = shorthand.get(target)
        if t.is_ref:
            # Checking a ref amounts to ensuring it exists in the label table after all is
            # said and done.
            # We'll assuming that the cleaned target (href-ified target) is valid provided the
            #   label is valid.
            if not t.target in app.env.get_domain('std').labels:
                shorthand.logger.warning(
                    f"{t.name} - Undefined label '{t.target}'")
        elif t.is_doc or t.is_static:
            # Check that any docs or other statics exist in the workspace at the path specified,
            # relative to the src dir.
            # For these, the target URI would have been updated as a relative path from the doc,
            # but we'll assume that, provided the doc exists, that the relative path is correct
            if not any(
                    p.match(f"{t.target}*") for p in list(
                        pathlib.Path(app.srcdir).joinpath(
                            t.target).parent.glob('*'))):
                shorthand.logger.warning(
                    f"{t.name} - Undefined doc or static asset '{t.target}'")
        else:
            shorthand.logger.error(
                f"Cannot check consistency for target '{t.name}' category '{t.category}'"
            )

    if app.config.shorthand_check_links:
        # Check extlinks and abslinks
        if 'sphinx.ext.extlinks' in app.extensions and app.config.extlinks:
            shorthand.logger.info("Checking extlinks consistency...")
            for name, target in app.config.extlinks.items():
                t = target[0] % target[1]
                shorthand.logger.info(f"-- {name}: {t}")
                check_link(t, warning_prefix=f"(extlink - {name})")

        shorthand.logger.info("Checking abslinks consistency...")
        for _nspace, abslinks in shorthand.all_from_category(
                'abslinks').items():
            for name, target in abslinks.items():
                shorthand.logger.info(f"-- {name}: {target.target}")
                check_link(
                    target.target,
                    warning_prefix=f"(target {name} of category 'abslinks')")


def href_to(target, *, docname=None):
    '''
    Generates an href for the target. The exact nature is determined by the HREF type.

    * 'ref' - target is 'cleaned'. E.g: 'dir/my_label:header' -> '<relative_path>/my_label#header'
    * 'doc/static' - target is re-targetted as a relative path from the callin document.
    * 'absolute_links' - just returns the target
  '''
    t = shorthand.get(target)
    if t is None:
        shorthand.logger.error(
            f"Unknown shorthand target from {app.env.docname} - '{target}'")
    else:
        if t.is_ref:
            # Use sphinx's built-in methods to 'href-ify' the target
            # the non-href-ified target should appear in the label table during consistency checking
            if ':' in t.target:
                doc, loc = t.target.split(":", 1)
                loc = _make_id(loc)
                doc = app.builder.get_relative_uri(docname or app.env.docname,
                                                   doc)
                return f"{doc}#{loc}"
            else:
                return app.builder.get_relative_uri(docname or app.env.docname,
                                                    t.target)
        elif t.is_doc or t.is_static:
            # Generate an HREF to the asset relative to where we are.
            # These too can be checked later
            return app.builder.get_relative_uri(docname or app.env.docname,
                                                t.target)
        elif t.is_abslink:
            return t.target
        elif t.is_extlink:
            extlink = app.config.extlinks.get(t.target, None)
            if extlink is None:
                shorthand.logger.error(f"Cannot find extlink {t.target}!")
            else:
                return extlink[0] % extlink[1]
        elif t.is_sub:
            shorthand.logger.error(
                "Cannot generate HREF for a substitution type!")
        else:
            shorthand.logger.error(
                f"Unknown category '{t.category}' for target '{t.name}'")
    return '#'


def anchor_to(target, txt):
    ''' Although in this phase we *could* generate RST, we don't actually know the context
      in which this method is used. Mostly likely, this will actually be invoked from an
      ``.. raw`` block or the like. To skip over needing the context, or trying to figure
      out what the 'typical' usage should be, we're just going to jump straight to HTML.
      We'll cache the targets and ensure they're valid during the consistency check phase.
  '''
    return f"<a href='{href_to(target)}'>{txt}</a>"


def link_to(target, txt=None, **opts):
    ''' Inserts a ``shorthand-link-to`` role with the given target and txt '''
    if txt is None:
        return f":shorthand-link-to:`{target}`"
    else:
        return f":shorthand-link-to:`{txt} <{target}>`"


def templating_context(app):
    return {
        'shorthand':
        sys.modules[__name__],
        'link_to':
        lambda target, txt=None, **opts: link_to(target, txt, **opts),
        'href_to':
        lambda target, **opts: href_to(target, **opts),
        'anchor_to':
        lambda target, txt=None, **opts: anchor_to(target, txt, **opts),
    }


def _link_to(
        typ: str,
        rawtext: str,
        text: str,
        lineno: int,
        inliner: Inliner,
        options: Dict = {},
        content: List[str] = []) -> Tuple[List[Node], List[system_message]]:
    '''
    Generates a reference for the given target from the shorthand_defs table.
    The category (i.e., hash-key) that the target resides under will determine the type of reference
    generated. For example, a target under the ``refs`` section will generate ``:ref:`` roles,
    whereas a static will generate a standard link `` `... <link-to-static>`_ ``

    Since ``:doc:`` roles, like ``static`` items, must be relatively linked, referencing a ``doc``
    places a static link to the given doc.
  '''
    text = utils.unescape(text)
    _has_explicit_title, title, target = split_explicit_title(text)
    t = shorthand.get(target)
    if t is None:
        # Target could not be found. Print an error
        shorthand.logger.error(
            f"Unknown shorthand target from {app.env.docname}:{lineno} - '{target}'"
        )
        n = nodes.reference(title, title, internal=True, refuri='')
    elif t.is_ref:
        gen = app.env.get_domain('std').roles['ref']
        gen.target = t.target
        gen.title = title
        return gen.run()
    elif t.is_static or t.is_doc:
        # Statics - and docs linked in the same way - are not checked for consistency.
        # We'll note any added docs or statics and check them ourselves during the appropriate phase.
        update_unchecked_targets(target)

        # static assets must be pointed to directly, but is also relative to the current docname
        # Resolve the static item relative to the current docname and push the new, generic reference, node
        uri = app.builder.get_relative_uri(app.env.docname, t.target)
        n = nodes.reference(title, title, internal=True, refuri=uri)
    elif t.is_extlink or t.is_abslink:
        n = nodes.reference(title,
                            title,
                            internal=False,
                            refuri=href_to(target))
    elif t.is_api:
        n = pending_xref('',
                         nodes.Text(title or t.target),
                         refdomain='py',
                         reftype='obj',
                         reftarget=t.target)
    else:
        shorthand.logger.error(
            f"Shorthand target '{target}' of category '{t.category}' does not support shorthand links"
        )
        n = nodes.reference(title, title, internal=True, refuri='')
    return ([n], [])


def add_defs(defs, *, namespace=None):
    ''' 
    Dynamically adds **namespaced** definitions and generates the appropriate output files.

    Defintions added here must be namespaced. Adding to the project definitions is not supported.
  '''
    if namespace:
        # Update the defs' namespace
        defs['namespace'] = namespace
    nspace = defs.get('namespace', None)
    if nspace is None:
        shorthand.logger.error(
            "Dynamically added definitions must be namespaced! " +
            "Please either add an 'namespace' key or provide an " +
            "additional 'namespace' keyword arg.")
    elif nspace in shorthand.shorthand_defs:
        shorthand.logger.error(
            f"Definitions have already been added with namespace '{nspace}'")
    else:
        shorthand.shorthand_defs[nspace] = shorthand.ShorthandDefs(app, defs)


def set_default_output_dir(dirname):
    shorthand.default_output_dir = pathlib.Path(dirname)
    return shorthand.default_output_dir


def setup(sphinx):
    '''
    Callback for Sphinx to setup the extension.
  '''

    global app
    app = sphinx

    sphinx.add_config_value('shorthand_defs', None, '')
    sphinx.add_config_value('shorthand_check_links', False, '')
    sphinx.connect('builder-inited', shorthand.generate)
    sphinx.connect('env-check-consistency', check_consistency)

    for new_role in [
            'link_to', 'link-to', 'shorthand-link-to', 'shorthand-link_to'
    ]:
        sphinx.add_role(new_role, _link_to)
