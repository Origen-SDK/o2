import origen
import _origen
import importlib
import os.path
import re
from origen.controller import TopLevel
from origen.translator import Translator
from origen.compiler import Compiler
from origen.errors import *


class Base(_origen.application.PyApplication):
    '''
        The base class of all Origen ``applications``.
    '''
    @property
    def name(self):
        ''' Returns the unique ID (name) of the app/plugin '''
        return _origen.app_config()["name"]

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
        return origen.root

    __instantiate_dut_called = False

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

    def __init__(self, *args, **options):
        self._compiler = Compiler()
        self._translator = Translator()
        if origen.app is None:
            self._plugin = False
        else:
            self._plugin = True

    def block_path_to_filepath(self, path):
        ''' Translates something like "dut.falcon" to <root>/<app>/blocks/dut/derivatives/falcon '''
        fields = path.split(".")
        filepath = origen.root.joinpath(self.name).joinpath('blocks')
        for i, field in enumerate(fields):
            if i > 0:
                filepath = filepath.joinpath('derivatives')
            filepath = filepath.joinpath(field)
        return filepath

    def instantiate_dut(self, path):
        ''' Instantiate the given DUT and return it, this must be called first before any
            sub-blocks can be instantiated '''
        if origen.dut is not None:
            raise RuntimeError(
                "Only one DUT target can be loaded, your current target selection instantiates multiple DUTs"
            )
        if origen._target_loading is not True:
            raise RuntimeError(
                "A DUT can only be instantiated within a target load sequence")
        self.__instantiate_dut_called = True
        dut = self.instantiate_block(path)
        if not isinstance(dut, TopLevel):
            raise RuntimeError(
                "The DUT object is not an instance of origen.application::TopLevel"
            )
        origen.dut = dut
        return dut

    def instantiate_block(self, path):
        '''
            Instantiate the given block and return it
        
            >>> origen.app.instantiate_block("dut.falcon")
            >>> origen.app.instantiate_block("nvm.flash.f2mb")
        '''
        if not self.__instantiate_dut_called:
            raise RuntimeError(
                f"No DUT has been instantiated yet, did you mean to call 'origen.instantiate_dut(\"{path}\")' instead?"
            )

        orig_path = path
        done = False
        # If no controller class is defined then look up the nearest available parent
        while not self.block_path_to_filepath(path).joinpath(
                'controller.py').exists() and not done:
            p = path
            path = re.sub(r'\.[^\.]+$', "", path)
            done = p == path

        # If no controller was found in the app, fall back to the Origen Base controller
        if done:
            if path == "dut":
                from origen.controller import TopLevel
                block = TopLevel()
            else:
                from origen.controller import Base
                block = Base()
        else:
            controller = '.derivatives.'.join(path.split("."))
            controller = self.name + ".blocks." + controller + ".controller"
            m = importlib.import_module(controller)
            block = m.Controller()

        block.app = self
        block.block_path = orig_path

        return block

    def load_block_files(self, controller, filename):
        '''
            Load the given block filetype to the given controller
        
            >>> origen.app.load_block_files(dut.flash, "registers.py")
        '''
        fields = controller.block_path.split(".")
        for i, field in enumerate(fields):
            if i == 0:
                filepath = origen.root.joinpath(
                    self.name).joinpath("blocks").joinpath(fields[i])
            else:
                filepath = filepath.joinpath("derivatives").joinpath(fields[i])
            p = filepath.joinpath(filename)
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
