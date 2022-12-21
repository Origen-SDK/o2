'''
Wrappers, helpers, and utilities for Origen's :link-to:`documentation system <documenting:introduction>`

Web commands invoked from the |web_cmd| will end up at :meth:run_cmd, which wraps the |sphinx_build_cmd| or
whatever else the command is. :func:`run_cmd` can also be invoked directly to kick of
website compilations, etc. from scripts.

Functions in this module are, themselves, not tied to a Sphinx runtime but instead defer
all *sphinx-interfacing* to the |ose|.

Assuming |origen-s_sphinx_app| is used, the settings here will be loaded during
:func:`origen_sphinx_extension.setup` and applied during :func:`origen_sphinx_extension.apply_origen_config`
'''

import _origen  #pylint:disable=import-error
import origen, origen.helpers  #pylint:disable=import-error
import subprocess, shutil
from typing import List
from types import ModuleType

ORIGEN_CORE_HOMEPAGE = 'https://origen-sdk.org/o2'
''' Hard-coded path to the Origen core's homepage - usable by applications to link to'''

OUTPUT_INDEX_FILE = 'index.html'
''' Sphinx's ``index.html`` file (the assumed homepage) '''

SPHINX_CONFIG = 'conf.py'
''' Sphinx's ``conf.py`` filename '''

SPHINX_OUTPUT = 'sphinx_build'
''' Sphinx's build command name '''

SPHINX_STATIC = '_static'
'''
  Default location of static content, assumed to be relative to the
  :data:source_dir
'''

SPHINX_TEMPLATE = '_templates'
'''
  Default location of explicit templates, assumed to be relative to the
  :data:source_dir
'''

RELEASE_ARGS = ['-W', '--keep-going', '-D origen_releasing_build=1']
'''
  Arguments passed to the |sphinx_app| when releasing webpages. Going for only allowing clean builds to be released.
  The option ``--release-with-warnings`` can be used to release the build regardless.
'''

source_dir = origen.app.website_source_dir
''' Resolved source directory in which the |sphinx_app| lives '''

static_dir = source_dir.joinpath(SPHINX_STATIC)
'''
  Resolved/default |sphinx_static_dir|

  This path will be automatically added in the |ose|, as other plugins which operate outside
  of Sphinx but generate *web content* may rely on this path begin part of the Sphinx project.

  Note
  ----

    * Although Sphinx can contain multiple :link-to:`static directories <sphinx_static_dir>`,
      this only points to a single one - which Origen will use.
'''

unmanaged_static_dir = static_dir.joinpath('build')
'''
  Resolved/default |sphinx_static_dir| which is automically unmanaged by revision control (e.g., |.gitignore|). Can be used
  to store *"static content that is dynamically generated"* or, put another way, content that is dynamically generated but
  due to Sphinx's build flow/assumptions or ease-of-use, needs to be placed in a 'static' location.

  This path will be automatically added in the |ose|, as other plugins which operate outside
  of Sphinx but generate *web content* may rely on this path begin part of the Sphinx project.
'''

templates_dir = source_dir.joinpath(SPHINX_TEMPLATE)
'''
  Resolved/default |sphinx_templates_dir|

  This path will be automatically added in the |ose|, as other plugins which operate outside
  of Sphinx but generate *web content* may rely on this path begin part of the Sphinx project.

  Note
  ----

    * Although Sphinx can contain multiple :link-to:`template directories <sphinx_templates_dir>`,
      this only points to a single one - which Origen will use.
'''

interbuild_dir = source_dir.joinpath('interbuild')
'''
  Points to |origen-s_sphinx_app| ``interbuild`` directory.

  This directory houses dynamic content generated from Sphinx (such as AutoAPI) which doesn't
  require check-in (is ignored by the |.gitignore| by default) but is part of the Sphinx project.

  Assuming a full-rebuild (no ``--no-api`` or similar options), this directory will always be rebuilt
  and can safely be deleted between runs.
'''

output_build_dir = origen.app.website_output_dir.joinpath(SPHINX_OUTPUT)
''' Resolved location for the final output '''

output_index_file = output_build_dir.joinpath(OUTPUT_INDEX_FILE)
''' Resolved output index file path '''

sphinx_config = origen.app.website_source_dir.joinpath(SPHINX_CONFIG)
''' Resolved source directory in which the |sphinx_app| lives '''


def run_cmd(subcommand, args):
    '''
    Entry point for the ``web`` command. The subcommand and any arguments will be processed here then handed off
    to the proper functions for execution.

    Provided this function is kept in sync with |web_cmd|, everything else should fall into place (or give necessary errors instead
    of just doing nothing).
  '''
    if subcommand == "build":
        if "clean" in args:
            run_cmd("clean", args)

        for d in [
                static_dir, unmanaged_static_dir, templates_dir,
                output_build_dir, interbuild_dir
        ]:
            if not d.exists():
                d.mkdir(parents=True)
        origen.logger.info("Running web:build command...")
        origen.logger.info(f"\t{sphinx_cmd(args)}")
        if run_sphinx(args).returncode:
            origen.logger.error("Failed to build the webpages! Exiting...")
            exit()

        if "release" in args:
            release(archive_id=args.get('archive', None))
        elif "archive" in args and "release" not in args:
            release(archive_id=args["archive"], archive_only=True)

        if "view" in args:
            run_cmd("view", args)
    elif subcommand == "clean":
        # Run 'clean' on any extension which supports it.
        clean(args)
    elif subcommand == "view":
        if site_built():
            origen.logger.info(
                f"Launching web browser with command: \"{view_cmd()}\"")
            subprocess.run(view_cmd(), shell=True)
        else:
            origen.logger.error(
                f"Could not find built website at {output_build_dir}. Please run 'origen web build --view' to build the site and view the results."
            )
            exit()
    else:
        origen.logger.error(f"Unrecognized web command: {subcommand}")
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
    out = subprocess.run(sphinx_cmd(args), shell=True)
    return out


