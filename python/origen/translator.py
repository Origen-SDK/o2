import origen
import re
import bs4
from bs4 import BeautifulSoup
from os import access, R_OK
from os.path import isfile
from origen.registers import Loader as Regs
from origen.sub_blocks import Loader as SubBlocks

class Translator:
    class RegBitReset:
        def __init__(self, value, mask):
            self.value = value
            self.mask = mask
            if self.value:
                self.value = int(self.value, 16)
            if self.mask:
                self.mask = int(self.mask, 16)

    def __init__(self):
        self.creator = None
        self.sb_creator = None

    def translate(self, remote_file):
        self.__init_creators()
        if self.__remote_ok(remote_file):
            snippet = "".join(self.__snippet(remote_file))
            if re.findall("spiritconsortium", snippet):
                self.__ip_xact_parse(remote_file)

    def __ip_xact_parse(self, remote_file):
        with open(remote_file,"r",encoding='utf8') as f:
            content = f.read()
            root = BeautifulSoup(content,'xml')
            for memory_map_tag in root.find_all('spirit:memoryMap'):
                memory_map_name = memory_map_tag.find_next("spirit:name").text
                with self.creator.MemoryMap(memory_map_name):
                    for addr_block_tag in root.find_all('spirit:addressBlock'):
                        addr_block_name = addr_block_tag.find_next("spirit:name").text.strip()
                        if addr_block_name:
                            with self.creator.AddressBlock(addr_block_name):
                                # Instantiate within an address block
                                self.__ip_xact_create_regs(addr_block_tag)
                        else:
                            # Instantiate at the top level
                            self.__ip_xact_create_regs(addr_block_tag)

    def __ip_xact_create_regs(self, addr_block_tag):
        for reg_tag in addr_block_tag.find_all('spirit:register'):
             reg_name = reg_tag.find_next("spirit:name").text
             reg_descp = reg_tag.find_next("spirit:description").text
             reg_offset = reg_tag.find_next("spirit:addressOffset").text
             reg_offset = int(reg_offset, 16) if reg_offset else 0
             reg_size = int(reg_tag.find_next("spirit:size").text)
             reg_access = self.__ip_xact_format_access(reg_tag.find_next("spirit:access").text)
             reg_reset_tag = reg_tag.find_next("spirit:reset")
             reg_reset = self.RegBitReset((reg_reset_tag.find_next("value").text), (reg_reset_tag.find_next("mask").text))
             with self.creator.Reg(reg_name, reg_offset, size=reg_size) as reg:
                  self.__ip_xact_create_bits(reg, reg_tag)
                   
    def __ip_xact_create_bits(self, reg, reg_tag):
         for bit_field_tag in reg_tag.find_all("spirit:field"):
             bit_name = bit_field_tag.find_next("spirit:name").text
             bit_descp = bit_field_tag.find_next("spirit:description").text
             bit_offset = bit_field_tag.find_next("spirit:bitOffset").text
             bit_size = int(bit_field_tag.find_next("spirit:bitWidth").text)
             bit_offset = int(bit_offset, 16) if bit_offset else 0
             bit_range = self.__ip_xact_calc_bit_range(bit_size, bit_offset)
             bit_access = self.__ip_xact_format_access(bit_field_tag.find_next("spirit:access").text)
             # TODO: Need to figure out how to find a bit access tag without finding the
             # access tag from the next register
             # bit_reset_tag = bit_field_tag.find_next("spirit:reset")
             reg.bit(bit_range, bit_name, access=bit_access)

    def __ip_xact_calc_bit_range(self, bit_size, bit_offset):
        if bit_size == 1:
            return bit_offset
        else:
            end = bit_offset + bit_size
            return [end, bit_offset]

    def __ip_xact_format_access(self, access):
        if '-' in access:
            access_strs = access.split('-')
            return f"{access_strs[0][0]}{access_strs[1][0]}"
        else:
            return access
    
    def __remote_ok(self, remote_file):
        if not isfile(remote_file):
            raise FileNotFoundError
        if not access(remote_file, R_OK):
            raise PermissionError
        return True

    def __snippet(self, remote_file, lines = 5):
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
    

