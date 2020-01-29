import pytest, abc
import origen, _origen # pylint: disable=import-error
from shared.python_like_apis import Fixture_DictLikeAPI, Fixture_ListLikeAPI

@pytest.fixture
def clean_eagle():
  origen.app.instantiate_dut("dut.eagle")
  assert origen.dut
  return origen.dut

@pytest.fixture
def clean_falcon():
  origen.app.instantiate_dut("dut.falcon")
  assert origen.dut
  return origen.dut

def test_empty_timesets(clean_falcon):
  assert len(origen.dut.timesets) == 0
  # assert origen.tester.timeset is None
  # assert origen.tester.current_period is None

def test_adding_a_simple_timeset(clean_falcon):
  t = origen.dut.add_timeset("t0", default_period=1)
  assert isinstance(t, _origen.dut.timesets.Timeset)
  assert t.name == "t0"
  assert t.default_period == 1
  assert t.__eval_str__ == "period"
  assert t.period == 1.0

  # Check the DUT
  assert len(origen.dut.timesets) == 1
  assert origen.dut.timesets.keys() == ["t0"]

  # # Ensure this doesn't to anything to the tester
  # assert origen.tester.timeset is None
  # assert origen.tester.current_period == 0

def test_adding_another_simple_timeset():
  t = origen.dut.add_timeset("t1")
  assert isinstance(t, _origen.dut.timesets.Timeset)
  assert t.name == "t1"

  # No default period set, so attempts to resolve the numerical value will fail.
  assert t.default_period == None
  assert t.__eval_str__ == "period"
  with pytest.raises(OSError):
    t.period

  # Check the DUT
  assert len(origen.dut.timesets) == 2
  assert origen.dut.timesets.keys() == ["t0", "t1"]

  # # Ensure this doesn't to anything to the tester
  # assert origen.tester.timeset is None
  # assert origen.tester.current_period == 0

def test_retrieving_timesets():
  t = origen.dut.timeset("t0")
  assert isinstance(t, _origen.dut.timesets.Timeset)
  assert t.name == "t0"

def test_none_on_retrieving_nonexistant_timesets():
  t = origen.dut.timeset("t")
  assert t is None

def test_exception_on_duplicate_timesets():
  with pytest.raises(OSError):
    origen.dut.add_timeset("t0")
  assert len(origen.dut.timesets) == 2
  assert origen.dut.timesets.keys() == ["t0", "t1"]

def test_adding_timeset_with_equation(clean_falcon):
  t = origen.dut.add_timeset("t0", "period/2", default_period=1)
  assert t.default_period == 1.0
  assert t.__eval_str__ == "period/2"
  assert t.period == 0.5

# def test_adding_simple_timeset_with_context_manager(clean_falcon):
#   with origen.dut.new_timeset("t0") as _t:
#     _t.default_period = 1
#     _t.period = "period/4 + 0.5"
#   assert isinstance(t, _origen.Timeset)
#   assert t.name == "t0"
#   assert t.default_period == 1
#   assert t.__eval_str__ == "period/4 + 0.5"
#   assert t.period == 0.75
#   assert len(origen.dut.timesets) == 1
#   assert origen.dut.timesets.keys == ["t0"]

# def test_adding_complex_timesets(clean_falcon):
#   with origen.dut.new_timeset("t0") as _t:
#     _t.default_period = 1
#     with _t.drive_wave("swd_clk") as w:
#       w.drive(1, "period/4")
#       w.drive(0, "period/2")
#       w.drive(1, "3*period/4")
#       w.drive(0, "period")
#     with _t.verify_wave("swd_io") as w:
#       w.verify("data", "period/10")
#   assert isinstance(t, _origen.Timeset)
#   assert t.name == "t0"
#   assert t.default_period == 1

#   assert len(t.drive_waves) == 4
#   assert t.drive_waves[0].__eval_str__ == "period/4"
#   assert t.drive_waves[0].data == 1
#   assert t.drive_waves[0].at == "0.25"
#   for i, wave in enumerate(t.drive_waves):
#     assert wave.at == 0.25*(i+1)
  
