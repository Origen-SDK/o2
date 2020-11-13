import origen
import re
from os import access, R_OK
from os.path import isfile
from origen.registers.loader import Loader as Regs
from origen.sub_blocks import Loader as SubBlocks
from origen.errors import *
from .translators.ip_xact import IpXact


class Translator:
    def __init__(self):
        self.creator = None
        self.sb_creator = None

    def translate(self, remote_file):
        self.__init_creators()
        if self.__remote_ok(remote_file):
            snippet = "".join(self.__snippet(remote_file))
            if re.findall("spiritconsortium", snippet):
                ip_xact = IpXact(self.creator)
                ip_xact.parse(remote_file)

    def __remote_ok(self, remote_file):
        if not isfile(remote_file):
            raise FileNotFoundError
        if not access(remote_file, R_OK):
            raise PermissionError
        return True

    def __snippet(self, remote_file, lines=5):
        with open(remote_file) as curr_file:
            return [next(curr_file) for x in range(lines)]

    def __init_creators(self):
        '''This is necessary because the DUT is not loaded when the translator
        is initialized.  Perhaps this should be tied to a callback or
        the translator could be lazily instantiated'''
        if self.creator is None:
            self.creator = Regs(origen.dut)
        if self.sb_creator is None:
            self.sb_creator = SubBlocks(origen.dut)
