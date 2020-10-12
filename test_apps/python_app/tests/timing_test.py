import pytest, abc
import origen, _origen # pylint: disable=import-error
from origen.pins import PinActions
from shared import instantiate_dut, clean_dummy, clean_eagle, clean_tester, clean_falcon
from shared.python_like_apis import Fixture_DictLikeAPI, Fixture_ListLikeAPI
from tests.pins import ports, clk_pins

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

# Before we go into consuming Timeset-specifics and adding additional waves, ensure that the dict-like API is working.
# Otherwise, we'll get a bunch of failures that have nothing to do with the actual timeset.

class TestTimesetsDictLike(Fixture_DictLikeAPI):
  def parameterize(self):
    return {
      "keys": ["t0", "t1", "t2"],
      "klass": _origen.dut.timesets.Timeset,
      "not_in_dut": "Blah"
    }

  def boot_dict_under_test(self):
    instantiate_dut("dut.falcon")
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
    instantiate_dut("dut.falcon")
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
    instantiate_dut("dut.falcon")
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
    instantiate_dut("dut.falcon")
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
    instantiate_dut("dut.eagle")
    origen.tester.target("DummyRenderer")
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
  #   instantiate_dut("dut.eagle")
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
  def define_complex_timeset(self, clean_falcon, clk_pins, ports):
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
    # Add a base wave
    w = wgrp.add_wave("1")
    w.push_event(at="period*0.25", unit="ns", action=w.DriveHigh)
    # Derive from the above wave
    w = wgrp.add_wave("0", derived_from="1")
    w.events[0].action = w.DriveLow

  @pytest.mark.skip
  def test_wavetable_inheritance(self, define_complex_timeset):
    ''' STIL:

        Timing {
          WaveformTable w1 {
            Waveforms {
              all { 01ZLH { '0' D/U/Z/L/H } }
            }
          }
          WaveformTable backwards {
            InheritWaveformTable w1;
            Waveforms {
              backwards_pins { 01ZLH { '0' U/D/Z/H/L } }
            }
          }
        }
    '''
    w1 = origen.dut.timeset("complex").wavetable('w1')
    backwards = origen.dut.timeset("complex").add_wavetable("backwards", inherit="w1").add_wavegroup("BackwardsPorts")
    backwards.add_wave('0').push_action('0', DriveHigh)
    backwards.add_wave('1').push_action('0', DriveLow)
    backwards.add_wave('L').push_action('0', CompareHigh)
    backwards.add_wave('H').push_action('0', CompareLow)
    assert origen.dut.timeset("complex").wavetable("backwards").wavegroup("Ports") is not None
    forwards = origen.dut.timeset("complex").wavetable("backwards").wavegroup("Ports")

    # Check that "Ports" was inherited
    assert origen.dut.timeset("complex").wavetable("backwards").period == 40
    assert forwards.waves["Ports"].waves["1"].events[0].action == "U"
    assert forwards.waves["Ports"].waves["Z"].events[0].action == "Z"
    assert forwards.waves["Ports"].waves["L"].events[0].action == "L"

    # Check that "BackwardsPorts" was applied correctly
    assert backwards.waves["BackwardsPorts"].waves["1"].events[0].action == "D"
    assert backwards.waves["BackwardsPorts"].waves["Z"].events[0].action == "Z"
    assert backwards.waves["BackwardsPorts"].waves["L"].events[0].action == "H"

    # Check that inheritance works with an instance
    w2 = origen.dut.add_timeset("complex").add_wavetable("w2", inherit=w1)
    assert w2.waves["Ports"].waves["1"].events[0].action == "U"
    assert w2.waves["Ports"].waves["Z"].events[0].action == "Z"
    assert w2.waves["Ports"].waves["L"].events[0].action == "L"

  @pytest.mark.skip
  def test_wavetable_inheritance_override(self, define_complex_timeset):
    ''' STIL:

        Timing {
          WaveformTable w1 {
            Waveforms {
              all { 01ZLH { '0' D/U/Z/L/H } }
            }
          }
          WaveformTable w2 {
            InheritWaveformTable w1;
            Waveforms {
              all { ZC { '0' X/X } }
            }
          }
        }
    '''
    w1 = origen.dut.timeset("complex").wavetable('w1')
    w2 = origen.dut.timeset("complex").add_wavetable("w2", inherit="w1")
    assert w2.waves["all"].waves["1"].events[0].action == "U"
    assert w2.waves["all"].waves["0"].events[0].action == "D"
    assert w2.waves["all"].waves["Z"].events[0].action == "Z"
    assert "X" not in w2.waves["all"].waves
    wgrp = w2.add_wavegroup('all')
    wgrp.add_wave('Z').push_action('0', Unknown)
    wgrp.add_wave('C').push_action('0', Unknown)
    assert w2.waves["all"].waves["1"].events[0].action == "U"
    assert w2.waves["all"].waves["0"].events[0].action == "D"
    assert w2.waves["all"].waves["Z"].events[0].action == "X"
    assert w2.waves["all"].waves["X"].events[0].action == "X"

  @pytest.mark.skip
  def test_waveform_inheritance(self, define_complex_timeset):
    ''' STIL:

        Timing {
          WaveformTable w1 {
            Waveforms {
              all { 01ZLH { Start: '0' D/U/Z/L/H } }
            }
          }
          WaveformTable w2 {
            Waveforms {
              support_compare_z {
                InheritWaveform w1.all;
                X { '0' X }
              }
            }
          }
        }
    '''
    w1 = origen.dut.add_timeset("complex").wavetable('w1')
    w2 = origen.dut.add_timeset("complex").add_wavetable("w2", period=40)
    wgrp = w2.add_waves("support_compare_x", inherit=('w1', 'all'))
    assert w2.wavegroups.get('all') is None
    assert w2.wavegroups.get('support_compare_x') is not None
    wgrp.add_wave('X').push_event(at="0", unit="ns", action=w.CompareUnknown)
    assert set(wgrp.waves.keys) == ('0', '1', 'Z', 'H', 'L', 'X')
    assert wgrp.waves['0'].events[0].action == wave.DriveLow
    assert wgrp.waves['H'].events[0].action == wave.CompareHigh
    assert wgrp.waves['X'].events[0].action == wave.CompareUnknown

  @pytest.mark.skip
  def test_waveform_inheritance_override(self, define_complex_timeset):
    ''' STIL:

        Timing {
          WaveformTable w1 {
            Waveforms {
              all { 01ZLH { Start: '0' D/U/Z/L/H } }
            }
          }
          WaveformTable w2 {
            Waveforms {
              backwards_drives {
                InheritWaveform w1.all;
                01 { '0' U/D }
              }
            }
          }
        }
    '''
    w1 = origen.dut.add_timeset("complex").wavetable('w1')
    w2 = origen.dut.add_timeset("complex").add_wavetable("w2", period=40)
    wgrp = w1.add_waves("backwards", inherit=('w1', 'all'))
    assert w2.wavegroups.get('all') is None
    assert w2.wavegroups.get('backwards') is not None
    wgrp.add_wave('0').push_event(at="0", unit="ns", action=w.DriveHigh)
    wgrp.add_wave('1').push_event(at="0", unit="ns", action=w.DriveLow)
    assert set(wgrp.waves.keys) == ('0', '1', 'Z', 'H', 'L', 'X')
    assert wgrp.waves['1'].events[0].action == wave.DriveLow
    assert wgrp.waves['0'].events[0].action == wave.DriveHigh
    assert wgrp.waves['Z'].events[0].action == wave.HighZ
    assert wgrp.waves['H'].events[0].action == wave.CompareHigh
    assert wgrp.waves['L'].events[0].action == wave.CompareLow

  @pytest.mark.skip
  def test_creating_abstract_waves(self):
    w1 = origen.dut.add_timeset("complex").add_wavetable('shape')
    wgrp = w1.add_wavegroup('surround_z')
    w = wgrp.add_wave('0')
    w.push_abstract_event(HighZ)
    w.push_abstract_event(DriveLow)
    w.push_abstract_event(HighZ)
    assert w.is_abstract
    assert w.waves['0'].action == HighZ
    assert w.waves['1'].action == DriveLow
    assert w.waves['2'].action == HighZ
    w = wgrp.add_wave('1', derive_from='0')
    w.waves['1'].action = DriveHigh
    assert w.is_abstract
    assert w.waves['0'].action == HighZ
    assert w.waves['1'].action == DriveHigh
    assert w.waves['2'].action == HighZ

  @pytest.mark.skip
  def test_wavetable_inheritance_with_abstract_events(self, define_complex_timeset):
    ''' STIL:

        Timing {
          WaveformTable shape {
            Waveforms {
              surround_z { 01 { Z; U/D; Z } }
              clk { C { U; D } }
            }
          }
          WaveformTable application {
            period = '50ns'
            InheritWaveformTable shape;
            Waveforms {
              pins { 01 {'0', 'period/10', 'period - period/10'} }
              clock { C { '0', 'period/2' } }
            }
          }
        }
    '''
    w1 = origen.dut.add_timeset("complex").add_wavetable('shape')
    w2 = origen.dut.add_timeset("complex").add_wavetable("application", period=50, inherit="shape")
    wgrp_pins = w2.add_wavegroup('pins')
    wgrp_clk = w2.add_wavegroup('clock')
    w = wgrp_pins.add_wave('0')
    w.apply_events(("shape", "surround_z"), "0", "period/10", "period - period/10")
    assert len(w.events) == 3
    assert w.events[0].action == HighZ
    assert w.events[0].at == 0
    assert w.events[1].action == DriveLow
    assert w.events[1].at == 5
    assert w.events[2].action == HighZ
    assert w.events[2].at == 45
    
    w = wgrp.add_wave('1', derive_from='0')
    assert len(w.events == 3)
    assert w.events[0].action == HighZ
    assert w.events[0].at == 0
    assert w.events[1].action == DriveHigh
    assert w.events[1].at == 5
    assert w.events[2].action == HighZ
    assert w.events[2].at == 45

    w = wgrp_clk.add_wave("C")
    w.apply_events(("shape", "clock"), "0", "period/2")
    assert len(w.events == 2)
    assert w.events[0].action == DriveHigh
    assert w.events[0].at == 0
    assert w.events[1].action == DriveLow
    assert w.events[1].at == 25

  @pytest.mark.skip
  def test_mixing_concrete_and_abstract_events(self):
    w1 = origen.dut.add_timeset("complex").add_wavetable('shape')
    w = w1.add_wavegroup('pins').add_wave('0')
    w.push_event("0", "Z")
    w.push_event("period/10", "1")
    w.push_abstract_event("0")
    assert w.is_abstract
    w2 = origen.dut.add_timeset("complex").add_wavetable("application", period=50, inherit="shape")
    w.add_wavegroup("pins").add_wave("0")
    w.apply_events(("shape", "pins"), "period/2")
    assert w.events[0].action == HighZ
    assert w.events[0].at == 0
    assert w.events[1].action == DriveHigh
    assert w.events[1].at == 5
    assert w.events[2].action == DriveLow
    assert w.events[2].at == 25

  @pytest.mark.skip
  def test_exception_on_missing_event_applications(self, abstract_waves):
    ''' Given a wavetable

          WaveformTable shape {
            Waveforms {
              surround_z { 01 { Z; U/D; Z } }
              clk { C { U; D } }
            }
          }

        In order to apply the a wave from, say, 'surround_z' - which has three abstract events,
        the inheriting wave must supply exactly three events. No more, no less.
    '''
    w2 = origen.dut.add_timeset("complex").add_wavetable("application", period=50, inherit="shape")
    w.add_wavegroup("pins").add_wave("0")
    with pytest.raises(OSError):
      w.apply_events(("shape", "pins"), 0)
    with pytest.raises(OSError):
      w.apply_events(("shape", "pins"), 0, 5, 10, 15)

  @pytest.mark.skip
  def test_exception_on_unknown_wave_application(self, abstract_waves):
    w2 = origen.dut.add_timeset("complex").add_wavetable("application", period=50, inherit="shape")
    w.add_wavegroup("pins").add_wave("0")
    with pytest.raises(OSError):
      w.apply_events(("blah", "pins"), 0, 5, 10)
    with pytest.raises(OSError):
      w.apply_events(("shape", "blah"), 0, 5, 10)

  @pytest.mark.skip
  def test_exception_when_consuming_abstract_event_data(self):
    w1 = origen.dut.add_timeset("complex").add_wavetable('shape')
    wgrp = w1.add_wavegroup('surround_z')
    w = wgrp.add_wave('0')
    w.push_abstract_event(HighZ)
    assert w.action == HighZ
    with pytest.raises(OSError):
      w.at

  @pytest.mark.skip
  def test_exception_on_unknown_wave_inheritance(self):
    w2 = origen.dut.add_timeset("complex").add_wavetable("w2", period=40)
    with pytest.raises(OSError):
      w2.add_wavegroup("pins").add_wave("0", inherit=("blah", "0"))
    with pytest.raises(OSError):
      w2.add_wavegroup("pins").add_wave("0", inherit=("w1", "blah"))

  @pytest.mark.skip
  def test_exception_on_unknown_wavetable_inheritance(self):
    with pytest.raises(OSError):
      origen.dut.add_timeset("complex").add_wavetable("w2", period=40, inherit="blah!")

