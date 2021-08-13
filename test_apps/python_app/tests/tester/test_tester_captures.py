import pytest
import origen, _origen
from tests.shared import clean_eagle, get_last_node, clean_tester


class Base:
    @pytest.fixture(autouse=True)
    def clean_eagle(self, clean_eagle, clean_tester):
        pass

    @property
    def t(self):
        return origen.tester

    def assert_node(self,
                    symbol=None,
                    cycles=None,
                    enables=None,
                    pin_ids=None):
        node = get_last_node()
        assert node['attrs'][0] == 'Capture'
        assert node['attrs'][1][0]['symbol'] == symbol
        assert node['attrs'][1][0]['cycles'] == cycles
        assert node['attrs'][1][0]['enables'] == enables
        assert node['attrs'][1][0]['pin_ids'] == pin_ids
        assert len(node['children']) == 0

    def assert_overlay_node(self,
                            overlay,
                            symbol=None,
                            cycles=None,
                            enables=None,
                            pin_ids=None):
        node = get_last_node()
        assert node['attrs'][0] == 'Overlay'
        assert node['attrs'][1][0]['label'] == overlay
        assert node['attrs'][1][0]['symbol'] == symbol
        assert node['attrs'][1][0]['cycles'] == cycles
        assert node['attrs'][1][0]['enables'] == enables
        assert node['attrs'][1][0]['pin_ids'] == pin_ids
        assert len(node['children']) == 0


class TestCaptureInterface(Base):
    ''' Mostly simple tests to make sure the node gets into the AST correctly.

        Whether the node gets handled correctly is a matter for the
        individual renders to resolved.
    '''
    def test_blank_capture(self):
        origen.tester.capture()
        self.assert_node(None, None, None, None)

    def test_capture_with_options(self):
        origen.tester.capture(symbol='A', cycles=5, pins=['portc'])
        self.assert_node(cycles=5,
                         symbol='A',
                         pin_ids=origen.dut.pins['portc'].__origen_pin_ids__)

    def test_pin_capture(self):
        origen.dut.pins['portc'].capture()
        self.assert_node(pin_ids=origen.dut.pins['portc'].__origen_pin_ids__)

    def test_pin_capture_with_options(self):
        origen.dut.pins['portc'].capture(cycles=2, symbol='B')
        self.assert_node(cycles=2,
                         symbol='B',
                         pin_ids=origen.dut.pins['portc'].__origen_pin_ids__)

    def test_pin_collection_capture(self):
        p = origen.dut.pins.collect('portc', 'clk')
        p.capture()
        self.assert_node(pin_ids=p.__origen_pin_ids__)

    def test_pin_collection_capture_with_options(self):
        p = origen.dut.pins.collect('portc', 'clk')
        p.capture(cycles=3, symbol='E')
        self.assert_node(cycles=3, symbol='E', pin_ids=p.__origen_pin_ids__)

    def test_bit_collection_capture(self):
        origen.dut.arm_debug.switch_to_swd()
        origen.dut.reg('reg1').capture()
        node = get_last_node()
        assert node["attrs"][0] == "RegCapture"
        cap = node["attrs"][1]["capture"]
        assert cap["symbol"] == None
        assert cap["cycles"] == None
        assert cap["pin_ids"] == None

    def test_bit_collection_capture_with_symbol(self):
        origen.dut.arm_debug.switch_to_swd()
        origen.dut.reg('reg1').capture(symbol="F")
        node = get_last_node()
        assert node["attrs"][0] == "RegCapture"
        cap = node["attrs"][1]["capture"]
        assert cap["symbol"] == "F"
        assert cap["cycles"] == None
        assert cap["pin_ids"] == None
        assert cap["enables"] == [0xFFFF_FFFF]

    def test_bit_collection_capture_with_mask(self):
        origen.dut.arm_debug.switch_to_swd()
        origen.dut.reg('reg1').capture(symbol="F", mask=0xFFFF)
        node = get_last_node()
        assert node["attrs"][0] == "RegCapture"
        cap = node["attrs"][1]["capture"]
        assert cap["symbol"] == "F"
        assert cap["cycles"] == None
        assert cap["pin_ids"] == None
        assert cap["enables"] == [0xFFFF]

    def test_error_on_bit_collection_invalid_options(self):
        origen.dut.arm_debug.switch_to_swd()
        with pytest.raises(
                RuntimeError,
                match="'cycles' capture option is not valid in this context"):
            origen.dut.reg('reg1').capture(cycles=3)
        with pytest.raises(
                RuntimeError,
                match="'pins' capture option is not valid in this context"):
            origen.dut.reg('reg1').capture(pins=[])

    def test_capture_while_verifying_bit_collection(self):
        origen.dut.arm_debug.switch_to_swd()

        # Should be no capture to start
        origen.dut.reg('reg1').verify(0)
        node = get_last_node()
        assert node["attrs"][0] == "RegVerify"
        assert node["attrs"][1]["data"] == []
        assert node["attrs"][1]["capture"] is None

        # Set capture parameters but should not trigger a capture
        origen.dut.reg('reg1').set_data(0xFF)
        origen.dut.reg('reg1').set_capture()
        node = get_last_node()
        assert node["attrs"][0] == "RegVerify"
        assert node["attrs"][1]["data"] == []
        assert node["attrs"][1]["capture"] is None

        # Initiate verify - which also ships the capture along with it
        origen.dut.reg('reg1').verify()
        node = get_last_node()
        assert node["attrs"][0] == "RegVerify"
        assert node["attrs"][1]["data"] == [255]
        cap = node["attrs"][1]["capture"]
        assert cap["symbol"] == None
        assert cap["cycles"] == None
        assert cap["pin_ids"] == None

        # Initiate capture - which does not ship the verify along with it
        origen.dut.reg('reg1').set_data(0xF)
        origen.dut.reg('reg1').capture()
        node = get_last_node()
        assert node["attrs"][0] == "RegCapture"
        assert node["attrs"][1]["data"] == []
        cap = node["attrs"][1]["capture"]
        assert cap["symbol"] == None
        assert cap["cycles"] == None
        assert cap["pin_ids"] == None

        # Register settings are preserved for the next verify though
        origen.dut.reg('reg1').verify()
        node = get_last_node()
        assert node["attrs"][0] == "RegVerify"
        assert node["attrs"][1]["data"] == [15]
        cap = node["attrs"][1]["capture"]
        assert cap["symbol"] == None
        assert cap["cycles"] == None
        assert cap["pin_ids"] == None

    @pytest.mark.xfail
    def test_capture_while_verifying_bit_collection_with_options(self):
        origen.dut.reg('reg1').set_capture(mask=0xF, symbol="F")
        origen.dut.reg('reg1').verify(0x1, mask=0xFF)
        node = get_last_node()
        assert node["attrs"][0] == "RegVerify"
        assert node["attrs"][1]["data"] == [1]
        assert node["attrs"][1]["bit_enable"] == [0xFF]
        cap = node["attrs"][1]["capture"]
        assert cap["symbol"] == "F"
        assert cap["enables"] == [0xF]
        assert cap["cycles"] == None
        assert cap["pin_ids"] == None


