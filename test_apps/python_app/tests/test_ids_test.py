import pytest
from origen_metal.prog_gen.test_ids import TestIDs as T
import os

@pytest.fixture
def t():
    t = T()
    yield t

def test_is_alive(t):
    t.include("bin", 3)
    t.include("softbin", 100, (200, 300))
    t.include("number", (10000, 20000))
    assert t.allocate("t1") == {"bin": 3, "softbin": 100, "number": 10000}
    assert t.allocate("t1") == {"bin": 3, "softbin": 100, "number": 10000}
    assert t.allocate("t2") == {"bin": 3, "softbin": 200, "number": 10001}

def test_bin_numbers_increment(t):
    t.include("bin", (1, 3))
    assert t.allocate("t1")["bin"] == 1
    assert t.allocate("t2")["bin"] == 2
    assert t.allocate("t3")["bin"] == 3

def test_duplicate_tests_same_bin(t):
    t.include("bin", (1, 3))
    assert t.allocate("t1")["bin"] == 1
    assert t.allocate("t2")["bin"] == 2
    assert t.allocate("t1")["bin"] == 1
    assert t.allocate("t3")["bin"] == 3

def test_caller_can_override_bin(t):
    t.include("bin", (1, 4))
    assert t.allocate("t1")["bin"] == 1
    assert t.allocate("t2", bin=3)["bin"] == 3
    assert t.allocate("t1")["bin"] == 1
    assert t.allocate("t2", bin=3)["bin"] == 3
    assert t.allocate("t3")["bin"] == 2
    assert t.allocate("t4", bin=10)["bin"] == 10
    assert t.allocate("t4")["bin"] == 10

def test_manually_assigned_bins_reserved(t):
    t.include("bin", (1, 4))
    assert t.allocate("t1")["bin"] == 1
    assert t.allocate("t2", bin=3)["bin"] == 3
    assert t.allocate("t3")["bin"] == 2
    assert t.allocate("t4")["bin"] == 4

def test_bin_assignments_can_be_inhibited(t):
    t.include("bin", (1, 4))
    assert t.allocate("t1")["bin"] == 1
    assert "bin" not in t.allocate("t1", bin=None)

def test_excluded_bins_not_used(t):
    t.include("bin", (1, 4))
    t.exclude("bin", 3)
    assert t.allocate("t1")["bin"] == 1
    assert t.allocate("t2")["bin"] == 2
    assert t.allocate("t3")["bin"] == 4

def test_system_can_be_saved_and_resumed(t):
    t.include("bin", (1, 4))
    assert t.allocate("t1")["bin"] == 1
    assert t.allocate("t2", bin=3)["bin"] == 3

    t.save("test.json")
    try:
        t = T("test.json")
        assert t.allocate("t3")["bin"] == 2
        assert t.allocate("t4")["bin"] == 4
        assert t.allocate("t2")["bin"] == 3
    finally:
        os.remove("test.json")

# Not sure if we want to keep this behavior from O1
#def test_previously_assigned_manual_bins_reclaimed(t):
#    t.include("bin", (1, 4))
#    assert t.allocate("t1")["bin"] == 1
#    assert t.allocate("t2")["bin"] == 2
#    assert t.allocate("t3", bin=2)["bin"] == 2
#
#    assert t.allocate("t1")["bin"] == 1
#    assert t.allocate("t2")["bin"] == 3
#    assert t.allocate("t3")["bin"] == 2

def test_when_all_bins_are_used_they_will_be_reused_oldest_first(t):
    t.include("bin", (1, 3))
    assert t.allocate("t1")["bin"] == 1
    assert t.allocate("t2")["bin"] == 2
    assert t.allocate("t3")["bin"] == 3
    assert t.allocate("t4")["bin"] == 1
    assert t.allocate("t4")["bin"] == 1
    assert t.allocate("t5")["bin"] == 2

    assert t.allocate("t1")["bin"] == 1
    assert t.allocate("t2")["bin"] == 2
    assert t.allocate("t3")["bin"] == 3
    assert t.allocate("t1")["bin"] == 1  # More recent reference makes 2 the oldest
    assert t.allocate("t6")["bin"] == 2

    assert t.allocate("t1")["bin"] == 1
    assert t.allocate("t2")["bin"] == 2
    assert t.allocate("t3")["bin"] == 3
    assert t.allocate("t1")["bin"] == 1  # More recent reference makes 2 the oldest
    assert t.allocate("t7")["bin"] == 2
    assert t.allocate("t8")["bin"] == 3

def test_tests_can_reserve_multiple_bins(t):
    t.include("bin", (10, 30))
    t.increment("bin", 5)

    t1 = t.allocate("t1", size=2)
    assert t1["bin"] == 10
    t2 = t.allocate("t2")
    assert t2["bin"] == 12  # Incremented by 2
    t3 = t.allocate("t3")
    assert t3["bin"] == 17  # Incremented by 5
    t4 = t.allocate("t4")
    assert t4["bin"] == 22
    t5 = t.allocate("t5")
    assert t5["bin"] == 10  # Reusing oldest
    t6 = t.allocate("t6")
    assert t6["bin"] == 15
