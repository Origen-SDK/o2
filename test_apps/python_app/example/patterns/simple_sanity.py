with Pattern(pin_header="all") as pat:
    tester = origen.tester
    reg = origen.dut.reg("reg1")

    tester.cc("Simple 8-bit write/verify")
    dut().simple_8bit.reset()
    dut().simple_8bit.write_register(0xF, address=0xAB)
    dut().simple_8bit.verify_register(0xF, address=0xAB)

    tester.cc("Simple 32-bit write/verify")
    dut().simple_32bit.reset()
    dut().simple_32bit.write_register(0xCECE_FFFF, address=0x1234_5678)
    dut().simple_32bit.verify_register(0xCECE_FFFF, address=0x1234_5678)

    # origen.dut.current_protocol = origen.dut.simple_32bit
    # reg.set_data(0xCCCC_EEEE)
    # reg.enables = 0xFFFF_0000
    # reg.write()

    # tester.cc("Simple 32-bit write/verify with mask")
    # dut().simple_32bit.reset()
    # dut().simple_32bit.write_register(0xCECE_FFFF, address=0x1234_5678, mask=0xFFFF_0000)
    # dut().simple_32bit.verify_register(0xCECE_FFFF, address=0x1234_5678, mask=0x0000_FFFF)
