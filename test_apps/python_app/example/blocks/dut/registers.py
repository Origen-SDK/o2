from time import time
import pdb
# Registers added in the global scope of these files will be added to
# a memory map called 'default' and within that an address block called
# 'default'. Such regs can be accessed via my_block.regs and users who
# don't care about maps don't have to even think about them.
# Note that such regs can also be accessed via my_block.default.default.regs

# A simple reg definition with all bits writable, here at address 0 and a
# default size of 32-bits

with origen.reg_description_parsing():
    SimpleReg("reg1", 0)
# Another simple reg with custom size
SimpleReg("reg2", 4, size=16)

#for i in range(20000):
#    SimpleReg(f"areg{i}", 4)

NUM_REGS = 1#20000

#origen.logger.info(f"Building {NUM_REGS} regs")
start_time = time()
for i in range(NUM_REGS):
    # This is the reg description
    with Reg(f"areg{i}", 0x0024):
        # This is the COCO description
        Field("coco", offset=7, access="ro")
        Field("aien", offset=6)
        Field("diff", offset=5)
        Field("adch", offset=0, width=5, reset=0x1F, enums={
            # A simple enum
            "val1": 3,
            # A more complex enum, all fields except for value are optional
            "val2": { "value": 5, "usage": "w", "description": "The value of something"},
        })
end_time = time()
#origen.logger.info(f"Building {NUM_REGS} regs complete")
origen.logger.info(f"Building {NUM_REGS} regs took: {end_time - start_time}")

# Field adch has no reset value
with origen.reg_description_parsing():
    with Reg("breg0", 0x0024, size=16):
        Field("adch", offset=0, width=5)

# Field adch has a simple reset value
with Reg("creg0", 0x0024, size=16):
    Field("adch", offset=0, width=5, reset=0)
    
# Field adch has multiple reset values
with Reg("dreg0", 0x0024, size=16):
    Field("adch", offset=0, width=5, resets={
        # A simple reset value, 'hard' is equivalent to reset=5
        "hard": 5,
        # A more complex reset, all fields except for value are optional
        "async": { "value": 0xF, "mask": 0b1010 },
    })

# Regs can be added within a defined memory map, and in this case no address
# block is given so that will mean they are placed in a default address block
# named 'default'.
with MemoryMap("user"):
    # Test that reg names can be reused when scoped within a different map
    SimpleReg("reg1", 0)

    with Reg("reg2", 0x0024, size=16):
        Field("adch", offset=0, width=4, reset=0x5)


# Finally regs can be added to a fully declared scope like this:
with MemoryMap("test"):
    with AddressBlock("bank0"):
        # Test that reg names can be re-used when scoped within a different map
        SimpleReg("reg1", 0)

        with Reg("reg2", 0x0024, size=16):
            Field("adch", offset=0, width=4, reset=0x10)
