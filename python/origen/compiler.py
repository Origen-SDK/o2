import origen
import pathlib
import re
import mako
import os
from mako.template import Template
from os import access, W_OK, X_OK, R_OK
from origen.errors import *

class Compiler:
    class MakoSyntax:
        var_sub = re.compile('\$\{.*\}')
        ctrl_struct =  re.compile('^\s*\%.*\%')
        module_block = re.compile('\<\%\!.*\%\>')
        tag = re.compile('\<\%.*\>')
        expresions = [var_sub, ctrl_struct, module_block, tag]

        def inspect(self, arg):
            for regex in self.expresions:
                if regex.search(arg):
                    return True
            return False

    # Use a class variable as the syntax should be viewed
    # as immutable by Compiler instances
    syntax = MakoSyntax()

    def __init__(self, *args, **options):
        self.__check_args(*args)
        self.stack = list(args) if args else []
        self.renders, self.output_files = [], []

    # Allow the stack to be incremented, enabling compile at a later time
    def push(self, *args, **options):
        if not args:
            raise TypeError('No compiler arguments passed, cannot push them to the stack!')
        self.__check_args(*args)
        self.stack += list(args)

    # Pop the first item of the stack
    def pop(self):
        if not self.stack:
            raise RuntimeError("Cannot pop compiler stack, nothing on it!")
        else:
            self.stack.pop()
        
    # Run the compiler with the stack as-is or with new args
    def run(self, *args, **options):
        self.__check_args(*args)
        opts = {
            # TODO: Hook up to origen.app.output_directory
            'output_dir':    self.templates_dir(),
            'templates_dir': self.templates_dir()
        }
        # This was much easier in Ruby as the dict method 'update' acts
        # like Ruby's hash 'merge' method
        for k, v in options.items():
            if k in opts:
                opts[k] = v
        opts = self.__make_pathlibs_if_necessary(opts)
        if args:
            self.push(*args, **options)
        if self.stack:
            for arg in self.stack:
                if isinstance(arg, pathlib.Path):
                    # Compile the file
                    curr_template = Template(filename=str(arg))
                    self.__write_output_file(curr_template.render(), arg)
                else:
                    # Could be a file name, a file path, or templated text
                    if self.syntax.inspect(arg):
                        # Need to check that the user passed in a dictionary
                        # that contains the metadata needed to render
                        if options:
                            # arg is valid Mako, compile the text directly
                            curr_template = Template(arg)
                            # NOTE: If the options dict does not contain every (and only those) pieces 
                            # of metadata needed by the templated string, it will fail
                            self.renders.append(curr_template.render(**options))
                        else:
                            raise TypeError("Missing metadata to compile templated text!")
                    else:
                        # Check if the str is a file located in the templates directory
                        # or if it is direct path to a templated file
                        template_path = opts['templates_dir'] / arg
                        self.__check_template(template_path)
                        curr_template = Template(filename=f"{template_path}")
                        # TODO: Figure out how to get the current DUT and app loaded 
                        # automatically for all templates
                        self.__write_output_file(curr_template.render(), template_path)   
                self.stack.pop()
        else:
            raise TypeError('Compiler stack is empty, cannot run!')

    def last_render(self):
        return self.renders[-1] if self.renders else None
    
    def templates_dir(self):
        templates_dir = pathlib.Path(f"{origen.root}/{origen.app.name}/templates")
        if not templates_dir.exists():
            raise FileNotFoundError(f"Application templates directory does not exist at {templates_dir}")
        elif not templates_dir.is_dir():
            raise NotADirectoryError(f"Application templates directory exists at {templates_dir} but it is not a real directory!")
        elif not access(templates_dir, W_OK):
            raise PermissionError(f"Application templates directory exists at {templates_dir} but is not writeable!")
        else:
            return templates_dir

    def __write_output_file(self, template_output, template_path):
        origen.logger.info(f"Compiling mako template {template_path}")
        output_path = str(template_path)
        output_path = output_path.replace('.mako','')
        output_path = pathlib.Path(output_path)
        if output_path.exists():
            output_path.unlink()
        with open(output_path, 'w+') as f:
            f.write(template_output)
        # TODO: Figure out why this doesn't work
        output_path.chmod(0o755)
        self.output_files.append(output_path)
        origen.logger.info(f"Compiler output created at {output_path}")     
    
    def __check_args(self, *args):
        # Args must be either a pathlib or a str
        for arg in args:
            if not isinstance(arg, (str, pathlib.Path)):
                raise TypeError('Compiler arguments must be of type str or pathlib.Path')   
            if isinstance(arg, pathlib.Path):
                if not arg.suffix == '.mako':
                    raise FileExtensionError('.mako')

    def __make_pathlibs_if_necessary(self, opts):
        for k,v in dict(filter(lambda k: '_dir' in k[0] , opts.items())).items():
            opts[k] = v if isinstance(v, pathlib.PurePath) else pathlib.Path(v)
            opts[k].resolve()
        return opts

    def __check_template(self, t):
        if not t.exists():
            raise FileNotFoundError(f"Template file does not exist at {t}")
        elif not access(t, R_OK):
            raise PermissionError(f"Template file exists at {t} but is not readable!")
        elif not access(t, X_OK):
            raise PermissionError(f"Template file exists at {t} but is not executable!")