# Registers added in the global scope of these files will be added to
# a memory map called 'default' and within that an address block called
# 'default'. Such regs can be accessed via my_block.regs and users who
# don't care about maps don't have to even think about them.
# Note that such regs can also be accessed via my_block.default.default.regs

# A simple reg definition with all bits writable, here at address 0 and a
# default size of 32-bits
SimpleReg("reg1", 0)
# Another simple reg with custom size
SimpleReg("reg2", 4, size=16)

# This is the reg definition
with Reg("reg3", 0x0024, size=16) as reg:
    # This is the COCO definition
    reg.bit(7, "coco", access="ro")
    reg.bit(6, "aien")
    reg.bit(5, "diff")
    reg.bit([4,0], "adch", reset=0x1F)

# Regs can be added within a defined memory map, and in this case no address
# block is given so that will mean they are placed in a default address block
# named 'default'.
with MemoryMap("user"):
    # Test that reg names can be reused when scoped within a different map
    SimpleReg("reg1", 0)

    with Reg("reg2", 0x0024, size=16) as reg:
        reg.bit([4,0], "adch", reset=0x1F)


# Finally regs can be added to a full declared scope like this:
with MemoryMap("test"):
    with AddressBlock("bank0"):
        # Test that reg names can be reused when scoped within a different map
        SimpleReg("reg1", 0)

        with Reg("reg2", 0x0024, size=16) as reg:
            reg.bit([4,0], "adch", reset=0x1F)
