with Pattern(pin_header="swd") as pat:
    origen.tester.cc(
        "Wrap some of the commonly used tasks into their own methods")
    origen.dut.arm_debug.switch_to_swd()

    origen.tester.cc("The register model should be available")
    origen.dut.arm_debug.dp.idcode.verify(0xDEAD_C0DE)
    origen.dut.arm_debug.dp.ctrlstat.verify(0xF000_0000)  # Verify powered-up

    origen.tester.cc("Verify the various IDs of the MemAPs")
    origen.dut.arm_debug.sys.idr.verify()
    origen.dut.arm_debug.core1.idr.verify()
    origen.dut.arm_debug.core2.idr.verify()

    origen.tester.cc("Do some register writes & verifies")
    origen.dut.mem(0x1234).write(0x4321)
    origen.dut.mem(0xabcd).verify(0xabcd)
