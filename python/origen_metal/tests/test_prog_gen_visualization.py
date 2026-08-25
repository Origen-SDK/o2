import origen_metal.prog_gen as prog_gen


def test_flow_visualization_can_be_enabled_and_disabled():
    prog_gen.set_flow_visualization(True)
    prog_gen.set_flow_visualization(False)
