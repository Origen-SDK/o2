#Flow.create interface: 'OrigenTesters::Test::Interface', flow_description: 'Probe1 Main' do
with Flow(flow_description="Probe1 Main") as flow:

    #unless Origen.app.environment.name == 'v93k_global'
    flow.resources_filename = 'prb1'

    flow.include('components/prb1_main')

    flow.include(
        'test'
    )  # import top-level test.rb directly, note that Flow.create options of sub-flow will be ignored!

    # Test that a reference to a deeply nested test works (mainly for SMT8)
    flow.add_test("on_deep_1",
                  if_failed="deep_test",
                  test_text="some_custom_text")

    flow.pass_bin(1, description="Good die!", softbin=1)

    # Print out the AST in developement
    import _origen
    _origen.flow()
