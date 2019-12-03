

reg("reg1", 0)
reg("reg2", 0x10)

with reg("reg1", 0x0024, size=16) as r:
    r.bit(7, "coco", access="ro")
    r.bit(6, "aien")
    r.bit(5, "diff")
    r.bit([4,0], "adch", reset=0x1F)

