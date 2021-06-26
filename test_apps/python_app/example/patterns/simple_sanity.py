with Pattern(pin_header="all") as pat:
    tester = origen.tester

    tester.cc("Simple 8-bit write/verify")
    dut().simple_8bit.reset()
    dut().simple_8bit.write_register(0xF, address=0xAB)
    dut().simple_8bit.verify_register(0xF, address=0xAB)

    tester.cc("Simple 32-bit write/verify")
    dut().simple_32bit.reset()
    dut().simple_32bit.write_register(0xCECE_FFFF, address=0x1234_5678)
    dut().simple_32bit.verify_register(0xCECE_FFFF, address=0x1234_5678)
