with Reg("ctrl", 0x0024, size=32):
    Field("enable", offset=0, width=1)
    Field("src", offset=1, width=4)
    Field("slow_nfast", offset=6, width=1)