import origen, pathlib, re, mako, os, abc
from mako.template import Template
from jinja2 import Template as JinjaTemplate
from os import access, W_OK, X_OK, R_OK
from origen.errors import *


class UnknownSyntaxError(Exception):
    ''' Raised when an unknown syntax if given '''
    def __init__(self, syntax):
        self.message = f"Origen's compiler cannot does not know how to compiler syntax '{syntax}''"


class ExplicitSyntaxRequiredError(Exception):
    ''' Raised when an explicit syntax is required, but none was given '''
    def __init__(self, src, *, direct_src):
        if direct_src:
            if len(src) > 80:
                chars = src[0:80]
            else:
                chars = src
            self.message = f"Origen's Compiler requires an explicit syntax for direct source starting with {chars}"
        else:
            self.message = f"Origen's Compiler cannot discern syntax for file {src}"


class Renderer(abc.ABC):

    # Python 3.6's docstring doesn't like this. There's nothing wrong with it functionally,
    # but the docstring won't parse correctly. Overwrite the docstring in web/source/conf.py
    # to workaround this without changing the API.
    @property
    @abc.abstractclassmethod
    def file_extensions(cls):
        return []  #raise NotImplementedError

    @abc.abstractmethod
    def render_file(self, file):
        pass

    @abc.abstractmethod
    def render_str(self, file):
        pass

    def __init__(self, compiler, renderer_opts={}):
        self.compiler = compiler
        self.renderer_opts = renderer_opts
        #self.file_extensions = [f if f.startswith('.') else f".{f}" for f in self._file_extensions]

    @classmethod
    def resolve_filename(cls, src):
        s = pathlib.Path(src).parts[-1]
        for ext in cls.file_extensions:
            s = s.replace(ext if ext.startswith('.') else f".{ext}", '')
        return pathlib.Path(src).parent.joinpath(s)


