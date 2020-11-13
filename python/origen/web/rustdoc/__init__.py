'''
Simple |sphinx_ext| for building a |Rust| project's documentation, via |cargo_doc|, and moving it
into the |sphinx_app|.

Rust projects are added through the ``rustdoc_projects`` |sphinx_config_var| - a |dict| whose
keys are the project names and whose values are a second |dict| containing that project's configuration:

.. code:: python

  # conf.py
  rustdoc_projects = {
    'project1': {
      'opt a': 'value',
      'opt b': 'value',
      # ...
    },
    'project2': {
      'opt a': 'value',
      'opt b': 'value',
      # ...
    }
    # ...
  }

The following configuration options are available per-project:

* ``source`` - **Required** - The Rust project's source location.
* ``default_build_options`` (``True``) - Adds default build options ``no-deps`` and ``workspace`` to the ``cargo doc`` command.
* ``rustdoc_output_dir`` (``./``) - Directory to move the resulting documentation to. Defaults to the current directory.
* ``apply_svg_workarounds`` (``False``) - Applies a fix for SVG images needed if releasing to ``github.io``. See :meth:`here <origen.web.rustdoc.RustDocProject.fix_svg>` for more details.
* ``build_options`` (``{}``) - Additional key-value pairs passed as arguments to ``cargo doc``.

These |sphinx_conf_vars| are also available:

* ``rustdoc_apply_svg_workarounds`` (``None``) - Applies SVG workaround to all ``Rustdoc Projects``, unless overridden by the project's config.
* ``rustdoc_output_dir`` (``None``) - Applies this output directory to all ``Rustdoc Projects`` unless overridden by the project's config.

See Also
--------

  * Example setup in the |src_code:core_conf|

'''

import subprocess, pathlib, copy, shutil, os
from sphinx.errors import ExtensionError
from sphinx.util.logging import getLogger

logger = getLogger('RustDoc')


