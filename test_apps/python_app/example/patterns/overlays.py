with Pattern(pin_header="all") as pat:
    port = origen.dut.pin("portc")
    reg = origen.dut.reg("reg1")
    t = origen.tester

    dut().timeset('simple').symbol_map['A'] = 'A'
    dut().timeset('simple').symbol_map['B'] = 'B'

    # Overlaying on the tester is WYSIWYG
    t.cc("Overlay the next cycle")
    t.overlay("test_overlaying_on_next_cycle").cycle()
    t.cycle()

    t.cc("Overlay the next two cycles")
    t.overlay("test_overlaying_on_next_two_cycles", cycles=2).repeat(2)
    t.repeat(4)

    t.cc("Overlaying on pins")
    t.overlay("test overlaying on pins", pins=["portc"], cycles=2).repeat(2)
    t.repeat(4)

    t.cc("Overlaying on pins, with options")
    t.overlay("test overlaying on pins, with options",
              pins=["portc"],
              cycles=2,
              mask=0x2,
              symbol='A').repeat(2)
    t.repeat(4)

    t.cc(
        "Overlay from pin group. This should functionally be the same as previous overlay"
    )
    port.overlay("test overlaying from pin group", cycles=2)
    t.repeat(4)

    t.cc("Overlay from pin group, with options")
    port.overlay("test overlaying from pin group, with options",
                 cycles=2,
                 mask=0x1,
                 symbol='A')
    t.repeat(4)

    t.cc("Overlay during drive operation")
    port.drive(0x3,
               overlay="test overlaying while driving pin group",
               overlay_cycles=2,
               overlay_symbol="B")
    t.repeat(4)

    t.cc(
        "Overlay during drive (again). Functionally, this should the same as the above"
    )
    port.overlay("test overlaying from pin group - version 2",
                 cycles=2,
                 symbol="B")
    port.drive(0x3)
    t.repeat(4)

    t.cc("Overlay during drive operation")
    port.verify(0x0,
                overlay="test overlaying while verifying pin group",
                overlay_cycles=2,
                overlay_symbol='A')
    t.repeat(4)

    # origen.tester.cc("Overlay while setting actions")
    # port.set_actions("HL", overlay="set_actions_overlay").repeat(2)
    # # origen.tester.repeat(2)
    # port.highz().cycle()

    # Test with set_overlay method

    # with port.overlay("ovl"):
    #     port.drive(0x1)
    #     port.drive(0x2)
    #     port.drive(0x3)

    # Test with overlay mask

    # Test with Pin Collections

    # Test with registers

    origen.dut.current_protocol = origen.dut.simple_32bit

    t.cc("Overlay a register write and verify with the same overlay")
    reg.set_overlay("Register Write/Verify Overlay")
    reg.set_data(0xCCCC_EEEE)
    reg.write()
    reg.verify()
    reg.clear_overlay()

    t.cc("These should not be overlayed")
    reg.write()
    reg.verify()

    # t.cc("Overlay On Register Write with symbol and mask")
    # reg.set_overlay("Register Write/Verify Overlay", symbol="A", mask=0xFFFF_0000)
    reg.set_overlay("Register Write/Verify Overlay", symbol="A")
    reg.write()
    reg.clear_overlay()

    # t.cc("One-liner to get similar functionality")
    # reg.overlay("Reg Overlay One-Liner", symbol="B", mask=0x0000_FFFF).write()

    # t.cc("Another one-liner using options instead")
    # reg.write(overlay="One-liner using options", overlay_symbol="A", overlay_mask=0x00FF_FF00)

    # t.cc("Try with verifying")
    # reg.overlay("Reg Overlay One-Liner", symbol="B", mask=0x0000_FFFF).verify()
    # reg.verify(overlay="One-liner using options", overlay_symbol="A", overlay_mask=0x00FF_FF00)