# class TestBuildingWavesWithBlocks

#   @pytest.mark.xfail
#   def test_with_static_timing(self):

#   @pytest.mark.xfail
#   def test_with_single_dynamic_applied_timing(self):

#   @pytest.mark.xfail
#   def test_with_multiple_dynamic_timing(self):

#   @pytest.mark.xfail
#   def test_with_mixed_timing(self):

#   @pytest.mark.xfail
#   def test_exception_on_unknown_wave(self):

#   @pytest.mark.xfail
#   def test_exception_when_using_abstract_waves(self):

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
    assert isinstance(wave.applied_to, list)
    applied_pins = ["porta0", "porta1", "porta2", "porta3", "portb0", "portb1"]
    for (i, p) in enumerate(wave.applied_to):
      assert p.name == applied_pins[i]
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
    ''' STIL:

        waveforms {
          PortOperations { 1 { 'period*0.25' 1; } }
        }
        =>
        waveforms {
          PortOperations { 10 { 'period*0.25' 1/0; } }
        }
        Or the equivalent:
        waveforms {
          PortOperations { 1 { 'period*0.25' 1; } }
          PortOperations { 0 { 'period*0.25' 0; } }
        }
    '''
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

def test_loader_api(clean_eagle, clean_dummy):
  assert len(origen.dut.timesets) == 4

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