class Compiler:
    def __init__(self, *args, **options):
        self.stack = list(args) if args else []
        self.renders, self.output_files = [], []
        self.renderers = {'mako': MakoRenderer, 'jinja': JinjaRenderer}

    @property
    def supported_extensions(self):
        exts = []
        for r in self.renderers.values():
            exts += [(ext if ext.startswith('.') else f".{ext}")
                     for ext in r.file_extensions]
        return exts

    @property
    def syntaxes(self):
        return self.renderers.keys()

    # Allow the stack to be incremented, enabling compile at a later time
    def push(self, *args, **options):
        if not args:
            raise TypeError(
                'No compiler arguments passed, cannot push them to the stack!')
        if 'direct_src' in options:
            self.stack.append((list(args), options))
        elif 'templates_dir' in options:
            t = options.get('templates_dir', None) or self.templates_dir
            self.stack.append(
                (list([self.templates_dir.joinpath(f)
                       for f in args]), options))
        else:
            self.stack.append((list([pathlib.Path(f) for f in args]), options))

    # Pop the first item of the stack
    def pop(self):
        if not self.stack:
            raise RuntimeError("Cannot pop compiler stack, nothing on it!")
        else:
            self.stack.pop()

    # Clear the stack
    def clear(self):
        self.stack, self.renders, self.output_files = [], [], []

    # Run the compiler with the stack as-is or with new args
    def run(self, *args, **options):
        if args:
            self.push(*args, **options)
        for (batch, opts) in self.stack:
            for job in batch:
                rendered = self.render(job, **opts)
                self.renders.append(rendered)
                if isinstance(rendered, pathlib.Path):
                    self.output_files.append(rendered)
                else:
                    self.output_files.append(None)
        self.stack.clear()

    @property
    def last_render(self):
        return self.renders[-1] if self.renders else None

    @property
    def templates_dir(self):
        templates_dir = pathlib.Path(f"{origen.app.root}/templates")
        if not templates_dir.exists():
            raise FileNotFoundError(
                f"Application templates directory does not exist at {templates_dir}"
            )
        elif not templates_dir.is_dir():
            raise NotADirectoryError(
                f"Application templates directory exists at {templates_dir} but it is not a real directory!"
            )
        elif not access(templates_dir, W_OK):
            raise PermissionError(
                f"Application templates directory exists at {templates_dir} but is not writeable!"
            )
        else:
            return templates_dir

    def select_syntax(self, filename, syntax=None):
        f = pathlib.Path(filename)
        if f.suffix == '.mako':
            return 'mako'
        elif f.suffix == '.jinja' or f.suffix == ".j2" or f.suffix == ".jinja2":
            return 'jinja'

    def render(self,
               src,
               *,
               direct_src=False,
               syntax=None,
               context={},
               use_standard_context=True,
               renderer_opts={},
               output_dir=None,
               output_name=None,
               file_to_string=False,
               **options):
        ''' Direct access to compling templates. '''
        r = self.renderer_for(src, direct_src=direct_src,
                              syntax=syntax)(self, renderer_opts)
        c = context.copy()
        if use_standard_context:
            c = {**c, **origen.standard_context()}
        if direct_src:
            return r.render_str(src, c)
        else:
            origen.logger.info(f"Compiling template {src}")
            if file_to_string:
                return r.render_str(open(src, r).readlines(), c)
            else:
                rendered = r.render_file(
                    src,
                    self.resolve_filename(src,
                                          output_dir=output_dir,
                                          output_name=output_name,
                                          renderer=r), c)
                origen.logger.info(f"Compiler output created at {rendered}")
                return rendered

    def renderer_for(self, src, direct_src=False, syntax=None):
        # If given a file, try to discern the renderer, unless a syntax was given
        s = syntax or (self.select_syntax(src) if not direct_src else None)

        # Either wasn't given a syntax or couldn't discern based on the filename.
        # Raise an error
        if not s:
            if direct_src:
                # Need to specify the syntax if a direct source was given.
                raise ExplicitSyntaxRequiredError(src, direct_src=True)
            else:
                # Given a file, but it didn't match any available renderers
                raise ExplicitSyntaxRequiredError(src, direct_src=False)
        if s in self.renderers:
            return self.renderers[s]
        else:
            # Given an unrecognized error. Complain.
            raise UnknownSyntaxError(syntax)

    def resolve_filename(self,
                         src,
                         *,
                         output_dir=None,
                         output_name=None,
                         renderer=None):
        '''
            Resolves the output name from the source, and given options.

            Resolution rules:

            * if no output_dir or output_name is given, the resolved filename will end up in
                ``<app_output_dir>/renders/<file.x>`` where the syntax-specific extension has been removed
                from the filename
            * if an output name is given, the name and extension is changed to the output name, but the
                output_dir remains as above.
            * if the output_dir is given, the templates are written to that location. Same rules for
                output_name apply
        '''
        src = pathlib.Path(src)
        r = renderer or self.renderer_for(src)
        output = pathlib.Path(output_dir
                              or origen.app.output_dir.joinpath(f'compiled'))
        output = output.joinpath(output_name or r.resolve_filename(src.name))
        return output

    def _write_output_file(self, contents, output_file):
        out = pathlib.Path(output_file)
        pathlib.Path.mkdir(out.parent, parents=True, exist_ok=True)
        with open(out, 'w') as f:
            f.write(contents)
        # TODO: Figure out why this doesn't work
        out.chmod(0o755)
        return out

    def __check_template(self, t):
        if not t.exists():
            raise FileNotFoundError(f"Template file does not exist at {t}")
        elif not access(t, R_OK):
            raise PermissionError(
                f"Template file exists at {t} but is not readable!")


class MakoRenderer(Renderer):
    class MakoSyntax:
        var_sub = re.compile(r'\$\{.*\}')
        ctrl_struct = re.compile(r'^\s*\%.*\%')
        module_block = re.compile(r'\<\%\!.*\%\>')
        tag = re.compile(r'\<\%.*\>')
        expresions = [var_sub, ctrl_struct, module_block, tag]

        def inspect(self, arg):
            for regex in self.expresions:
                if regex.search(arg):
                    return True
            return False

    # Use a class variable as the syntax should be viewed
    # as immutable by Compiler instances
    syntax = MakoSyntax()
    file_extensions = ['mako']
    preprocessor = [lambda x: x.replace("\r\n", "\n")]
    '''
        Preprocessor to remove double newlines from Windows sources

        See Also
        --------
        * :ticket_mako_multiple_newlines:`Relevant Stack-Overflow Question <>`
    '''

    def render_str(self, src, context):
        return Template(src, preprocessor=self.preprocessor).render(**context)

    def render_file(self, src, output_file, context):
        print(src)
        return self.compiler._write_output_file(
            Template(filename=str(src),
                     preprocessor=self.preprocessor).render(**context),
            output_file)


class JinjaRenderer(Renderer):
    file_extensions = ['jinja', 'jinja2', 'j2']
    preprocessor = []

    def render_str(self, src, context):
        return JinjaTemplate(src).render(**context)

    def render_file(self, src, output_file, context):
        #import pdb; pdb.set_trace()
        print(src)
        return self.compiler._write_output_file(
            JinjaTemplate(open(src).read()).render(**context), output_file)
