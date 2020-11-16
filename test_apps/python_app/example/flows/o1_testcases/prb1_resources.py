# Similar to the test flows an interface instance is passed in as the first argument.
with Flow() as flow:

    flow.resources_filename = 'prb1'

    # Logic here should be minimal,
    # pass whatever options you want
    # but the recommended approach is
    # to infer the pattern name and as
    # many additional details as
    # possible from the test name
    flow.func("program_ckbd", duration="dynamic")

    flow.include('efa_resources')

    flow.func("margin_read1_ckbd")
    flow.func("normal_read_ckbd")
    flow.func("margin_read0_ckbd")

    flow.func("erase_all", duration="dynamic")

    flow.para('charge_pump', high_voltage=True)

    # TODO: Add when doing J750
    #with tester().eq("j750") as j750:
    #    j750.render_test_instances('templates/j750/vt_instances')
    #    # Don't think this should be supported from a flow source in O2, can be
    #    # easily handled by an app's build script
    #    #compile 'templates/j750/program_sheet.txt', :passed_param => true
