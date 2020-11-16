with Flow() as flow:
    with flow.if_enable('additional_erase', force=flow.options.get("force")):
        flow.func("erase_all", number=flow.options["number"])