# def test_setting_waveform_symbols():
#     t = origen.dut.add_timeset("test_symbols")
#     wtbl = t.add_wavetable("w1")
#     wtbl.period = "40"
#     wgrp = wtbl.add_waves("Ports")

#     # Add drive data waves
#     # This wave should tranlate into STIL as:
#     #   1 { DriveHigh: '10ns', U; }
#     w = wgrp.add_wave("1")
#     #w.indicator = "1"
#     w.apply_to("porta", "portb")
#     w.push_event(at="period*0.25", unit="ns", action=w.DriveHigh)

#     # This wave should tranlate into STIL as:
#     #   0 { DriveLow: '10ns', D; }
#     w = wgrp.add_wave("0")
#     #w.indicator = "0"
#     w.apply_to("porta", "portb")
#     w.push_event(at="period*0.25", unit="ns", action=w.DriveLow)

#     # Add highZ wave
#     # This wave should tranlate into STIL as:
#     #   Z { HighZ: '10ns', Z; }
#     w = wgrp.add_wave("Z")
#     #w.indicator = "Z"
#     w.apply_to("porta", "portb")
#     w.push_event(at="period*0.25", unit="ns", action=w.HighZ)

#     # Add comparison waves
#     # This wave should tranlate into STIL as:
#     #   H { CompareHigh: '4ns', H; }
#     w = wgrp.add_wave("H")
#     #w.indicator = "H"
#     w.apply_to("porta", "portb")
#     w.push_event(at="period*0.10", unit="ns", action=w.VerifyHigh)

