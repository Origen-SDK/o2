"""Tests for exception-safe program-generation context managers."""

import pytest
import origen_metal._origen_metal as om


@pytest.fixture(autouse=True)
def reset_program_generator():
    """Reset global program-generation state around every test."""
    om.prog_gen.reset()
    yield
    om.prog_gen.reset()


@pytest.mark.parametrize("context_name", ["group", "condition", "resources"])
def test_context_closes_when_body_raises(context_name):
    """An exceptional context exit must not leave its AST node open."""
    interface = om.interface.PyInterface()
    outer_refs = om.prog_gen.start_new_flow("outer", None, None, None)
    child_refs = om.prog_gen.start_new_flow("child", True, None, None)

    if context_name == "group":
        context = interface.group("group", if_flag="enabled")
    elif context_name == "condition":
        context = interface.if_flag("enabled")
    else:
        context = interface.resources()

    with pytest.raises(RuntimeError, match="sentinel"):
        with context:
            raise RuntimeError("sentinel")

    om.prog_gen.end_flow(child_refs)
    om.prog_gen.end_flow(outer_refs)
