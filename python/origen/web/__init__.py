import _origen #pylint:disable=import-error
import origen #pylint:disable=import-error
import subprocess, shutil

''' Relative to the application's website output directory '''
OUTPUT_INDEX_FILE = 'index.html'

''' Relative to the SPHINX_ROOT_OFFSET '''
SPHINX_CONFIG = 'config.py'

SPHINX_OUTPUT = 'sphinx_build'

SPHINX_STATIC = '_static'

SPHINX_TEMPLATE = '_templates'

source_dir = origen.app.website_source_dir
static_dir = source_dir.joinpath(SPHINX_STATIC).joinpath('build')
templates_dir = source_dir.joinpath(SPHINX_TEMPLATE).joinpath('origen')
interbuild_dir = source_dir.joinpath('interbuild')
output_build_dir = origen.app.website_output_dir.joinpath(SPHINX_OUTPUT)
output_index_file = output_build_dir.joinpath(OUTPUT_INDEX_FILE)

def run_cmd(subcommand, args):
  if subcommand == "build":
    if "clean" in args:
      run_cmd("clean", args)

    for d in [static_dir, templates_dir, output_build_dir, interbuild_dir]:
      if not d.exists():
        d.mkdir(parents=True)
    _origen.logger.log("Running web:buld command...")
    _origen.logger.log(f"\t{sphinx_cmd(args)}")
    run_sphinx(args)

    if "view" in args:
      run_cmd("view", args)
  elif subcommand == "clean":
      if origen.app.website_output_dir.exists:
        _origen.logger.log(f"Removing built website at {str(origen.app.website_output_dir)}")
        shutil.rmtree(origen.app.website_output_dir, ignore_errors=True)
      else:
         _origen.logger.log("No built website to clean!")
  elif subcommand == "view":
    if site_built():
      _origen.logger.log(f"Launching web browser with command: \"{view_cmd()}\"")
      subprocess.run(view_cmd())
    else:
      _origen.logger.error(f"Could not find built website at {output_build_dir}. Please run 'origen web build --view' to build the site and view the results.")
      exit()
  else:
    origen.logger.error(f"Unrecongized web command: {subcommand}")
    exit()

def view_cmd():
  if origen.running_on_windows:
    # Lots of quotes to account for potential spaces in the path.
    # https://superuser.com/questions/239565/can-i-use-the-start-command-with-spaces-in-the-path
    return f"cmd /C start \"\" \"{output_index_file}\""
  else:
    return f"xdg-open \"{output_index_file}\""

def site_built():
  '''
    Returns true if some static site pages are found in the applications web output directory. False otherwise.
    The phrase 'some static sites pages are found' is defined to mean <website_output_dir>/build/index.html exists.
  '''
  return output_index_file.exists()

def run_sphinx(args):
  '''
    Launches the Sphinx-build command with the necessary options and monitors the output.
    If the build is successful, returns the output path. Otherwise, returns the output.
  '''
  out = subprocess.run(sphinx_cmd(args))
  return out

def sphinx_cmd(args):
  build_opts = ""
  if 'no-api' in args:
    # no-api is achieved by overriding the autoapi, autodoc, and rustdoc configs to
    # all be empty
    build_opts += "-D origen_no_api=True"
  return f"poetry run sphinx-build {origen.app.website_source_dir} {output_build_dir} {build_opts}"

def sphinx_make():
  return f"{origen.app.website_source_dir}/../make{'.bat' if origen.running_on_windows else ''}"