#     # This wave should tranlate into STIL as:
#     #   L { CompareLow: '4ns', L; }
#     w = wgrp.add_wave("L")
#     #w.indicator = "L"
#     w.apply_to("porta", "portb")
#     w.push_event(at="period*0.10", unit="ns", action=w.VerifyLow)

#     wgrp = wtbl.add_waves("Clk")

#     # This wave should tranlate into STIL as:
#     #   1 { StartClk: '0ns', U; "@/2", D; }
#     w = wgrp.add_wave("1")
#     #w.indicator = "1"
#     w.apply_to("clk")
#     w.push_event(at=0, unit="ns", action=w.DriveHigh)
#     w.push_event(at="period/2", unit="ns", action=w.DriveLow)

#     # This wave should tranlate into STIL as:
#     #   0 { StopClk: '0ns', D; }
#     w = wgrp.add_wave("0")
#     #w.indicator = "0"
#     w.apply_to("clk")
#     w.push_event(at=0, unit="ns", action=w.DriveLow)

@pytest.mark.xfail
class TestSymbolMapDictLikeAPI(Fixture_DictLikeAPI):
  def parameterize(self):
    return {
      "keys": ['1', '0', 'H', 'L', 'C', 'Z'],
      "klass": str,
      "not_in_dut": "blah"
    }

  def boot_dict_under_test(self):
    instantiate_dut("dut.eagle")
    origen.tester.target("DummyRenderer")
    return origen.dut.timeset('simple').symbol_map