#   assert len(t.verify_waves) == 1
#   assert t.verify_waves[0].__expr_str__ == "data"
#   assert t.verify_waves[0].data == 0
#   assert t.verify_waves[0].at == 0.1

# This fixture assumes that a DUT is already instantiated.
# @pytest.fixture
# def define_basic_swd_timing(self):
#   swdio_wave = wtbl.add_wave("SWDIO")
#   swdio_wave.apply_to("swdio")
#   swdio_wave.add_event("swd_io_drive1")

# Before we go into consuming Timeset-specifics and adding addtional waves, ensure that the dict-like API is working.
# Otherwise, we'll get a bunch of failures that have nothing to do with the actual timeset.

class TestTimesetsDictLike(Fixture_DictLikeAPI):
  def parameterize(self):
    return {
      "keys": ["t0", "t1", "t2"],
      "klass": _origen.dut.timesets.Timeset,
      "not_in_dut": "Blah"
    }

  def boot_dict_under_test(self):
    origen.app.instantiate_dut("dut.falcon")
    dut = origen.dut
    dut.add_timeset("t0")
    dut.add_timeset("t1")
    dut.add_timeset("t2")
    return dut.timesets

class TestWaveTablesDictLike(Fixture_DictLikeAPI):
  def parameterize(self):
    return {
      "keys": ["wtbl1", "wtbl2"],
      "klass": _origen.dut.timesets.Wavetable,
      "not_in_dut": "Blah"
    }

  def boot_dict_under_test(self):
    origen.app.instantiate_dut("dut.falcon")
    t = origen.dut.add_timeset("t")
    t.add_wavetable("wtbl1")
    t.add_wavetable("wtbl2")
    return t.wavetables

class TestWaveGroupDictLike(Fixture_DictLikeAPI):
  def parameterize(self):
    return {
      "keys": ["w1", "w2", "w3"],
      "klass": _origen.dut.timesets.WaveGroup,
      "not_in_dut": "Blah"
    }

  def boot_dict_under_test(self):
    origen.app.instantiate_dut("dut.falcon")
    wt = origen.dut.add_timeset("t").add_wavetable("wt")
    wt.add_wave("w1")
    wt.add_wave("w2")
    wt.add_wave("w3")
    return wt.waves

class TestWavesDictLike(Fixture_DictLikeAPI):
  def parameterize(self):
    return {
      "keys": ["1", "0", "H", "L"],
      "klass": _origen.dut.timesets.Wave,
      "not_in_dut": "Blah"
    }

  def boot_dict_under_test(self):
    origen.app.instantiate_dut("dut.falcon")
    wgrp = origen.dut.add_timeset("t").add_wavetable("wt").add_waves("wgrp")
    wgrp.add_wave("1")
    wgrp.add_wave("0")
    wgrp.add_wave("H")
    wgrp.add_wave("L")
    return wgrp.waves

class TestEventsListLike(Fixture_ListLikeAPI):

  def verify_i0(self, i):
    assert isinstance(i, _origen.dut.timesets.Event)
    assert i.__at__ == "period*0.25"

  def verify_i1(self, i):
    assert isinstance(i, _origen.dut.timesets.Event)
    assert i.__at__ == "period*0.50"

  def verify_i2(self, i):
    assert isinstance(i, _origen.dut.timesets.Event)
    assert i.__at__ == "period*0.75"

  def boot_list_under_test(self):
    origen.app.instantiate_dut("dut.eagle")
    w = origen.dut.add_timeset("t").add_wavetable("wt").add_waves("w1").add_wave("1")

    w.push_event(at="period*0.25", unit="ns", action=w.DriveHigh)
    w.push_event(at="period*0.50", unit="ns", action=w.DriveLow)
    w.push_event(at="period*0.75", unit="ns", action=w.HighZ)
    return w.events

# class TestEventsListLike(Fixture_ListLikeAPI):
#   def equals(i, expected):
#     assert i.action == expected["action"]
#     assert i.data == expected["data"]
#     assert i.at == expected["at"]

