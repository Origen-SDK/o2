with Flow() as flow:

    if flow.top_level_options["environment"] == "probe":
        flow.func("pgm_ckbd", number=flow.options["number"] + 10)
        flow.func("mrd_ckbd", number=flow.options["number"] + 20)

    flow.include_additional_prb2_test = True
