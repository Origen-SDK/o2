import origen
import _origen
import importlib, inspect
import os.path
import re
from origen.controller import TopLevel
from origen.translator import Translator
from origen.compiler import Compiler
from origen.errors import *
from origen.callbacks import _callbacks
from types import ModuleType
from pathlib import Path
import origen_metal


class Base(_origen.application.PyApplication):
    '''
        The base class of all Origen ``applications``.
    '''
    @property
    def name(self):
        ''' Returns the unique ID (name) of the app/plugin '''
        return self._name

    @property
    def output_dir(self):
        ''' Returns the directory in which generated content should be placed '''
        return _origen.output_directory()

    @property
    def website_output_dir(self):
        ''' Returns the output directory offset from :meth:`output_dir` in which
            generated web content should be placed '''
        return _origen.website_output_directory()

    @property
    def website_source_dir(self):
        ''' Returns the source directory for |origen-s_sphinx_app| '''
        return _origen.website_source_directory()

    @property
    def website_release_location(self):
        ''' Returns the release location (URL, system-path, etc.) which the resulting
            website should be placed upon using the ``--release`` option of |web_cmd|'''
        return _origen.app_config()['website_release_location']

    @property
    def website_release_name(self):
        ''' Returns the name under which to release the website '''
        return _origen.app_config()['website_release_name']

    @property
    def root(self):
        ''' Returns the application's root directory '''
        return self._root

    @property
    def translator(self):
        ''' Returns the application's instance of :class:`Origen's Translator <origen.translator.Translator>` '''
        return self._translator

    @property
    def compiler(self):
        ''' Returns the application's instance of :class:`Origen's Compiler <origen.compiler.Compiler>` '''
        return self._compiler

    @property
    def plugin(self):
        ''' Will be set to true if the app instance it not the top-level app and is therefore operating as a plugin '''
        return self._plugin

    @property
    def app_dir(self):
        ''' Returns a path to the application's main Python dir, that is <app.root>/<app.name>'''
        return self._app_dir

    @property
    def python_dir(self):
        ''' An alias for app_dir '''
        return self._app_dir

    # TEST_NEEDED app.config_dir
    @property
    def config_dir(self):
        return self.root.joinpath("config")

    # TEST_NEEDED app.commands_dir
    @property
    def commands_dir(self):
        d = self.app_dir.joinpath("commands")
        if d.exists():
            return d
        else:
            return None

    @property
    def session(self):
        ''' Return this app's session store'''
        return origen.sessions.app_session(self)

    @property
    def user_session(self):
        ''' Return this app's user session store'''
        return origen.sessions.user_session(self)

    @property
    def rc(self):
        return origen_metal.frontend.frontend().revision_control

    @property
    def linter(self):
        return self._linter

    @property
    def publisher(self):
        return self._publisher

    @property
    def unit_tester(self):
        return self._unit_tester

    @property
    def release_scribe(self):
        return self._release_scribe

    @property
    def mailer(self):
        return origen.mailer

    @property
    def is_plugin(self):
        return self._plugin

    def __init__(self, *args, **options):
        self._compiler = Compiler()
        self._translator = Translator()
        if (origen.app is None) and origen.is_app_present:
            self._plugin = False
            self._root = origen.root
            self._name = _origen.app_config()["name"]
            origen_metal.frontend.frontend(
            ).rc = _origen.utility.revision_control.app_rc()
            self._unit_tester = _origen.utility.unit_testers.app_unit_tester()
            #self._linter = _origen.utility.linter.app_linter()
            self._publisher = _origen.utility.publisher.app_publisher()
            self._release_scribe = _origen.utility.release_scribe.app_release_scribe(
            )
        else:
            self._plugin = True
            self._root = options["root"]
            self._name = options["name"]
            origen_metal.frontend.frontend().rc = None
            self._unit_tester = None
            self._linter = None
            self._publisher = None
        self._app_dir = self.root.joinpath(self.name)
        self._block_path_cache = {}

    def block_path_to_dir(self, path, force_search=False):
        ''' Translates something like "dut.falcon" to <root>/<app>/blocks/dut/derivatives/falcon
        
            A tuple is returned where the first item is True/False indicating whether a block dir was found
            and if so then the second item is valid and contains the path to it.
            
            Note that this method caches the results and will always return the same result for the same path
            by default regardless of whether the block directories have been changed on disk.
            To force a re-evaluation of the given path pass 'True' as a second argument.
         '''
        if not force_search and path in self._block_path_cache:
            return self._block_path_cache[path]

        fields = path.split(".")
        filepath = self.root.joinpath(self.name).joinpath('blocks')
        for i, field in enumerate(fields):
            if i > 0:
                if filepath.joinpath('derivatives').joinpath(field).exists():
                    filepath = filepath.joinpath('derivatives').joinpath(field)
                elif filepath.joinpath('blocks').joinpath(field).exists():
                    filepath = filepath.joinpath('blocks').joinpath(field)
                elif filepath.joinpath(field).exists():
                    filepath = filepath.joinpath(field)
                elif filepath.joinpath(f"{field}.py").exists():
                    break
                else:
                    self._block_path_cache[path] = (False, None)
                    break
            elif filepath.joinpath(f"{field}.py").exists():
                break
            else:
                filepath = filepath.joinpath(field)
                if not filepath.exists():
                    self._block_path_cache[path] = (False, None)
                    break

        if path not in self._block_path_cache:
            self._block_path_cache[path] = (True, filepath)

        return self._block_path_cache[path]

    def instantiate_dut(self, path, **kwargs):
        ''' Instantiate the given DUT and return it, this must be called first before any
            sub-blocks can be instantiated '''
        if origen.dut is not None:
            raise RuntimeError(
                "Only one DUT target can be loaded, your current target selection instantiates multiple DUTs"
            )
        if origen._target_loading is not True:
            raise RuntimeError(
                "A DUT can only be instantiated within a target load sequence")
        origen.__instantiate_dut_called = True
        dut = self.instantiate_block(path)
        if not isinstance(dut, TopLevel):
            raise RuntimeError(
                "The DUT object is not an instance of origen.application::TopLevel"
            )
        origen.dut = dut
        origen.callbacks.emit("toplevel__initialized", kwargs=kwargs)
        return dut

    def instantiate_block_from_mod(self,
                                   mod,
                                   class_name="Controller",
                                   sb_options={}):
        if not self.__instantiate_dut_called:
            raise RuntimeError(
                f"No DUT has been instantiated yet, did you mean to call 'origen.instantiate_dut(\"{mod}\")' instead?"
            )

        # Load the module and try to find its controller.
        if isinstance(mod, ModuleType):
            m = mod
        else:
            m = importlib.import_module(mod + ".controller")
        c = getattr(m, class_name)
        if 'kwargs' in inspect.signature(c).parameters:
            block = c(**sb_options)
        else:
            block = c()
        block.app = self
        block.block_path = mod
        block.from_mod_path = True
        return block

    def instantiate_block(self,
                          path,
                          base_path=None,
                          *,
                          class_name="Controller",
                          sb_options=None):
        '''
            Instantiate the given block and return it
        
            >>> origen.app.instantiate_block("dut.falcon")
            >>> origen.app.instantiate_block("nvm.flash.f2mb")
        '''
        if not origen.__instantiate_dut_called:
            raise RuntimeError(
                f"No DUT has been instantiated yet, did you mean to call 'origen.instantiate_dut(\"{path}\")' instead?"
            )

        orig_path = path
        block_dir = None

        # The block path reference will be evaluated in the following order:
        # * A reference to a sub-block of the current block (if a base_path has been given)
        # * A reference to a block within the current app
        # * A reference to a block within a plugin (when the first component of the path matches a plugin name)

        if base_path is not None:
            r = self.block_path_to_dir(f"{base_path}.{path}")
            if r[0]:
                block_dir = r[1]

        app_dir = None
        if block_dir is None:
            r = self.block_path_to_dir(path)
            if not r[0]:
                paths = path.split(".")
                if paths[0] == "origen":
                    return origen.core_app.instantiate_block(
                            ".".join(paths[1:]),
                            None,
                            class_name=class_name,
                            sb_options=sb_options)
                elif path[0] == "origen_metal":
                    raise RuntimeError("origen_metal is not available as controller or application")
                else:
                    if len(paths) > 1 and origen.has_plugin(paths[0]):
                        return origen.plugin(paths[0]).instantiate_block(
                            ".".join(paths[1:]),
                            None,
                            class_name=class_name,
                            sb_options=sb_options)
                    else:
                        raise RuntimeError(
                            f"No block was found at path '{orig_path}'")
            else:
                block_dir = r[1]

        # If no controller class is defined then look up the nearest available parent
        controller_dir = block_dir
        controller_file = None
        blocks_dir = app_dir or self.app_dir.joinpath("blocks")
        p = f"{path.split('.')[-1]}.py"
        if controller_dir.joinpath(p).exists():
            controller_file = controller_dir.joinpath(p)
        else:
            while controller_dir != blocks_dir:
                if controller_dir.joinpath("controller.py").exists():
                    controller_file = controller_dir.joinpath("controller.py")
                    break
                elif controller_dir.joinpath(p).exists():
                    controller_file = controller_dir.joinpath(p)
                    break
                controller_dir = controller_dir.parent
                d = os.path.basename(controller_dir)
                if d == "derivatives":
                    controller_dir = controller_dir.parent
                # Nested blocks don't inherit controllers, they either have their own or use
                # a generic Origen controller
                elif d == "blocks":
                    break

        # If no controller was found in the app, fall back to the Origen Base controller
        if controller_file is None:
            if path == "dut":
                from origen.controller import TopLevel
                block = TopLevel()
            else:
                from origen.controller import Base
                block = Base()

        else:
            # Returns something like 'blocks/dut/derivatives/falcon/controller.py'
            p = os.path.relpath(controller_file, self.app_dir)
            # Now turn that into a Python import path
            p = p.replace("/", ".")
            p = p.replace("\\", ".")
            p = p.replace(".py", "")
            m = importlib.import_module(f"{self.name}.{p}")
            if hasattr(m, class_name):
                c = getattr(m, class_name)
                if 'kwargs' in inspect.signature(c).parameters:
                    block = c(**sb_options)
                else:
                    block = c()
            else:
                raise RuntimeError(
                    f"No class name '{class_name}' found in module {m}")

        block._app = self
        block._block_path = orig_path
        block._block_dir = block_dir

        return block

    def load_block_files(self, controller, filename):
        '''
            Load the given block filetype to the given controller
        
            >>> origen.app.load_block_files(dut.flash, "registers.py")
        '''
        # if isinstance(controller.block_path, ModuleType):
        #     return controller
        # fields = controller.block_path.split(".")
        # for i, field in enumerate(fields):
        #     if i == 0:
        #         filepath = origen.root.joinpath(
        #             self.name).joinpath("blocks").joinpath(fields[i])
        #     else:
        #         filepath = filepath.joinpath("derivatives").joinpath(fields[i])
        #     p = filepath.joinpath(filename)

        blocks_dir = self.app_dir.joinpath("blocks")
        load_dirs = []
        load_dir = controller.block_dir
        while load_dir != blocks_dir:
            load_dirs.insert(0, load_dir)
            load_dir = load_dir.parent
            d = os.path.basename(load_dir)
            if d == "derivatives":
                load_dir = load_dir.parent
            # A blocks dir means the end of the inheritance trail
            elif d == "blocks":
                break

        for load_dir in load_dirs:
            p = load_dir.joinpath(filename)
            if p.exists():
                if filename == "registers.py":
                    from origen.registers.loader import Loader
                    context = Loader(controller).api()

                elif filename == "sub_blocks.py":
                    from origen.sub_blocks import Loader
                    context = Loader(controller).api()

                elif filename == "pins.py":
                    from origen.pins import Loader
                    context = Loader(controller).api()

                elif filename == "timing.py":
                    from origen.timesets import Loader
                    context = Loader(controller).api()

                elif filename == "services.py":
                    from origen.services import Loader
                    context = Loader(controller).api()

                elif filename == "attributes.py":
                    from origen.attributes import Loader
                    context = Loader(controller).api()

                else:
                    block = controller
                    context = locals()
                origen.load_file(p, locals=context)

        return controller

    def translate(self, remote_file):
        '''
            Runs :class:`Origen's Translator <origen.translator.Translator>`

            See Also
            --------
            * :class:`origen.translator.Translator`
            * :meth:`origen.translator.Translator.translate`
        '''
        self.translator.translate(remote_file)

    def compile(self, *args, **options):
        '''
            Runs :class:`Origen's Compiler <origen.compiler.Compiler>`.
            **Note that** this will also run any existing jobs in the compiler's stack.
        
            See Also
            --------
            * :class:`origen.compiler.Compiler`
            * :meth:`origen.compiler.Compiler.run`
        '''
        self.compiler.run(*args, **options)
        return self.compiler


# def on_app_init(func):
#     _callbacks.register_listener("on_app_init", None, func)
#     return func


class Application(Base):
    def hey(self):
        return "you"
