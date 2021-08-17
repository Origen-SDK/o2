with Flow() as flow:
    if flow.options.get("force"):
        flow.func("erase_all", number=flow.options["number"])
    else:
        with flow.if_enable('additional_erase'):
            flow.func("erase_all", number=flow.options["number"])
