import origen, _origen, pytest # pylint: disable=import-error
from tests.shared import clean_falcon, clean_eagle # pylint: disable=import-error
from tests.pins import pins, is_pin_group # pylint: disable=import-error

class TestPinsInDUTHierarchy:
  def test_pins_in_subblocks(self, clean_falcon, pins):
    # We should have pins at the DUT level, but not in any subblocks.
    assert len(origen.dut.pins) == 4
    assert len(origen.dut.sub_blocks["core1"].pins) == 0

    # Add a pin at the subblock.
    assert origen.dut.sub_blocks["core1"].add_pin("p1")
    assert len(origen.dut.sub_blocks["core1"].pins) == 1
    p = origen.dut.sub_blocks["core1"].pin("p1")
    is_pin_group(p)

    # Add another pin
    assert origen.dut.sub_blocks["core1"].add_pin("_p1")
    assert len(origen.dut.sub_blocks["core1"].pins) == 2
    _p = origen.dut.sub_blocks["core1"].pin("_p1")
    is_pin_group(_p)

    # Verify the pins at origen.dut are unchanged.
    assert len(origen.dut.pins) == 4
    assert origen.dut.pin("_p1") is None

class TestPinMetadata:
  class MyRandomClass:
    pass

  def test_physical_pin_has_empty_metadata(self, clean_eagle):
    assert origen.dut.physical_pin("porta0").added_metadata == []

  def test_adding_metadata_to_physical_pin(self):
    # Essentially just check that nothing here throws an exception
    origen.dut.physical_pin("porta0").add_metadata("meta1", 1)
    origen.dut.physical_pin("porta0").add_metadata("meta2", "meta2!")
    origen.dut.physical_pin("porta0").add_metadata("meta3", {})
    origen.dut.physical_pin("porta0").add_metadata("meta4", TestPinMetadata.MyRandomClass())

  def test_getting_all_metadata_keys(self):
    assert origen.dut.physical_pin("porta0").added_metadata == ["meta1", "meta2", "meta3", "meta4"]

  def test_getting_metadata_from_physical_pin(self):
    assert origen.dut.physical_pin("porta0").get_metadata("meta1") == 1
    assert origen.dut.physical_pin("porta0").get_metadata("meta2") == "meta2!"
    assert isinstance(origen.dut.physical_pin("porta0").get_metadata("meta3"), dict)
    assert isinstance(origen.dut.physical_pin("porta0").get_metadata("meta4"), TestPinMetadata.MyRandomClass)

  def test_setting_existing_metadata_on_physical_pin(self):
    assert origen.dut.physical_pin("porta0").set_metadata("meta1", "hi!")
    assert origen.dut.physical_pin("porta0").set_metadata("meta2", "meta2 updated!")
    assert origen.dut.physical_pin("porta0").get_metadata("meta1") == "hi!"
    assert origen.dut.physical_pin("porta0").get_metadata("meta2") == "meta2 updated!"

  def test_setting_nonexistant_metadata_adds_it(self):
    assert origen.dut.physical_pin('porta0').get_metadata("meta5") is None
    assert origen.dut.physical_pin("porta0").set_metadata("meta5", 5.0) == False
    assert origen.dut.physical_pin("porta0").get_metadata("meta5") == 5.0

  def test_interacting_with_reference_metadata(self):
    d = origen.dut.physical_pin("porta0").get_metadata("meta3")
    assert isinstance(d, dict)
    assert "test" not in d
    d["test"] = True
    assert "test" in d
    d2 = origen.dut.physical_pin("porta0").get_metadata("meta3")
    assert "test" in d2

  def test_nonetype_on_retrieving_nonexistant_metadata(self):
    assert origen.dut.physical_pin("porta0").get_metadata("blah") is None

  def test_exception_on_adding_duplicate_metadata(self):
    with pytest.raises(OSError):
      origen.dut.physical_pin("porta0").add_metadata("meta1", False)

  def test_additional_metadata(self):
    origen.dut.physical_pin('porta1').add_metadata("m1", 1.0)
    origen.dut.physical_pin('porta1').add_metadata("m2", -2)
    assert origen.dut.physical_pin('porta1').get_metadata("m1") == 1.0
    assert origen.dut.physical_pin('porta1').get_metadata("m2") == -2
    assert origen.dut.physical_pin('porta0').get_metadata("m1") is None
    assert origen.dut.physical_pin('porta0').get_metadata("m2") is None

  def test_metadata_with_same_name_on_different_objects(self):
    origen.dut.physical_pin('porta0').add_metadata("index", 0)
    origen.dut.physical_pin('porta1').add_metadata("index", 1)
    assert origen.dut.physical_pin('porta0').get_metadata("index") == 0
    assert origen.dut.physical_pin('porta1').get_metadata("index") == 1

class TestPinLoaderAPI:
  def test_pin_loader_api(self, clean_eagle):
    assert origen.dut.pins.keys() == [
      "porta0", "porta1", "porta",
      "portb0", "portb1", "portb2", "portb3", "portb",
      "portc0", "portc1", "portc",
      "clk", "swd_clk", "swdclk", "tclk",
      "swdio", "reset"
    ]
    # assert origen.dut.pin("portc").reset_data == 0x3
    assert origen.dut.pin("clk").reset_actions == "0"
    assert origen.dut.pin_headers.keys() == ["ports", "clk", "all", "pins-for-toggle", "pins-for-toggle-rev", "swd"]
    assert origen.dut.pin_headers["ports"].pin_names == ["porta", "portb", "portc"]
    assert origen.dut.pin_headers["clk"].pin_names == ["clk"]
