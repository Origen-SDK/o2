with Flow() as flow:
    options = {"number": 2}
    options.update(flow.options)

    for x in range(options["number"]):
        flow.func(f"bitcell_iv_{x}")
