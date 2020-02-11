import origen
import re
import bs4
from origen.errors import *
from bs4 import BeautifulSoup
import os, sys

class IpXact:
    # The creator may need to be passed in the parse method if
    # there are multiple types of creators
    def __init__(self, creator):
        self.creator = creator

    class RegBitReset:
        def __init__(self, value, mask):
            self.value = value
            self.mask = mask
            if self.value:
                self.value = int(self.value, 16)
            if self.mask:
                self.mask = int(self.mask, 16)

    def parse(self, remote_file):
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
                                self.__create_regs(addr_block_tag)
                        else:
                            # Instantiate at the top level
                            self.__create_regs(addr_block_tag)

    def __create_regs(self, addr_block_tag):
        for reg_tag in addr_block_tag.find_all('spirit:register'):
             reg_name = reg_tag.find_next("spirit:name").text
             reg_descp = reg_tag.find_next("spirit:description").text
             reg_offset = reg_tag.find_next("spirit:addressOffset").text
             reg_offset = int(reg_offset, 16) if reg_offset else 0
             reg_size = int(reg_tag.find_next("spirit:size").text)
             reg_access = self.__format_access(reg_tag.find_next("spirit:access").text)
             reg_reset_tag = reg_tag.find_next("spirit:reset")
             reg_reset = self.RegBitReset((reg_reset_tag.find_next("value").text), (reg_reset_tag.find_next("mask").text))
             with self.creator.Reg(reg_name, reg_offset, size=reg_size) as reg:
                  self.__create_fields(reg, reg_tag)
                   
    def __create_fields(self, reg, reg_tag):
         for field_field_tag in reg_tag.find_all("spirit:field"):
             field_name = field_field_tag.find_next("spirit:name").text
             field_descp = field_field_tag.find_next("spirit:description").text
             field_offset = field_field_tag.find_next("spirit:bitOffset").text
             field_size = int(field_field_tag.find_next("spirit:bitWidth").text)
             field_offset = int(field_offset, 16) if field_offset else 0
             field_range = self.__calc_field_range(field_size, field_offset)
             field_access = self.__format_access(field_field_tag.find_next("spirit:access").text)
             # TODO: Need to figure out how to find a bit reset tag without finding the
             # reset tag from the next register
             # field_reset_tag = field_field_tag.find_next("spirit:reset")
             breakpoint()
             reg.Field(field_name, offset=field_offset, access=field_access, width=field_size)

    def __calc_field_range(self, field_size, field_offset):
        if field_size == 1:
            return field_offset
        else:
            end = field_offset + field_size
            return [end, field_offset]

    def __format_access(self, access):
        if '-' in access:
            access_strs = access.split('-')
            return f"{access_strs[0][0]}{access_strs[1][0]}"
        else:
            return access