class RustDocProject:
    BRUSH_SVG_SRC = 'brush.svg'
    BRUSH_PNG_SRC = 'brush.png'
    WHEEL_SVG_SRC = 'wheel.svg'
    WHEEL_PNG_SRC = 'wheel.png'
    DOWN_ARROW_SVG_SRC = 'down-arrow.svg'
    DOWN_ARROW_PNG_SRC = 'down-arrow.png'
    OVERRIDE_PATH = pathlib.Path(
        os.path.abspath(__file__)).parent.joinpath('override')
    BRUSH_FILE = str(OVERRIDE_PATH.joinpath(BRUSH_PNG_SRC))
    WHEEL_FILE = str(OVERRIDE_PATH.joinpath(WHEEL_PNG_SRC))
    DOWN_ARROW_FILE = str(OVERRIDE_PATH.joinpath(DOWN_ARROW_PNG_SRC))

    def __init__(self, proj, config):
        config = copy.deepcopy(config)
        use_defaults = config.pop('default_build_options', True)
        self.proj = proj
        self.source = config.pop('source', None)
        self.output_dir = pathlib.Path(config.pop('rustdoc_output_dir',
                                                  './')).joinpath(proj)
        self.apply_svg_workarounds = config.pop('apply_svg_workarounds')
        if use_defaults:
            self.build_options = {
                **{
                    'no-deps': None,
                    'workspace': None,
                },
                **config.pop('build_options', {})
            }
        else:
            self.build_options = config.pop('build_options', {})

        # Check arguments
        if len(config) != 0:
            # Warn that extra config values were used
            logger.warning(
                f"Config for project {self.proj} has unused keys: {','.join(config.keys())}"
            )

        if self.source is None:
            # Fail here. Need to know where the project souurce is.
            raise ExtensionError(
                f"RustDoc Project {self.proj} must include a 'source'!")
        else:
            self.source = pathlib.Path(self.source)

        if not self.source.exists():
            # Ensure the given path exists
            raise ExtensionError(
                f"Could not find path {self.source} given by RustDoc project {self.proj}"
            )

    def cmd(self):
        ''' Returns the ``cargo doc`` command to build the project's documentation

        Returns:
          str: ``cargo doc`` command
    '''
        opt_str = ""
        for opt, val in self.build_options.items():
            if val:
                opt_str += f" --{opt} {val}"
            else:
                opt_str += f" --{opt}"
        return f"cargo doc {opt_str}"

    def build(self):
        ''' Runs the build :meth:`command <cmd>` and :meth:`moves the resulting docs <mv_docs>`
        into the Sphinx project space
    '''
        logger.debug(f"Running Rustdoc command: {self.cmd()}")
        subprocess.run(self.cmd(), cwd=self.source, shell=True)
        if self.output_dir:
            self.mv_docs()
        if self.apply_svg_workarounds:
            self.fix_svg()

    def mv_docs(self):
        '''
      Copy the resulting docs from the target/doc directory into the output directory.
      Note: these are copied since the ``--target-dir`` option will actually rebuild
      the project in the new directory, which isn't what we want. We just want
      the output docs.
    '''
        s = self.source.joinpath("target/doc")
        if self.output_dir:
            if not self.output_dir.exists():
                self.output_dir.mkdir(parents=True)
            else:
                shutil.rmtree(str(self.output_dir))
                self.output_dir.mkdir(parents=True)
            if s.exists:
                logger.info(f"Moving docs from {s} to {self.output_dir}...")
                shutil.move(str(s), str(self.output_dir))
            else:
                logger.error(f"Could not find resulting docs at {s}")
        else:
            logger.error(
                f"No output directory given. Cannot move resulting docs!")

    def fix_svg(self):
        '''
      At the time of this implementation, ``github.io`` pages seem to dislike rendering local svg files.
      It doesn't seem to have a problem with SVG in general, nor a problem with the SVGs Rust docs
      actually uses - just the way its referenced.

      A quick workaround for this is just to convert Rust's SVGs into PNGs and post-process the resulting
      html files to reference the PNGs instead. This will be done for all html files and the
      SVGs: ``brush.svg``, ``wheel.svg``, and ``down-arrow.svg``.

      SVGs were converted using :svg_to_png_converter:`this site <>`.
    '''
        shutil.copy2(str(self.BRUSH_FILE),
                     str(self.output_dir.joinpath('doc')))
        shutil.copy2(str(self.WHEEL_FILE),
                     str(self.output_dir.joinpath('doc')))
        shutil.copy2(str(self.DOWN_ARROW_FILE),
                     str(self.output_dir.joinpath('doc')))
        for html in list(self.output_dir.joinpath('doc').glob('**/*.html')):
            lines = open(html, "r", encoding='utf8').readlines()
            for i, _l in enumerate(lines):
                # Replace the .svg files with .png
                lines[i] = lines[i].replace(self.DOWN_ARROW_SVG_SRC,
                                            self.DOWN_ARROW_PNG_SRC)
                lines[i] = lines[i].replace(self.BRUSH_SVG_SRC,
                                            self.BRUSH_PNG_SRC)
                lines[i] = lines[i].replace(self.WHEEL_SVG_SRC,
                                            self.WHEEL_PNG_SRC)
            open(html, "w", encoding='utf8').writelines(lines)


def setup(sphinx):
    ''' Hook into sphinx just before it starts to read all the templates.
      Add this point, we'll build the Rust docs using 'cargo doc', parse the resulting contents,
      and create our own templates for Python-based calls.
  '''
    sphinx.connect("builder-inited", build)
    sphinx.add_config_value("rustdoc_projects", {}, '')
    sphinx.add_config_value("rustdoc_output_dir", None, '')
    sphinx.add_config_value("rustdoc_apply_svg_workarounds", None, '')


def build(sphinx):
    ''' Build each Rustdoc project
  '''
    for proj, config in sphinx.config.rustdoc_projects.items():
        logger.info(f"Building docs for project {proj}")
        if sphinx.config.rustdoc_output_dir:
            output_dir = sphinx.config.rustdoc_output_dir
        else:
            output_dir = pathlib.Path(sphinx.env.srcdir).joinpath(
                sphinx.config.html_static_path[0]).joinpath('rustdoc')
        if sphinx.config.rustdoc_apply_svg_workarounds:
            apply_svg_workarounds = sphinx.config.rustdoc_apply_svg_workarounds
        else:
            apply_svg_workarounds = False
        RustDocProject(
            proj, {
                **{
                    'rustdoc_output_dir': output_dir,
                    'apply_svg_workarounds': apply_svg_workarounds
                },
                **config
            }).build()
