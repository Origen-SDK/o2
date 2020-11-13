import origen, subprocess, builtins, types, inspect, re, pathlib
from .. import origen_sphinx_extension as ose


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
        source[0] = origen.app.compiler.render(source[0],
                                               syntax=syntax,
                                               direct_src=True,
                                               context=jinja_context(app))
    else:
        jinja_integrator(app, docname, source)


def insert_header(app, docname, source):
    '''
    Insert content at the beginning of the docs.

    Currently inserts:
    
      * Any |shorthand| ``include`` RST files
  '''
    ext = ose.sphinx_ext(app, 'origen.web.shorthand')
    doc = pathlib.Path(app.env.doc2path(docname))
    if '.rst' in doc.suffixes:
        if ext:
            includes = ext.all_include_rsts()
            # Make sure we aren't including the shared file in the shared files themselves
            if not any(i.match(str(doc)) for i in includes):
                depth = len(doc.relative_to(origen.web.source_dir).parents) - 1
                incs = [
                    "../" * depth + str(i.relative_to(origen.web.source_dir))
                    for i in includes
                ]
                source[0] = "\n".join([
                    f".. include:: {i}\n  :start-after: start-content\n\n"
                    for i in incs
                ]) + source[0]
                return True
    return False


# Setup taken from here: https://www.ericholscher.com/blog/2016/jul/25/integrating-jinja-rst-sphinx/
@origen.helpers.continue_on_exception(ose.logger)
def jinja_integrator(app, docname, source):
    src = source[0]
    try:
        rendered = app.builder.templates.render_string(src, jinja_context(app))
        source[0] = rendered
    except Exception as e:
        m = getattr(e, 'message', repr(e))
        raise type(e)(
            f"Exception occurred processing {docname}{': ' + m if m else ''}"
        ) from e


def jinja_render_string(app, src, additional_context={}):
    return app.builder.templates.render_string(src, jinja_context(app))


def jinja_context(app):
    shorthand_ext = ose.sphinx_ext(app, 'origen.web.shorthand')
    if shorthand_ext:
        shorthand_context = shorthand_ext.templating_context(app)
    else:
        shorthand_context = {}
    return {
        **builtins.__dict__,
        **types.__dict__,
        **inspect.__dict__,
        **{
            'origen':
            origen,
            'origen_sphinx_extension':
            ose,
            'origen_sphinx_app':
            app,

            # The remaining are non-namespaced helpers which may conflict with other context shifted in
            # from the user's conf.html_context
            # (This is why 'app' and 'ose' are listed twice, both as a namespaced and non-namespace option)
            'app':
            app,
            'ose':
            ose,
            'insert_cmd_output':
            lambda cmd, **opts: insert_cmd_output(app, cmd, **opts),
        },

        # Load in the shorthand helpers if the extension is loaded
        **shorthand_context,

        # Load in the user's html_context last, overwriting anything we've already added
        **app.config.html_context,
    }


def insert_cmd_output(app, cmd, *, shell=True, **opts):
    # Run the command and gather the output
    out = subprocess.run(cmd,
                         shell=shell,
                         stderr=subprocess.PIPE,
                         stdout=subprocess.PIPE)
    stdout = out.stdout.decode('utf-8').strip()
    if out.returncode == 1:
        ose.logger.warning(
            f"Failed to insert command \"{cmd}\". Command failed to run:")
        ose.logger.warning(f"STDOUT: {stdout}")
        ose.logger.warning(f"STDERR: {out.stderr.decode('utf-8').strip()}")

    # Embed the output in a code block
    # Need to also shift the spacing of the output so its all under the code block
    # Also allow for the caller to place some prepend some additional spacing, in case this is used
    #   inside another block
    spacing = " " * opts['prepend_spaces'] if 'prepend_spaces' in opts else ""
    retn = [f"{spacing}.. code:: none", ""]
    retn += [f"{spacing}  {l}" for l in stdout.split("\n")]
    return "\n".join(retn)


@origen.helpers.continue_on_exception(ose.logger)
def process_docstring(app, what, name, obj, options, lines):
    ''' Runs the template engine on docstrings, allowing for jinja syntax inside docstrings. '''
    app.emit("origen-preprocess-docstring", what, name, obj, options, lines)
    try:
        _lines = jinja_render_string(app, "\n".join(lines))
    except Exception as e:
        # Technically, not all exceptions have a message, so get for the attribute first
        m = getattr(e, 'message', repr(e))
        raise type(
            e
        )(f"Exception occurred processing the docstring for {name} of doc-type '{what}' (from {app.env.docname}) {': ' + m if m else ''}"
          ) from e
    _lines += "\n"
    lines.clear()
    lines += _lines.split("\n")
