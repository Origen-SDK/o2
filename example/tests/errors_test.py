import origen
import pytest
from origen.errors import *

def boot_falcon():
    return origen.app.instantiate_dut("dut.falcon") if origen.dut is None else origen.dut

def test_duplicate_instance_error():
    dut = boot_falcon()
    # Default error message
    with pytest.raises(DuplicateInstanceError) as err_info:
        raise DuplicateInstanceError(dut)
    assert str(err_info.value) == "Cannot create instance of 'example.blocks.dut.controller.Controller' named 'dut', it already exists!"
    # Using a class alias in the error message
    with pytest.raises(DuplicateInstanceError) as err_info:
        raise DuplicateInstanceError(dut, 'DUT')
    assert str(err_info.value) == "Cannot create instance of 'DUT' named 'dut', it already exists!"
    # Make sure users know the class alias argument must be a string
    with pytest.raises(TypeError) as err_info:
        raise DuplicateInstanceError('test_str', [1])
    assert str(err_info.value) == "Class alias must be of type 'str'!"