import mako
from mako.template import Template

class Compiler:
    def __init__(self, *files, **options):
       self.stack = list(files) if isinstance(files, tuple) else []
       print('Added files to the compiler stack on init:')
       self.__print_files()
       self.__print_options(**options)

    def run(self, **options):
        if self.stack:
            self.__print_options(**options)
            print('Compiling these files on the stack:')
            self.__print_files()
            return True
        else:
            print('No files on the compiler stack, cannot run!')
            return False

    def add(self, *files, **options):
        self.stack.append(list(files))
        print('Adding the following files to the compiler:')      
        self.__print_files()
        self.__print_options(**options)

    def __print_options(self, **options):
        print('Running the compiler with these options:')
        for k,v in options.items():
            print(f"  {k}: {v}")

    def __print_files(self):
        for file_name in self.stack:
            print(f"  {file_name}")