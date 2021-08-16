with Pattern(pin_header="cap_test") as pat:
    tester = origen.tester
    portc = origen.dut.pin("portc")
    reg = origen.dut.reg1
    dut().timeset('simple').symbol_map['A'] = 'A'
    dut().timeset('simple').symbol_map['B'] = 'B'

    tester.cc("Basics")
    tester.cc("---")
    tester.cc("Capture a single cycle")
    tester.capture().cycle()
    tester.cycle()
    tester.cc("Capture a single cycle on portc")
    tester.capture(pins=["portc"]).cycle()
    tester.cycle()
    tester.cc("Capture two cycles on portc")
    tester.capture(pins=["portc"], cycles=2).repeat(2)
    tester.cc("Capture three cycles on portc with symbol 'A'")
    tester.capture(pins=["portc"], cycles=3, symbol="A").repeat(3)
    tester.cc("Capture four cycles on portc with symbol 'B'")
    tester.capture(pins=["portc"], cycles=4, symbol="B").repeat(2)
    tester.repeat(2)
    tester.cc("Capture four cycles on portc and clk")
    tester.capture(pins=["portc", "clk"], cycles=4).repeat(10)

    tester.cc("Basics with implied pins (portc)")
    tester.cc("---")
    tester.cc("Capture next cycle (portc)")
    portc.capture(cycles=1).cycle()
    tester.cycle()
    tester.cc("Capture next two cycles (portc)")
    portc.capture(cycles=2).repeat(2)
    tester.cycle()
    tester.cc("Capture next two cycles with symbol 'A' (portc)")
    portc.capture(cycles=2, symbol="A").repeat(2)
    tester.cycle()
    tester.cc(
        "Capture next two cycles with symbol 'B' masking the second bit (portc)"
    )
    portc.capture(cycles=2, symbol="B", mask=0x2).repeat(2)
    tester.cycle()

    tester.cc("Two captures with symbols")
    tester.cc("---")
    tester.cc("This however, is fine.")
    portc.capture(cycles=2, symbol="A").repeat(2)
    portc.capture(cycles=2, symbol="B").repeat(2)
    tester.cycle()

    origen.dut.arm_debug.switch_to_swd()
    tester.cc("Capturing a register (using arm debug)")
    tester.cc("---")
    tester.cc("Capture 'reg1'")
    reg.capture()
    tester.cc("Capture 'reg1' with symbol 'A'")
    reg.capture(symbol='A')
    tester.cc("Capture 'reg1' with symbol 'B' and mask 0xFFFF")
    reg.capture(mask=0xFFFF, symbol='B')

    # tester.cc("Verify with capture options")
    # tester.cc("---")
    # tester.cc("Verify 'reg1' with data 0 while capturing")
    reg.set_data(0xCECE_CECE)
    # reg.verify(capture=True)
    # tester.cc("Verify 'reg1' with data 0 while capturing with symbol 'A'")
    # reg.verify(capture={"symbol": 'A'})
    # tester.cc("Verify 'reg1' with data 0 while capturing with symbol 'B' and mask 0xFFFF")
    # reg.verify(capture={"symbol": 'B', "mask": 0xFFFF})

    tester.cc(
        "Verify with captures previously set. Next two transactions will be captured and verified"
    )
    tester.cc("---")
    tester.cc("Capture next two transactions")
    reg.set_capture()
    reg.verify()
    reg.verify()
    reg.clear_capture()
    tester.cc("This should not be captured")
    reg.verify()

    # Changing capture configuration
    with tester.eq("v93k") as v93k:
        tester.cc("--Should only render on v39k--")
        tester.cycle()
    # tester.capture_config.digcap.method = "digcap"
    # tester.capture_config.symbol = "E"
    # tester.capture(cycles=4)
    # tester.capture_config.digcap.method = None
    # tester.capture_config.symbol = None
    # tester.capture(cycles=4)

    # To Do: Capture with Options
