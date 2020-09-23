with Pattern(pin_header="swd") as pat:
    origen.tester.cc("Reset SWD")
    origen.dut.swd.line_reset()

    origen.tester.cc("Write AP to 0xCECE_ECEC")
    origen.dut.swd.write_ap(0xCECE_ECEC, address=0)
    origen.tester.repeat(10)

    origen.tester.cc("Verify AP")
    origen.dut.swd.verify_ap(0xCECE_ECEC, address=0)
    origen.tester.repeat(10)

    origen.tester.cc("Write DP to 0x1234_ABCD")
    origen.dut.swd.write_dp(0x1234_ABCD, address=0)
    origen.tester.repeat(10)

    origen.tester.cc("Verify DP")
    origen.dut.swd.verify_dp(0x1234_ABCD, address=0)
    origen.tester.repeat(10)

    origen.tester.cc("Write AP with AP address expecting OK response")
    origen.dut.swd.write_ap(0x0BAD_C0DE, address=4, acknowledge="Ok")
    origen.tester.repeat(10)

    origen.tester.cc("Verify AP with AP address, expecting WAIT response, and verifying the parity bit")
    origen.dut.swd.verify_ap(0x0BAD_C0DE, address=8, acknowledge=origen.dut.swd.WAIT(), parity=0)
    origen.tester.repeat(10)

    origen.tester.cc("Write DP with DP address expecting FAULT response")
    origen.dut.swd.write_dp(0xC0DE_1BAD, address=0xC, acknowledge="Fault")
    origen.tester.repeat(10)

    origen.tester.cc("Verify DP with DP address, ignoring the target's acknowledgement, and verifying the parity bit")
    origen.dut.swd.verify_dp(0xC0DE_1BAD, address=0xC, acknowledge=origen.dut.swd.NONE())
    origen.tester.repeat(10)

    origen.tester.cc("Reset SWD")
    origen.dut.swd.line_reset()
