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
  OVERRIDE_PATH = pathlib.Path(os.path.abspath(__file__)).parent.joinpath('override')
  BRUSH_FILE = str(OVERRIDE_PATH.joinpath(BRUSH_PNG_SRC))
  WHEEL_FILE = str(OVERRIDE_PATH.joinpath(WHEEL_PNG_SRC))
  DOWN_ARROW_FILE = str(OVERRIDE_PATH.joinpath(DOWN_ARROW_PNG_SRC))

  def __init__(self, proj, config):
    config = copy.deepcopy(config)
    use_defaults = config.pop('default_build_options', True)
    self.proj = proj
    self.source = config.pop('source', None)
    self.output_dir = pathlib.Path(config.pop('rustdoc_output_dir', './')).joinpath(proj)
    self.apply_svg_workarounds = config.pop('apply_svg_workarounds')
    if use_defaults:
      self.build_options = {**{
        'no-deps': None,
        'workspace': None,
      }, **config.pop('build_options', {})}
    else:
      self.build_options = config.pop('build_options', {})
    
    # Check arguments
    if len(config) != 0:
      # Warn that extra config values were used
      logger.warning(f"Config for project {self.proj} has unused keys: {','.join(config.keys())}")
    
    if self.source is None:
      # Fail here. Need to know where the project souurce is.
      raise ExtensionError(f"RustDoc Project {self.proj} must include a 'source'!")
    else:
      self.source = pathlib.Path(self.source)
    
    if not self.source.exists():
      # Ensure the given path exists
      raise ExtensionError(f"Could not find path {self.source} given by RustDoc project {self.proj}")
  
  def cmd(self):
    opt_str = ""
    for opt, val in self.build_options.items():
      if val:
        opt_str += f" --{opt} {val}"
      else:
        opt_str += f" --{opt}"
    return f"cargo doc {opt_str}"

  def build(self):
    print(self.cmd())
    subprocess.run(self.cmd(), cwd=self.source)
    if self.output_dir:
      self.mv_docs()
    if self.apply_svg_workarounds:
      self.fix_svg()
  
  def mv_docs(self):
    '''
      Copy the resulting docs from the target/doc directory and into the 
      output directory.
      Note: these are copied since the --target-dir option will actually rebuild
      the project in the new directory, which isn't what we want. We just want
      the output docs.
    '''
    s = self.source.joinpath("target/doc")
    if self.output_dir:
      if not self.output_dir.exists():
        # shutil will get mad if the directory doesn't exists.
        self.output_dir.mkdir(parents=True)
      else:
        # shutil will also get mad if the directory does exists.
        shutil.rmtree(str(self.output_dir))
        self.output_dir.mkdir(parents=True)
      if s.exists:
        logger.info(f"Moving docs from {s} to {self.output_dir}...")
        shutil.move(str(s), str(self.output_dir))
      else:
        logger.error(f"Could not find resulting docs at {s}")
    else:
      logger.error(f"No output directory given. Cannot move resulting docs!")

  def fix_svg(self):
    '''
      At the time of this implementation, github.io pages seem to not like rendering local svg files. It doesn't seem to have a problem
      with SVG in the general sense, nor a problem with the SVGs Rust docs actually use - just the way its referenced.

      A quick workaround for this was just to convert Rust's SVGs to PNGs and post-process the resulting html files to reference the PNGs instead.
      This will be done for all html files and the SVGs: 'brush.svg', 'wheel.svg', and 'down-arrow.svg'.

      SVGs were converted using this site: https://svgtopng.com/
    '''
    shutil.copy2(str(self.BRUSH_FILE), str(self.output_dir.joinpath('doc')))
    shutil.copy2(str(self.WHEEL_FILE), str(self.output_dir.joinpath('doc')))
    shutil.copy2(str(self.DOWN_ARROW_FILE), str(self.output_dir.joinpath('doc')))
    for html in list(self.output_dir.joinpath('doc').glob('**/*.html')):
      lines = open(html, "r", encoding='utf8').readlines()
      for i, _l in enumerate(lines):
        # Replace the .svg files with .png
        lines[i] = lines[i].replace(self.DOWN_ARROW_SVG_SRC, self.DOWN_ARROW_PNG_SRC)
        lines[i] = lines[i].replace(self.BRUSH_SVG_SRC, self.BRUSH_PNG_SRC)
        lines[i] = lines[i].replace(self.WHEEL_SVG_SRC, self.WHEEL_PNG_SRC)
      open(html, "w", encoding='utf8').writelines(lines)

def setup(sphinx):
  # Hook into the sphinx just before it starts to read all the templates.
  # Add this point, we'll build the Rust docs using 'cargo doc', parse the resulting contents,
  #   and create our own templates for Python-based calls.
  # https://www.sphinx-doc.org/en/master/extdev/appapi.html#event-env-before-read-docs
  sphinx.connect("builder-inited", build)
  sphinx.add_config_value("rustdoc_projects", {}, '')
  sphinx.add_config_value("rustdoc_output_dir", None, '')
  sphinx.add_config_value("rustdoc_apply_svg_workarounds", None, '')

def build(sphinx):
  for proj, config in sphinx.config.rustdoc_projects.items():
    logger.info(f"Building docs for project {proj}")
    if sphinx.config.rustdoc_output_dir:
      output_dir = sphinx.config.rustdoc_output_dir
    else:
      output_dir = pathlib.Path(sphinx.env.srcdir).joinpath(sphinx.config.html_static_path[0]).joinpath('rustdoc')
    if sphinx.config.rustdoc_apply_svg_workarounds:
      apply_svg_workarounds = sphinx.config.rustdoc_apply_svg_workarounds
    else:
      apply_svg_workarounds = False
    RustDocProject(proj, {**{'rustdoc_output_dir': output_dir, 'apply_svg_workarounds': apply_svg_workarounds}, **config}).build()