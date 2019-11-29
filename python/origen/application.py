import origen
import _origen
import importlib
import os.path
import re
import pdb

# The base class of all application classes
class Base:
    config = _origen.app_config()
    # Returns the unique ID (name) of the app/plugin
    id =  config["id"]

    # Translates something like "dut.falcon" to <root>/<app>/blocks/dut/derivatives/falcon
    def block_path_to_filepath(self, path):
        fields = path.split(".")
        filepath = origen.root.joinpath(self.id).joinpath('blocks')
        for i, field in enumerate(fields):
            if i > 0:
                filepath = filepath.joinpath('derivatives')
            filepath = filepath.joinpath(field)
        return filepath

    def instantiate_block(self, path):
        orig_path = path
        done = False
        # If no controller class is defined then look up the nearest available parent
        while not self.block_path_to_filepath(path).joinpath('controller.py').exists() and not done:
            p = path
            path = re.sub('\.[^\.]+$', "", path)
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
            controller = self.id + ".blocks." + controller + ".controller"
            m = importlib.import_module(controller)
            block = m.Controller()
        return block

    #def __repr__(self):
    #    return "<an app>"

# Load the application
def load():
    a = importlib.import_module(f'{_origen.app_config()["id"]}.application')
    app = a.Application()
    origen.app = app

