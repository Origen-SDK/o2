import origen
import pytest
import importlib
from origen_metal import _origen_metal

from origen.tester import IGXL
tester_module = importlib.import_module("origen.tester")
from tests.shared import clean_falcon


class PinmapRecorder:
    def __init__(self):
        self.pins = []
        self.group_pins = []

    def add_pin(self, name, **options):
        self.pins.append((name, options))

    def add_group_pin(self, group, pin, **options):
        self.group_pins.append((group, pin, options))


class Container(dict):
    def keys(self):
        return list(super().keys())


class FakeEvent:
    def __init__(self, action, at, unit="ns"):
        self.action = action
        self.__at__ = at
        self.unit = unit


class FakeWave:
    def __init__(self, *events):
        self.events = events


class FakeWavetable:
    __period__ = "period"

    def applied_waves(self):
        return {
            "data0": {
                "1": FakeWave(FakeEvent("DriveHigh", "period*0.25")),
                "0": FakeWave(FakeEvent("DriveLow", "period*0.25")),
                "H": FakeWave(FakeEvent("VerifyHigh", "period*0.50")),
            }
        }


class FakeTimeset:
    __period__ = "period"
    default_period = 40
    wavetables = Container({"w1": FakeWavetable()})


class TimingRecorder:
    def __init__(self):
        self.rows = []

    def add_timeset_basic(self, *args, **kwargs):
        self.rows.append((args, kwargs))


def test_igxl_exposes_ultraflex_program_resource_api():
    for method in [
            "functional",
            "empty",
            "other",
            "ppmu",
            "pin_pmu",
            "dcvi_powersupply",
            "new_custom_test_instance",
            "new_patset",
            "new_patsubr",
            "add_reference",
            "new_job",
            "add_global_spec",
            "add_ac_spec",
            "add_dc_spec",
            "add_pin",
            "add_power_pin",
            "add_group_pin",
            "add_level",
            "add_edgeset",
            "add_timeset",
            "add_timeset_basic",
            "set_resource_filename",
            "add_dut_pinmap",
            "add_dut_timesets_basic",
    ]:
        assert hasattr(IGXL, method), method


def test_igxl_pinmap_can_be_derived_from_the_dut(clean_falcon):
    origen.dut.add_pin("data", width=2)
    origen.dut.add_pin("clock")
    recorder = PinmapRecorder()

    IGXL.add_dut_pinmap(
        recorder,
        groups=["data"],
        pin_type="I/O",
        comment="from DUT",
    )

    assert recorder.pins == [
        ("data0", {
            "pin_type": "I/O",
            "comment": "from DUT"
        }),
        ("data1", {
            "pin_type": "I/O",
            "comment": "from DUT"
        }),
        ("clock", {
            "pin_type": "I/O",
            "comment": "from DUT"
        }),
    ]
    assert recorder.group_pins == [
        ("data", "data0", {
            "pin_type": "I/O",
            "comment": "from DUT"
        }),
        ("data", "data1", {
            "pin_type": "I/O",
            "comment": "from DUT"
        }),
    ]


def test_igxl_basic_timesets_can_be_derived_from_the_dut(monkeypatch):
    fake_dut = type("FakeDut", (),
                    {"timesets": Container({"functional": FakeTimeset()})})()
    monkeypatch.setattr(tester_module.origen, "dut", fake_dut)
    recorder = TimingRecorder()

    IGXL.add_dut_timesets_basic(recorder)

    assert recorder.rows == [(("functional", "period", "data0"), {
        "setup": "i/o",
        "timing_mode": "Machine",
        "drive_on": "",
        "drive_data": "period*0.25",
        "drive_return": "",
        "drive_off": "",
        "compare_open": "period*0.50",
        "compare_close": "",
    })]


def test_igxl_timing_derivation_rejects_conflicting_edges():
    with pytest.raises(RuntimeError, match="Conflicting O2 timing events"):
        IGXL._single_igxl_edge(["1ns", "2ns"], "conflict")


def test_numeric_specs_use_origen_testers_scaling(tmp_path):
    prog_gen = _origen_metal.prog_gen
    prog_gen.reset()
    refs = prog_gen.start_new_flow("numeric_specs", None, None, None)
    uflex = IGXL("ULTRAFLEX")
    uflex.add_ac_spec("cycle", "nominal", typ=10e-9)
    uflex.add_dc_spec("vdd_main", "nominal", typ=0.002)
    uflex.add_dc_spec("ioh", "nominal", typ=0.002)
    prog_gen.end_flow(refs)

    files = prog_gen.render_program_for("ULTRAFLEX", str(tmp_path))
    assert str(tmp_path.joinpath("global.txt")) in files
    specs = tmp_path.joinpath("global.txt").read_text()
    assert "=10*ns" in specs
    assert "=0.002*V" in specs
    assert "=2*mA" in specs