class TestVectorBasedCaptures(Base):
    ''' Corner & error cases specific to the 'VectorBased' renderer '''
    def test_error_on_captures_exceeding_cycles(self):
        origen.target.setup(["tester/v93k_smt7.py", "dut/eagle.py"])
        origen.producer.continue_on_fail = False

        def error_on_captures_exceeding_cycles(self):
            with origen.producer.Pattern(
                    pin_header="cap_test",
                    name="test_error_on_captures_exceeding_cycles") as _pat:
                origen.tester.capture(cycles=1000).cycle()

        def error_on_captures_exceeding_cycles_with_pins(self):
            with origen.producer.Pattern(
                    pin_header="cap_test",
                    name="error_on_captures_exceeding_cycles_with_pins"
            ) as _pat:
                origen.tester.capture(cycles=1000, pins=['clk']).cycle()

        origen.target.setup(["tester/j750.py", "dut/eagle.py"])
        with pytest.raises(
                OSError,
                match="Pattern end reached but requested captures still remain"
        ):
            origen.producer.generate(error_on_captures_exceeding_cycles)
        origen.target.setup(["tester/j750.py", "dut/eagle.py"])
        with pytest.raises(
                OSError,
                match="Pattern end reached but requested captures still remain"
        ):
            origen.producer.generate(
                error_on_captures_exceeding_cycles_with_pins)

        # V93K overrides the capture node as standalone capture cycles are meaningless to it (currently)
        origen.target.setup(["tester/v93k_smt7.py", "dut/eagle.py"])
        with pytest.raises(
                OSError,
                match="Pattern end reached but requested captures still remain"
        ):
            origen.producer.generate(
                error_on_captures_exceeding_cycles_with_pins)

    def test_error_on_overlapping_captures(self):
        def error_on_overlapping_captures(context):
            with origen.producer.Pattern(
                    pin_header="cap_test",
                    name="test_error_on_overlapping_captures") as _pat:
                origen.tester.capture(cycles=5).cycle()
                origen.tester.capture(cycles=5).repeat(9)

        def error_on_overlapping_pin_captures(context):
            with origen.producer.Pattern(
                    pin_header="cap_test",
                    name="test_error_on_overlapping_pin_captures") as _pat:
                origen.dut.pin('clk').capture(cycles=5).cycle()
                origen.dut.pin('clk').capture(cycles=5).repeat(9)

        origen.producer.continue_on_fail = False

        origen.target.setup(["tester/v93k_smt7.py", "dut/eagle.py"])
        with pytest.raises(
                OSError,
                match=
                "Generic capture is already occurring. Cannot initiate another capture"
        ):
            origen.producer.generate(error_on_overlapping_captures)

        origen.target.setup(["tester/v93k_smt7.py", "dut/eagle.py"])
        with pytest.raises(
                OSError,
                match=
                "Capture requested on pin 'clk' but this pin is already capturing"
        ):
            origen.producer.generate(error_on_overlapping_pin_captures)
