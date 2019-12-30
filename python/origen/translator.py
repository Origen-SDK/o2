import re
import bs4
from bs4 import BeautifulSoup
from os import access, R_OK
from os.path import isfile

class Translator:

    def __parse_ip_xact(self, remote_file):
        with open(remote_file,"r",encoding='utf8') as f:
            content = f.read()
            root = BeautifulSoup(content,'xml')
            for memory_map in root.find_all('spirit:memoryMap'):
                for child in memory_map.children:
                    breakpoint();

    def __remote_ok(self, remote_file):
        if not isfile(remote_file):
            raise FileNotFoundError
        if not access(remote_file, R_OK):
            raise PermissionError
        return True

    def __snippet(self, remote_file):
        with open(remote_file) as curr_file:
            return [next(curr_file) for x in range(5)]

    def translate(self, remote_file):
        if self.__remote_ok(remote_file):
            snippet = "".join(self.__snippet(remote_file))
            if re.findall("spiritconsortium", snippet):
                self.__parse_ip_xact(remote_file)

    