def sphinx_cmd(args):
    '''
    Given that we're running ``web:build``, processes the arguments and returns a command executing *sphinx build* with
    the proper context.
  '''
    build_opts = []
    if 'no-api' in args:
        # no-api is achieved by overriding the autoapi, autodoc, and rustdoc configs to
        # all be empty
        build_opts.append("-D origen_no_api=True")
    if 'release' in args or 'as-release' in args:
        if 'release-with-warnings' in args:
            build_opts.extend(RELEASE_ARGS[2:])
        else:
            build_opts.extend(RELEASE_ARGS)
    if 'sphinx-args' in args:
        # Add an user arguments
        build_opts.append(args['sphinx-args'])
    return f"poetry run sphinx-build {origen.app.website_source_dir} {output_build_dir} {' '.join(build_opts)}"


def sphinx_make():
    '''
    Returns the path to the makefile created from ``sphinx quickstart``
  '''
    return f"{origen.app.website_source_dir}/../make{'.bat' if origen.running_on_windows else ''}"


def sphinx_extensions() -> [str]:
    '''
    Returns a list of :sphinx_extensions:`Sphinx extensions <>` currently in ``conf.py`` as strings.

    Notes
    -----

    * This does not actually run ``Sphinx``, so this is based on introspection only. Extensions which dynamically add other
      extensions will not be discovered here.
    
    See Also
    --------

    * :func:`sphinx_extension_mods`
  '''
    conf = origen.helpers.mod_from_file(str(sphinx_config))
    return conf.extensions


def sphinx_extension_mods() -> List[ModuleType]:
    '''
    Returns a list of :sphinx_extensions:`Sphinx extensions <>` currently in ``conf.py`` as the actual modules.

    Notes
    -----

    * This does not actually run ``sphinx``, so this is based on introspection only. Extensions which dynamically add other
      extensions will not be discovered here.
    
    See Also
    --------

    * :func:`sphinx_extensions`
  '''
    def imp(ext):
        exec(f"import {ext}")
        return eval(ext)

    return [imp(ext) for ext in sphinx_extensions()]


def get_sphinx_config_out_of_app():
    '''
    Uses introspection/metaprogramming principles to discern Sphinx's ``conf.py``
    content without actually running Sphinx.
    **This will not pick up content which is added dynamically during the build phases.
    This is only to get the user's ``conf`` contents**.
  '''
    return origen.helpers.mod_from_file(str(sphinx_config))


def clean(args=None):
    '''
    Runs ``clean`` on any extension which supports it.

    *Supporting clean* just means that the extension responds to a ``clean`` method.
  '''
    config = get_sphinx_config_out_of_app()

    # Remove any existing output
    if origen.app.website_output_dir.exists:
        origen.logger.info(
            f"Removing built website at {str(origen.app.website_output_dir)}")
        shutil.rmtree(origen.app.website_output_dir, ignore_errors=True)
    else:
        origen.logger.info("No built website to clean!")

    if interbuild_dir.exists:
        shutil.rmtree(interbuild_dir, ignore_errors=True)

    # Run any extension which has a 'clean' method
    for ext in sphinx_extension_mods():
        if origen.helpers.has_method(ext, "clean"):
            origen.logger.info(f"Cleaning extension {ext.__name__}")
            ext.clean(config)


def release(src=None,
            name=None,
            location=None,
            archive_id=None,
            archive_offset='archive',
            archive_only=False):
    '''
    General purpose release script that should cover basic cases.

    In the general sense, *releasing* the webpages amounts to just moving the contents somewhere and optionally
    performing some RC check-in function.

    Will leverage much of the RC driver for this so that the flow is just:

    1. Populate the repo
    2. Move the contents into the repo
    3. Check the repo back in

    If the release-location is just a path, then steps 1 & 2 can be skipped.

    The offset within either the path or repo will be the website_release_name with the 'archive/{archive-ID}' prefixed, if applicable.

    .. code-block:: python
    
      # With release location = 'path/to/release/to and offset = 'o2' and no archive indicated
      release_dir => path/to/release/to/o2

      # With the above and an archive ID = 'dev1'
      release_dir => path/to/release/to/archive/o2/dev1

      # With the above and a given archive offset = 'my/archives/'
      release_dir => path/to/release/to/my/archives/o2/dev1
    
  '''
    if archive_id is True:
        raise NotImplementedError(
            "Archive ID from app version is not supported yet!")
    _name = name or origen.app.website_release_name or origen.app.name
    _loc = location or origen.app.website_release_location
    _src = src or output_build_dir

    if _loc.path:

        def _release(dest):
            # Remove any existing contents
            shutil.rmtree(str(dest))
            shutil.copytree(str(_src), str(dest))

        dest = _loc.path
        if not archive_only:
            origen.logger.display(
                f"Releasing built website to '{str(_loc.path)}' as '{_name}'")
            dest = _loc.path.joinpath(_name)
            origen.logger.info(f"Releasing to path {dest}")
            dest.mkdir(parents=True, exist_ok=True)
            _release(dest)

        if archive_id:
            dest = _loc.path.joinpath(archive_offset).joinpath(_name).joinpath(
                archive_id)
            dest.mkdir(parents=True, exist_ok=True)
            origen.logger.display(f"Archiving built website to '{str(dest)}'")
            _release(dest)

    else:
        raise NotImplementedError(
            "Releasing via revision control has not been implemented yet!")
    origen.logger.display(f"Successfully released website for {_name}")
