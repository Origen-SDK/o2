# A sub flow is a flow like any other.
# Any arguments passed in when
# instantiating this flow will be available via flow.options
with Flow() as flow:
    # Define default options
    options = {
        "pulses": 4,
        "post_verify": True,
        "number": 0,
    }
    options.update(flow.options)

    number = options["number"]

    for i in range(options["pulses"]):
        flow.func("erase_all", number=number)
        number += (i + 1) * 10

    if options["post_verify"]:
        flow.include('erase_vfy', number=number)