#   def parameterize(self):
#     return {
#       "items": [
#         {action: "drive", data: 1, at: "0"},
#         {action: "drive", data: 0, at: "period*0.25"},
#         {action: "drive", data: 1, at: "period*0.5"},
#         {action: "drive", data: 0, at: "period*0.75"}
#       ],
#       "klass": _origen.dut.timesets.Event,
#       "not_in_dut": "Blah"
#     }

  # def boot_dict_under_test(self):
  #   origen.app.instantiate_dut("dut.eagle")
  #   w = origen.dut.add_timeset("t").add_wavetable("wt").add_wave("w")
  #   w.add_event("drive 1", "0")
  #   w.add_event("drive 0", "period*0.25")
  #   w.add_event("drive 1", "period*0.5")
  #   w.add_event("drive 0", "period*0.75")
  #   return w.events

class TestComplexTimingScenerios:
  
  # Test adding a timeset with more complex features.
  # This example should translate to the following STIL
  #   WaveformTable w1 {
  #     Period 'period';
  #     Waveforms {
  #       clk { 1 { StartClk: '0ns', U; "@/2", D; } }
  #       clk { 0 { StopClk: '0ns', D; } }
  #     }
  #   }
  @pytest.fixture
  def define_complex_timeset(self, clean_falcon):
    t = origen.dut.add_timeset("complex")
    wtbl = t.add_wavetable("w1")
    wtbl.period = "40"
    wgrp = wtbl.add_waves("Ports")

    # Add drive data waves
    # This wave should tranlate into STIL as:
    #   1 { DriveHigh: '10ns', U; }
    w = wgrp.add_wave("1")
    #w.indicator = "1"
    w.apply_to("porta", "portb")
    w.push_event(at="period*0.25", unit="ns", action=w.DriveHigh)

    # This wave should tranlate into STIL as:
    #   0 { DriveLow: '10ns', D; }
    w = wgrp.add_wave("0")
    #w.indicator = "0"
    w.apply_to("porta", "portb")
    w.push_event(at="period*0.25", unit="ns", action=w.DriveLow)

    # Add highZ wave
    # This wave should tranlate into STIL as:
    #   Z { HighZ: '10ns', Z; }
    w = wgrp.add_wave("Z")
    #w.indicator = "Z"
    w.apply_to("porta", "portb")
    w.push_event(at="period*0.25", unit="ns", action=w.HighZ)

    # Add comparison waves
    # This wave should tranlate into STIL as:
    #   H { CompareHigh: '4ns', H; }
    w = wgrp.add_wave("H")
    #w.indicator = "H"
    w.apply_to("porta", "portb")
    w.push_event(at="period*0.10", unit="ns", action=w.VerifyHigh)

    # This wave should tranlate into STIL as:
    #   L { CompareLow: '4ns', L; }
    w = wgrp.add_wave("L")
    #w.indicator = "L"
    w.apply_to("porta", "portb")
    w.push_event(at="period*0.10", unit="ns", action=w.VerifyLow)

    wgrp = wtbl.add_waves("Clk")

    # This wave should tranlate into STIL as:
    #   1 { StartClk: '0ns', U; "@/2", D; }
    w = wgrp.add_wave("1")
    #w.indicator = "1"
    w.apply_to("clk")
    w.push_event(at=0, unit="ns", action=w.DriveHigh)
    w.push_event(at="period/2", unit="ns", action=w.DriveLow)

    # This wave should tranlate into STIL as:
    #   0 { StopClk: '0ns', D; }
    w = wgrp.add_wave("0")
    #w.indicator = "0"
    w.apply_to("clk")
    w.push_event(at=0, unit="ns", action=w.DriveLow)

  # The fixture above is the linearized, more verbose, but more explicit way to define timing.
  # The fixtures below will generate equivalent timesets using 'syntatic sugar' to make the definitions more user friendly.
  # Note though that these fixtures don't do anything too crazy. If crazy waves are needed, the interface above will need to be used.
  # @pytest.fixture
  # def define_complex_timing_more_easily(self, clean_falcon):
  #   # Adds five individual waves and returns them as a group.
  #   waves = origen.dut.add_timeset("complex").add_wavetable("w1", period="40").add_waves("PortOperations", indicators=["1", "0", "H", "L", "Z"])
  #   waves["H"]
  #   origen.dut.add_timeset("complex").wavetable["w1"].waves["PortOperations-H"]

  #   # Add drive data waves
  #   # This wave should tranlate into STIL as:
  #   #   10HLZ { Drives: '10ns', U/D/H/L/Z; }
  #   #waves.indicators = 
  #   waves.apply_to("porta", "portb")
  #   waves.push_event(at="period*0.25", unit="ns", action=[waves.DriveHigh, waves.DriveLow, waves.VerifyHigh, waves.VerifyLow, waves.HighZ])

  # @pytest.fixture
  # def define_complex_timing_with_context_managers(self, clean_falcon):
  #   with origen.dut.add_timeset("complex_context_manager") as t:
  #     with t.add_wavetable("w1") as wtbl:
  #       wtbl.period = "40"
  #       with wtbl.add_waves("PortOperations", 5) as (waves, e0, e1, e2, e3, e4):
  #         waves.apply_to("porta", "portb")
  #         waves.indicators = ["1", "0", "H", "L", "Z"]
  #         waves.unit = "ns"
  #         e0.to_drive_high_wave(at="period*0.25")
  #         e1.to_drive_low_wave(at="period*0.25")
  #         e2.to_compare_high_wave(at="period*0.25")
  #         e3.to_compare_low_wave(at="period*0.25")
  #         e4.to_highz_wave(at="period*0.25")
  
  # @pytest.fixture
  # def define_wavetable_inheritance(self, clean_falcon):
  #   with origen.dut.add_timeset("inherited_wavetable") as t:
  #     with t.add_wavetable("default") as wtbl:
  #       wtbl.period = "40"
  #       with wtbl.add_waves("Clk") as w:
  #         w.apply_to("clk")
  #         w.indicators = ["0", "1"]
  #         w.push_drive_low_event()
  #         w.push_clocking_event()
  #       with wtbl.add_waves("ShiftedClk", base="Clk") as grp:
  #         grp.apply_to("other_clk")
  #         grp.waves[1].to_clocking_event(at="")

  @pytest.fixture
  def define_waves_derived_from(self, clean_falcon):
    wgrp = origen.dut.add_timeset("complex").add_wavetable("w1", period="40").add_waves("PortOperations")
    origen.dut.timesets["complex"].wavetables["w1"].period = 40
    w = wgrp.add_wave("1")
    w.push_event(at="period*0.25", unit="ns", action=w.DriveHigh)
    w = wgrp.add_wave("0", derived_from="1")
    w.events[0].action = w.DriveLow

  # We're assuming timesets, wavetable, etc. have already passed their appropriate dict/list-like_api test, so assuming that
  # interface is good there and just filling in some missing pieces.
  def test_retrieving_timeset_data(self, define_complex_timeset):
    wtbl = origen.dut.timesets["complex"].wavetables["w1"]
    
    # Try to get the resolved period.
    assert wtbl.period == 40

    # Get the period before evaluation
    assert wtbl.__period__ == "40"

    # Get the pins which have waves defined for them in the wavetable
    # clk_waves = wtbl.waves.applied_to("clk")
    # assert isinstance(clk_waves, dict)
    # assert len(clk_waves) == 2
    # assert set(clk_waves.keys()) == {"StartClk", "StopClk"}
    # assert isinstance(clk_waves["StartClk"], _origen.dut.timesets.Wave)
    # assert clk_waves["StartClk"].name == "StartClk"

    # Given a single waveform, retrieve its events
    wave = wtbl.waves["Ports"].waves["1"]
    assert wave.indicator == "1"
    assert wave.applied_to == ["porta", "portb"]
    assert len(wave.events) == 1
    e = wave.events[0]
    assert isinstance(e, _origen.dut.timesets.Event)
    assert e.action == wave.DriveHigh
    assert e.unit == "ns"
    assert e.__at__ == "period*0.25"
    assert e.at == 10

    wave = wtbl.waves["Clk"].waves["1"]
    e = wave.events[0]
    assert e.action == wave.DriveHigh
    assert e.unit == "ns"
    assert e.__at__ == "0"
    assert e.at == 0
    e = wave.events[1]
    assert e.action == wave.DriveLow
    assert e.unit == "ns"
    assert e.__at__ == "period/2"
    assert e.at == 20

  def test_retrieving_timeset_data_derived_from(self, define_waves_derived_from):
    wgrp = origen.dut.timesets["complex"].wavetables["w1"].waves["PortOperations"]
    w1 = wgrp.waves['1']
    w2 = wgrp.waves['0']
    assert w1.indicator == '1'
    assert w1.events[0].at == 10
    assert w1.events[0].unit == "ns"
    assert w1.events[0].action == w1.DriveHigh
    assert w2.indicator == '0'
    assert w2.events[0].at == 10
    assert w2.events[0].unit == "ns"
    assert w2.events[0].action == w2.DriveLow
    # Ensure that w2 was derived from w1, but does not share references to its events.
    assert w1.events[0].action == w1.DriveHigh

  def test_exception_on_duplicate_wavetable(self, define_complex_timeset):
    assert 'complex' in origen.dut.timesets
    assert 'w1' in origen.dut.timesets['complex'].wavetables
    with pytest.raises(OSError):
      origen.dut.timesets['complex'].add_wavetable('w1')
  
  def test_exception_on_duplicate_wave_group(self, define_complex_timeset):
    assert 'complex' in origen.dut.timesets
    assert 'w1' in origen.dut.timesets['complex'].wavetables
    assert 'Ports' in origen.dut.timesets['complex'].wavetables['w1'].waves
    with pytest.raises(OSError):
      origen.dut.timesets['complex'].wavetables['w1'].add_wave('Ports')

  def test_exception_on_duplicate_wave(self, define_complex_timeset):
    assert 'complex' in origen.dut.timesets
    assert 'w1' in origen.dut.timesets['complex'].wavetables
    assert 'Ports' in origen.dut.timesets['complex'].wavetables['w1'].waves
    assert '1' in origen.dut.timesets['complex'].wavetables['w1'].waves['Ports'].waves
    with pytest.raises(OSError):
      origen.dut.timesets['complex'].wavetables['w1'].waves['Ports'].add_wave('1')

  def test_exception_on_event_missing_action(self, define_complex_timeset):
    w = origen.dut.timesets['complex'].wavetables['w1'].waves['Ports'].waves["1"]
    with pytest.raises(TypeError):
      w.push_event(at="10")
  
  def test_exception_on_event_missing_at(self, define_complex_timeset):
    w = origen.dut.timesets['complex'].wavetables['w1'].waves['Ports'].waves["1"]
    with pytest.raises(TypeError):
      w.push_event(action=w.DriveHigh)
  
  def test_exception_on_evaluating_with_missing_period(self, clean_falcon):
    w = origen.dut.add_timeset('t').add_wavetable('wtbl').add_wave('w').add_wave("1")
    w.push_event(at="period/2", action=w.DriveHigh)
    assert origen.dut.timesets['t'].wavetables['wtbl'].period is None
    assert w.events[0].__at__ == "period/2"
    with pytest.raises(OSError):
      w.events[0].at
  
  def test_exception_on_unknown_action(self, define_complex_timeset):
    w = origen.dut.timesets['complex'].wavetables['w1'].waves['Ports'].waves["1"]
    with pytest.raises(OSError):
      w.push_event(action="blah!", at="period/2")

def test_loader_api(clean_eagle):
  assert len(origen.dut.timesets) == 2

  # Test adding some more waveforms. The updated Waveform Table should now look like:
  #   WaveformTable w1 {
  #     Period 'period';
  #     Waveforms {
  #       porta portb { 
  #         1 { DriveHigh: '0ns', U; }
  #         0 { DriveLow: '0ns', D; }
  #         H { ExpectHigh: '0ns', G; }
  #         L { DriveLow: '0ns', Q; }
  #         Z { HighZ: '0ns', Z; }
  #         C { Capture: '0ns', V; }
  #       }
  #       clk {
  #         1 { StartClk: '0ns', U; "@/2", D; }
  #         0 { StopClk: '0ns', D; }
  #       }
  #     }
  #   }
  #def test_adding_to_existing_timeset(self):
  #  pass

# ### With Tester ####

# class TestDummy:
#   def test_dummy_class(self):
#     assert False