import origen

def test_translator_ip_xact():
    origen.app.instantiate_dut("dut.falcon")
    remote_file = f"{origen.root}/vendor/ip-xact/spirit1-4_ip-xact.xml"
    assert len(origen.dut.memory_maps) == 3
    breakpoint()
    origen.app.translate(remote_file)
    breakpoint()
    assert len(origen.dut.memory_maps) == 4
    # TODO: Must be a better way to check this
    assert str(type(origen.dut.memory_map("RegisterMap"))) == "<class 'MemoryMap'>"
    assert origen.dut.memory_map("RegisterMap").regs.len() == 2
    assert origen.dut.memory_map("RegisterMap").regs.keys() == ['dut_top_level_reg','dut_top_level_reg_number_two']
    # TODO: Cannot test register attributes currently due to this issue:
    # (Pdb) origen.dut.memory_map("RegisterMap").regs('dut_top_level_reg_number_two')
    # *** TypeError: 'Registers' object is not callable

def test_export_to_python():
    breakpoint()
    origen.app.to_python()