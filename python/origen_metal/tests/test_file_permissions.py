import pytest
import origen_metal as om
from origen_metal.framework import FilePermissions

_FP = om._origen_metal.framework.file_permissions.FilePermissions

def new_fp(permissions):
    return FilePermissions(permissions)

def test_file_permissions_defaults():
    fp = new_fp(0o7)
    assert isinstance(fp, _FP)
    assert str(fp) == "custom(0o007)"
    assert fp.to_s == "custom(0o007)"
    assert int(fp) == 0o007
    assert fp.to_i == 0o007

    fp_custom = om.framework.file_permissions.custom(0o007)
    assert fp == fp_custom

def test_file_permission_comparison():
    assert new_fp(0o700) == new_fp(0o700)
    assert new_fp(0o700) != new_fp(0o710)

    with pytest.raises(NotImplementedError, match="FilePermissions only support equals and not-equals comparisons"):
        new_fp(0o777) >= new_fp(0o007)

enumerate_permissions = [
    (0o700, "private"),
    (0o750, "group"),
    (0o770, "group_writable"),
    (0o775, "public_with_group_writable"),
    (0o755, "public"),
    (0o777, "world_writable")
]

@pytest.mark.parametrize("as_i, as_s", enumerate_permissions)
def test_enumerate_permissions(as_i, as_s):
    fp = new_fp(as_i)
    assert str(fp) == as_s
    assert int(fp) == as_i

    fp = new_fp(as_s)
    assert str(fp) == as_s
    assert int(fp) == as_i
    fp = new_fp(as_s.upper())
    assert str(fp) == as_s
    assert int(fp) == as_i

    assert fp == getattr(om.framework.file_permissions, as_s)()

def test_custom_permissions_matches_with_enumerated_ones():
    fp = om.framework.file_permissions.custom(0o777)
    assert str(fp) == "world_writable"
    assert int(fp) == 0o777

def test_error_conditions():
    with pytest.raises(RuntimeError, match="Given permissions 0o1111 exceeds maximum supported Unix permissions 0o777"):
        new_fp(0o1111)

    with pytest.raises(RuntimeError, match="Cannot infer permissions from input 'hi!'"):
        new_fp("hi!")

    with pytest.raises(TypeError, match="Can not build FilePermissions from type 'list'"):
        new_fp([])