def test_default_symbol_map(clean_eagle, clean_dummy):
  assert dict(origen.dut.timeset('simple').symbol_map) == {
    '1': '1', '0': '0',
    'H': 'H', 'L': 'L',
    'C': 'C', 'Z': 'X'
  }

def test_setting_symbol(clean_eagle, clean_dummy):
  assert origen.dut.timeset('simple').symbol_map['1'] == '1'
  origen.dut.timeset('simple').symbol_map['1'] = '0'
  assert origen.dut.timeset('simple').symbol_map['1'] == '0'
  origen.dut.timeset('simple').symbol_map['1'] = 'Hi'
  assert origen.dut.timeset('simple').symbol_map['1'] == 'Hi'

def test_adding_custom_symbols(clean_eagle, clean_dummy):
  assert 'a' not in origen.dut.timeset('simple').symbol_map
  origen.dut.timeset('simple').symbol_map['a'] = 'b'
  assert 'a' in origen.dut.timeset('simple').symbol_map
  assert origen.dut.timeset('simple').symbol_map['a'] == 'b'

  # Custom symbols should also be retrievable via the "action string" syntax
  assert '|a|' in origen.dut.timeset('simple').symbol_map
  assert origen.dut.timeset('simple').symbol_map['|a|'] == 'b'

def test_getting_symbols_from_pin_actions(clean_eagle, clean_dummy):
  assert origen.dut.timeset('simple').symbol_map['1'] == '1'
  assert origen.dut.timeset('simple').symbol_map[PinActions.DriveHigh()] == '1'

def test_setting_symbols_from_pin_actions( clean_eagle, clean_dummy):
  assert origen.dut.timeset('simple').symbol_map['1'] == '1'
  origen.dut.timeset('simple').symbol_map[PinActions.DriveHigh()] = '2'
  assert origen.dut.timeset('simple').symbol_map[PinActions.DriveHigh()] == '2'

