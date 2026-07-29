with Flow(flow_description="Probe1 Main", namespace="OrigenTesters") as flow:

    #unless Origen.app.environment.name == 'v93k_global'
    flow.set_resources_filename('prb1')

    flow.include('components/prb1_main')

    flow.include(
        'test'
    )  # import top-level test.rb directly, note that Flow.create options of sub-flow will be ignored!

    # Test that a reference to a deeply nested test works (mainly for SMT8)
    flow.add_test("on_deep_1",
                  if_failed="deep_test",
                  test_text="some_custom_text",
                  id="deep_test_1")

    with tester().neq("v93ksmt8"):
        with flow.if_failed("deep_test_1"):
            flow.render_str("multi_bin;")

    flow.good_die(1, description="Good die!", softbin=1)

    with flow.resources():
        flow.include('prb1_resources')
