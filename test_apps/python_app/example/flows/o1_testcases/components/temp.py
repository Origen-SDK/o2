with Flow() as flow:
    flow.meas("bgap_voltage_meas",
              tnum=1050,
              bin=119,
              soft_bin=2,
              hi_limit=45,
              number=flow.options["number"] + 10)
    flow.meas("bgap_voltage_meas1", number=flow.options["number"] + 20)
