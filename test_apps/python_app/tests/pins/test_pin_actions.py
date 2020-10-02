import pytest
import origen, _origen # pylint: disable=import-error
from tests.shared import clean_eagle, clean_falcon, clean_tester # pylint: disable=import-error
from tests.shared.python_like_apis import Fixture_ListLikeAPI # pylint: disable=import-error

klass = origen.pins.PinActions
backend_klass =_origen.dut.pins.PinActions
standard_actions = {
  'Drive': 'D', 'Verify': 'V',
  'DriveHigh': '1', 'DriveLow': '0',
  'VerifyHigh': 'H', 'VerifyLow': 'L',
  'Capture': 'C', 'HighZ': 'Z'
}

def std_test():
  return klass("10HL")

class TestPinActions:

  def test_standard_actions(self):
    assert klass.standard_actions() == standard_actions

  @pytest.mark.parametrize("action", standard_actions.items())
  def test_standard_actions_instances(self, action):
    assert hasattr(klass, action[0])
    ins = getattr(klass, action[0])()
    assert isinstance(ins, backend_klass)
    assert str(ins) == action[1]
    assert ins.all_standard

  def test_comparison_to_pin_actions(self):
    assert klass.DriveHigh() == klass.DriveHigh()
    assert klass.DriveHigh() != klass.DriveLow()

  def test_comparison_to_str(self):
    assert klass.DriveHigh() == str(klass.DriveHigh())
    assert klass.DriveHigh() != str(klass.DriveLow())

  def test_new(self):
    actions = klass()
    assert isinstance(actions, backend_klass)
    assert len(actions) == 0
    assert str(actions) == ""

  def test_new_with_str(self):
    actions = klass("HLHL")
    assert isinstance(actions, backend_klass)
    assert len(actions) == 4
    assert str(actions) == "HLHL"

  def test_new_with_list(self):
    actions = klass("H", "L", "Z", klass.VerifyHigh(), klass.VerifyLow())
    assert isinstance(actions, backend_klass)
    assert len(actions) == 5
    assert str(actions) == "HLZHL"

  def test_type_error_with_new(self):
    with pytest.raises(TypeError):
      klass(None)
    with pytest.raises(TypeError):
      klass([])
    with pytest.raises(TypeError):
      klass("H", "L", [])

  def test_error_with_nonstandard_action(self):
    with pytest.raises(OSError) as e:
      klass("A")
    assert "Cannot derive PinActions enum from encoded character A!" in str(e.value)

  class TestPinActions(Fixture_ListLikeAPI):
    ''' Although this *feels* more like a ``str``, the actual list-like
        behavior is emulated like that of a ``list``.

        Include the ``ListLikeAPI`` tests then add some custom ones for ``str``
        in the ``PinActions`` tests.
    '''

    def parameterize(self):
      return {
        "slice_klass": _origen.dut.pins.PinActions
      }
      
    def verify_i0(self, i):
      assert isinstance(i, _origen.dut.pins.PinActions)
      assert i == _origen.dut.pins.PinActions.Drive()

    def verify_i1(self, i):
      assert isinstance(i, _origen.dut.pins.PinActions)
      assert i == _origen.dut.pins.PinActions.Verify()

    def verify_i2(self, i):
      assert isinstance(i, _origen.dut.pins.PinActions)
      assert i == _origen.dut.pins.PinActions.Capture()

    def boot_list_under_test(self):
      return klass("CVD")

  def test_representation(self):
    actions = std_test()
    assert str(actions) == "10HL"
    assert str(actions)[0] == "1"
    assert str(actions)[-1] == "L"
    assert actions[0] == klass.VerifyLow()
    assert actions[-1] == klass.DriveHigh()

  def test_custom_actions(self):
    actions = klass("|x|")
    assert str(actions) == "|x|"
    assert actions[0] == klass.Other("x")
    assert not actions.all_standard

  def test_multiple_other_actions(self):
    actions = klass("10|A||B|HL|C|Z")
    assert str(actions) == "10|A||B|HL|C|Z"
    assert not actions.all_standard
    assert actions[0] == klass.HighZ()
    assert actions[1] == klass.Other('C')
    assert actions[4] == klass.Other('B')
    assert actions[5] == klass.Other('A')

  def test_more_creating_other_actions(self):
    actions = klass("10|A||B|HL|C|Z")
    assert actions == "10|A||B|HL|C|Z"
    actions2 = klass("1", "0", "|A|", "|B|", "H", "L|C|Z")
    assert actions == actions2
    actions3 = klass(
      "10|A|",
      klass.Other('B'),
      klass.VerifyHigh(),
      klass.VerifyLow(),
      klass.Other('C'),
      "Z"
    )
    assert actions == actions3

  def test_overriding_standard_actions(self):
    actions = klass("10|1||0|")
    assert str(actions) == "10|1||0|"
    assert actions[0] == klass.Other('0')
    assert str(actions[0]) == '|0|'
    assert actions[1] == klass.Other('1')
    assert str(actions[1]) == '|1|'
    assert actions[2] == klass.DriveLow()
    assert str(actions[2]) == '0'
    assert actions[3] == klass.DriveHigh()
    assert str(actions[3]) == '1'
    assert not actions.all_standard


  def test_multi_char_symbols(self):
    actions = klass("1|Hi||Hello|")
    assert len(actions) == 3
    assert str(actions) == "1|Hi||Hello|"
    assert actions[0] == klass.Other("Hello")
    assert str(actions[0]) == '|Hello|'
    assert actions[1] == klass.Other("Hi")
    assert str(actions[1]) == '|Hi|'
    assert actions[2] == klass.DriveHigh()
    assert not actions.all_standard
