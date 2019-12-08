reg("reg1", 0)

# This is the reg definition
with Reg("reg2", 0x0024, size=16) as reg:
    # This is the COCO definition
    reg.bit(7, "coco", access="ro")
    reg.bit(6, "aien")
    reg.bit(5, "diff")
    reg.bit([4,0], "adch", reset=0x1F)