def test_exception_on_setting_symbols_from_invalid_pin_actions_size(clean_eagle, clean_dummy):
  with pytest.raises(ValueError):
    origen.dut.timeset('simple').symbol_map[PinActions("111")]
  with pytest.raises(ValueError):
    origen.dut.timeset('simple').symbol_map[PinActions("")]
  with pytest.raises(TypeError):
    origen.dut.timeset('simple').symbol_map[{}]

def test_corner_case__setting_custom_action_with_same_symbol_as_standard_action(clean_eagle, clean_dummy):
  assert origen.dut.timeset('simple').symbol_map['1'] == '1'
  origen.dut.timeset('simple').symbol_map['|1|'] = '2'
  # assert origen.dut.timeset('simple').symbol_map['1'] == '1'
  assert origen.dut.timeset('simple').symbol_map['|1|'] == '2'

def test_retrieving_all_symbol_maps(clean_tester, clean_eagle):
  origen.tester.target('J750')
  origen.tester.target('V93KSMT7')
  symbol_maps = origen.dut.timeset('simple').symbol_maps
  assert isinstance(symbol_maps, list)
  assert len(symbol_maps) == 3
  assert isinstance(symbol_maps[0], _origen.dut.timesets.SymbolMap)

def test_retreiving_symbol_map_for_a_particular_target(clean_tester, clean_eagle):
  origen.tester.target('J750')
  origen.tester.target('V93KSMT7')
  assert isinstance(origen.dut.timeset('simple').symbol_map.for_target('J750'), _origen.dut.timesets.SymbolMap)
  assert isinstance(origen.dut.timeset('simple').symbol_map.for_target('V93KSMT7'), _origen.dut.timesets.SymbolMap)

def test_retrieving_symbols_for_a_particular_target(clean_tester, clean_eagle):
  origen.tester.target('J750')
  origen.tester.target('V93KSMT7')
  assert origen.dut.timeset('simple').symbol_map.for_target('J750')['Z'] == 'X'
  assert origen.dut.timeset('simple').symbol_map.for_target('V93KSMT7')['Z'] == 'X'

def test_setting_symbols_sets_for_all_targets(clean_tester, clean_eagle):
  origen.tester.target('J750')
  origen.tester.target('V93KSMT7')
  assert origen.dut.timeset('simple').symbol_map.for_target('J750')['Z'] == 'X'
  assert origen.dut.timeset('simple').symbol_map.for_target('V93KSMT7')['Z'] == 'X'
  origen.dut.timeset('simple').symbol_map.set_symbol('Z', '|abc|')
  assert origen.dut.timeset('simple').symbol_map.for_target('J750')['Z'] == '|abc|'
  assert origen.dut.timeset('simple').symbol_map.for_target('V93KSMT7')['Z'] == '|abc|'

def test_setting_symbols_for_a_particular_target(clean_tester, clean_eagle):
  origen.tester.target('J750')
  origen.tester.target('V93KSMT7')
  assert origen.dut.timeset('simple').symbol_map.for_target('J750')['Z'] == 'X'
  assert origen.dut.timeset('simple').symbol_map.for_target('V93KSMT7')['Z'] == 'X'
  origen.dut.timeset('simple').symbol_map.set_symbol('Z', '|abc|', 'J750')
  assert origen.dut.timeset('simple').symbol_map.for_target('J750')['Z'] == '|abc|'
  assert origen.dut.timeset('simple').symbol_map.for_target('V93KSMT7')['Z'] == 'X'

def test_exception_on_invalid_symbol_map_target(clean_tester, clean_eagle):
  with pytest.raises(KeyError):
    origen.dut.timeset('simple').symbol_map.for_target('J750')
  with pytest.raises(KeyError):
    origen.dut.timeset('simple').symbol_map.set_symbol('Z', '|abc|', 'J750')
