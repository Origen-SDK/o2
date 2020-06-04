import origen, subprocess, builtins, types, inspect, re, pathlib
from .. import origen_sphinx_extension as ose

import importlib

def preprocess_src(app, docname, source):
  # Origen's compiler supports multiple templating engines as well as a different context
  # than Sphinx's Jinja engine.
  # The Jinja compiler will be run on every source from Sphinx, unless the Origen compiler matches.
  # In order to be detected from Origen though, the file must be appended with the template engine.
  # For example:
  #   index.rst -> Processed with Sphinx's Jinja interface
  #   index.rst.mako -> Processed with Origen's mako interface
  #   TODO: index.rst.jinja -> Processed with Origen's jinja interface
  syntax = origen.app.compiler.select_syntax(app.env.doc2path(docname))
  if syntax:
    source[0] = origen.app.compiler.render(
      source[0],
      syntax=syntax,
      direct_src=True,
      context=jinja_context(app)
    )
  else:
    jinja_integrator(app, docname, source)

def docname_exts(app, docname):
  return pathlib.Path(app.env.doc2path(docname)).suffixes

def sphinx_ext(ext_name, app):
  if ext_name in app.extensions:
    return importlib.import_module(app.extensions[ext_name].module.__name__)

def insert_header(app, docname, source):
  '''
    TODO(coreyeng) This could check for the contents in the event of multiple matching files.
      We'd have the filename and actual source, but seems like overkill at the moment.
  '''
  doc = pathlib.Path(docname)
  #for pat in app.config.origen_content_header.get('exclude_patterns', []):
  #  if doc.match(pat):
  #    return False
  if '.rst' in docname_exts(app, docname):
    rst_ext = sphinx_ext('origen.web.rst_shared_defs', app)
    if rst_ext:
      shared_defs = rst_ext.all_shared()
      # Make sure we aren't including the shared file in the shared files themselves
      if not any((s.parent.match(str(doc.parent)) if not str(doc.parent) == "." else False) for s in shared_defs):
        if 'insert-rst-shared-defs' in app.config.origen_content_header:
          source[0] = "\n".join([f".. include:: {shared}\n  :start-after: start-content\n\n" for shared in shared_defs]) + source[0]
        return True
  return False

# Setup taken from here: https://www.ericholscher.com/blog/2016/jul/25/integrating-jinja-rst-sphinx/
@origen.helpers.continue_on_exception(ose.logger)
def jinja_integrator(app, docname, source):
  src = source[0]
  import builtins, types, inspect
  try:
    rendered = app.builder.templates.render_string(src, jinja_context(app))
    source[0] = rendered
  except Exception as e:
    m = getattr(e, 'message', repr(e))
    raise type(e)(f"Exception occurred processing {docname}{': ' + m if m else ''}") from e

def jinja_render_string(app, src, additional_context={}):
  return app.builder.templates.render_string(src, jinja_context(app))

def jinja_context(app):
  return {
    **builtins.__dict__,
    **types.__dict__,
    **inspect.__dict__,
    **{
      'origen': origen,
      'origen_sphinx_extension': ose,
      'origen_sphinx_app': app,

      # The remaining are non-namespaced helpers which may conflict with other context shifted in
      # from the user's conf.html_context
      # (This is why 'app' and 'ose' are listed twice, both as a namespaced and non-namespace option)
      'app': app,
      'ose': ose,
      'ref_for': lambda ref, txt=None, **opts : ref_for(app, ref, txt, **opts),
      'path_to': lambda ref, **opts: path_to(app, ref, **opts),
      'insert_cmd_output': lambda cmd, **opts: insert_cmd_output(app, cmd, **opts),
      'anchor_for': lambda ref, txt=None, **opts: anchor_for(app, ref, txt, **opts),
      'href_for': lambda ref, **opts: href_for(app, ref, **opts)
    },

    # Load in the user's html_context last, overwriting anything we've already added
    **app.config.html_context,
  }

def rst_shared_defs(app):
  return sphinx_ext('origen.web.rst_shared_defs', app)

def find_def(app, ref, group):
  d = rst_shared_defs(app).find(ref, group=group)
  if d is None:
    raise LookupError(f"Could not locate definition for '{ref}'")
  return d

def ref_for(app, ref, txt=None, *, group=None, **opts):
  target = find_def(app, ref, group)
  return f":ref:`{txt if txt else target[0]} <{target[1]}>`"

def path_to(app, ref, *, group=None,  **opts):
  return find_def(app, ref, group)[1]

def anchor_for(app, ref, txt=None, *, group=None,  **opts):
  target = find_def(app, ref, group)
  return f"<a href=\"{href_for(app, ref, **opts)}\">{txt or target[0]}</a>"

def href_for(app, ref, *, group=None, **opts):
  return hrefify(find_def(app, ref, group)[1], **opts)

def hrefify(ref, **opts):
  '''
    Converts a reference, given a string, into an anchor

    >>> to_anchor('guides/documenting/your_sphinx_app:Origen's Sphinx App')
        guides/documenting/your_sphinx_app.html#origen-s-sphinx-app
  '''
  tmp = ref.split(':', 1)
  tmp[1] = re.sub(r'\W', '-', tmp[1])

  # Kind of a hack but seems to work
  return ('../' * ref.count('/')) + '.html#'.join(tmp)

def insert_cmd_output(app, cmd, **opts):
  # Run the command and gather the output
  out = subprocess.run(cmd, capture_output=True)
  out = out.stdout.decode('utf-8').strip()

  # Embed the output in a code block
  # Need to also shift the spacing of the output so its all under the code block
  # Also allow for the caller to place some prepend some additional spacing, in case this is used
  #   inside another block
  spacing = " " * opts['prepend_spaces'] if 'prepend_spaces' in opts else ""
  retn = [f"{spacing}.. code:: none", ""]
  retn += [f"{spacing}  {l}" for l in out.split("\n")]
  return "\n".join(retn)

@origen.helpers.continue_on_exception(ose.logger)
def process_docstring(app, what, name, obj, options, lines):
  ''' Runs the template engine on docstrings, allowing for jinja syntax inside docstrings. '''
  try:
    _lines = jinja_render_string(app, "\n".join(lines))
  except Exception as e:
    # Technically, not all exceptions have a message, so get for the attribute first
    m = getattr(e, 'message', repr(e))
    raise type(e)(f"Exception occurred processing the docstring for {name} of doc-type {what}{': ' + m if m else ''}") from e
  _lines += "\n"
  lines.clear()
  lines += _lines.split("\n")
