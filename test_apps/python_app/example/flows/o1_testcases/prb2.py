with Flow(bypass_sub_flows=True,
          add_flow_enable="enabled",
          environment="probe") as flow:
    flow.name_override = 'prb2' # simple override to avoid uppercase
    flow.description = '''
      An example of creating an entire test program from a single source file
    '''
    #unless Origen.app.environment.name == 'v93k_global'
    flow.set_resources_filename('prb2')

    flow.func("erase_all", duration="dynamic", number=10000)

    flow.func("margin_read1_all1", number=10010)

    flow.func("erase_all", duration="dynamic", number=10020)
    flow.func("margin_read1_all1", number=10030)

    flow.include('components/prb2_main', number=11000)

    flow.func("erase_all", duration="dynamic", number=12000)
    flow.func("margin_read1_all1", id='erased_successfully', number=12010)

    # Check that an instance variable change in a sub-flow (prb2_main in this case)
    # is preserved back here in the main flow
    if flow.include_additional_prb2_test:
        with flow.if_enable('extra_tests'):
            flow.include('components/prb2_main', number=13000)

    flow.func("margin_read1_all1", number=14000)
