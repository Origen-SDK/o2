import pytest
import origen, _origen
from tests.tester.test_tester_captures import Base
from tests.shared import clean_eagle, get_last_node, clean_tester


class TesterOverlayInterface(Base):
    def test_single_overlay(self):
        o = "test_overlaying_indefinitely"
        self.t.overlay(o)
        self.assert_overlay_node(o)

    def test_overlay_options(self):
        o = "test_overlay_with_options"
        self.t.overlay(label=o, cycles=8, symbol="O", pins=['portc'])
        self.assert_overlay_node(
            o,
            cycles=8,
            symbol="O",
            pin_ids=origen.dut.pins['portc'].__origen_pin_ids__)

    def test_pin_overlay(self):
        o = "test_pin_overlay"
        origen.dut.pins['portc'].overlay(o)
        self.assert_overlay_node(
            o, pin_ids=origen.dut.pins['portc'].__origen_pin_ids__)

    def test_pin_overlay_with_options(self):
        o = "test_pin_overlay_with_options"
        origen.dut.pins['portc'].overlay(o, cycles=2, symbol='D')
        self.assert_overlay_node(
            o,
            cycles=2,
            symbol='D',
            pin_ids=origen.dut.pins['portc'].__origen_pin_ids__)

    def test_pin_collection_overlay(self):
        o = "test_pin_collection_overlay"
        p = origen.dut.pins.collect('portc', 'clk')
        p.overlay(o)
        self.assert_overlay_node(o, pin_ids=p.__origen_pin_ids__)

    def test_pin_collection_overlay_with_options(self):
        o = "test_pin_collection_overlay_with_options"
        p = origen.dut.pins.collect('portc', 'clk')
        p.overlay(o, cycles=3, symbol='G')
        self.assert_overlay_node(o,
                                 cycles=3,
                                 symbol='G',
                                 pin_ids=p.__origen_pin_ids__)

    @pytest.mark.xfail
    def test_bit_collection_overlay(self):
        o = "test_bit_collection_overlay"
        origen.dut.arm_debug.switch_to_swd()
        origen.dut.reg('reg1').overlay(o)
        node = get_last_node()
        assert node["attrs"][0] == "RegOverlay"
        ovl = node["attrs"][1]["overlay"]
        assert ovl["label"] == o
        assert ovl["symbol"] == None
        assert ovl["cycles"] == None
        assert ovl["pin_ids"] == None
