"""Test-program generation helpers."""


def set_flow_visualization(enabled=True):
    """Enable or disable interactive flow visualization artifacts.

    When enabled, program generation writes ``*.flow.json`` and
    ``*.flow.html`` files under the tester output's ``flow_visualizations``
    directory. The HTML is self-contained and test nodes open their effective
    test-method parameter library.
    """
    from origen_metal import _origen_metal

    _origen_metal.prog_gen.set_flow_visualization(enabled)
