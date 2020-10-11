# pylint: disable=undefined-variable
Timeset("simple", default_period=10)
t = Timeset("complex")
wtbl = t.add_wavetable("wtbl")

wtbl = t.add_wavetable("w1")
wtbl.period = "40"
wgrp = wtbl.add_waves("Ports")

# Add drive data waves
# This wave should tranlate into STIL as:
#   1 { DriveHigh: '10ns', U; }
w = wgrp.add_wave("1")
w.apply_to("porta", "portb")
w.push_event(at="period*0.25", unit="ns", action=w.DriveHigh)

# This wave should tranlate into STIL as:
#   0 { DriveLow: '10ns', D; }
w = wgrp.add_wave("0")
w.apply_to("porta", "portb")
w.push_event(at="period*0.25", unit="ns", action=w.DriveLow)

# Add highZ wave
# This wave should tranlate into STIL as:
#   Z { HighZ: '10ns', Z; }
w = wgrp.add_wave("Z")
w.apply_to("porta", "portb")
w.push_event(at="period*0.25", unit="ns", action=w.HighZ)

# Add comparison waves
# This wave should tranlate into STIL as:
#   H { CompareHigh: '4ns', H; }
w = wgrp.add_wave("H")
w.apply_to("porta", "portb")
w.push_event(at="period*0.10", unit="ns", action=w.VerifyHigh)

# This wave should tranlate into STIL as:
#   L { CompareLow: '4ns', L; }
w = wgrp.add_wave("L")
w.apply_to("porta", "portb")
w.push_event(at="period*0.10", unit="ns", action=w.VerifyLow)

wgrp = wtbl.add_waves("Clk")

# This wave should tranlate into STIL as:
#   1 { StartClk: '0ns', U; "@/2", D; }
w = wgrp.add_wave("1")
w.apply_to("clk")
w.push_event(at=0, unit="ns", action=w.DriveHigh)
w.push_event(at="period/2", unit="ns", action=w.DriveLow)

# This wave should tranlate into STIL as:
#   0 { StopClk: '0ns', D; }
w = wgrp.add_wave("0")
w.apply_to("clk")
w.push_event(at=0, unit="ns", action=w.DriveLow)

t = Timeset("backwards", default_period=40)
t.symbol_map['1'] = '0'
t.symbol_map['0'] = '1'
t.symbol_map[origen.pins.PinActions.VerifyHigh()] = 'L'
t.symbol_map[origen.pins.PinActions.VerifyLow()] = 'H'

t = Timeset("nonesense", default_period=10)
t.symbol_map['0'] = 'a'
t.symbol_map['1'] = 'b'
t.symbol_map['2'] = 'c'
t.symbol_map['3'] = 'd'
