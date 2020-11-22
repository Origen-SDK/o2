with Flow() as flow:
    # Added set-flag to test that manually set flags are brought up to the top-level for SMT8
    flow.func("margin_read1_all1",
              number=flow.options["number"],
              on_fail={"set_flag": "ers_vfy_failed"})
