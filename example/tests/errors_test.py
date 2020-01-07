import origen
import pytest
from origen.errors import *

def boot_falcon():
    return origen.app.instantiate_dut("dut.falcon") if origen.dut is None else origen.dut

def test_duplicate_detected_error():
    boot_falcon()
    assert list(origen.dut.sub_blocks.keys()) == ['core0', 'core1', 'core2', 'core3']
    with pytest.raises(DuplicateInstanceError) as err_info:
        origen.dut.add_sub_block('core0')
    assert str(err_info.value) == "Cannot create instance of 'sub_block' named 'core0', it already exists!"
