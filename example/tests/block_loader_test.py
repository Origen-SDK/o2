import origen

def test_files_get_loaded():
    origen.app.instantiate_block("dut.falcon")
    dut = origen.dut
    dut.load_regs()
    assert dut.model.number_of_regs() == 3

    origen.app.instantiate_block("dut.eagle")
    dut = origen.dut
    dut.load_regs()
    assert dut.model.number_of_regs() == 2
