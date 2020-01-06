import origen
import pytest
from origen.errors import *

def boot_falcon():
    if origen.app is None:
        return origen.app.instantiate_dut("dut.falcon")
    else:
        return origen.app

def test_duplicate_detected_error():
    app = boot_falcon()
    with pytest.raises(DuplicateDetectedError) as err_info:
        raise DuplicateDetectedError(app)
    assert str(err_info.value) == "Cannot create instance of class 'example.application.Application' named 'example', it already exists!"
