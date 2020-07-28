with Pattern() as pat:
    dut().pin("portc").capture().cycle()
    dut().pin("portc").highz().cycle()
    tester().repeat(4)
    dut().pin("portc").capture().cycle()
    dut().pin("portc").highz().cycle()
    tester().repeat(9)