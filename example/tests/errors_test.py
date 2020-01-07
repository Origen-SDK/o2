import origen
import pytest
from origen.errors import *

def boot_falcon():
    return origen.app.instantiate_dut("dut.falcon") if origen.dut is None else origen.dut

def test_duplicate_instance_error():
    boot_falcon()
    assert list(origen.dut.sub_blocks.keys()) == ['core0', 'core1', 'core2', 'core3']
    with pytest.raises(DuplicateInstanceError) as err_info:
        origen.dut.add_sub_block('core0')
    assert str(err_info.value) == "Cannot create instance of 'sub_block' named 'core0', it already exists!"
    # Check the default error message
    with pytest.raises(DuplicateInstanceError) as err_info:
        raise DuplicateInstanceError('test_str')
    assert str(err_info.value) == "Cannot create instance of 'str', it already exists!"
    # Make sure users know the class alias argument must be a string
    with pytest.raises(TypeError) as err_info:
        raise DuplicateInstanceError('test_str', [1])
    assert str(err_info.value) == "Class alias must be of type 'str'!"