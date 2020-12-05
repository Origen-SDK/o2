from origen.unit_helpers import *

# An instance of the interface is
# passed in here, iterators and other
# argument passing will be supported
# similar to Pattern.create.
with Flow() as flow:
    flow.description = ''

    #if tester.v93k? && tester.create_limits_file
    #  flow.func("program_ckbd", bin=100, soft_bin=1100, number=40000)

    flow.meas("read_pump",
              tnum=1050,
              bin=119,
              soft_bin=2,
              lo_limit=35,
              number=40005)
    flow.meas("read_pump",
              tnum=1060,
              bin=119,
              soft_bin=2,
              hi_limit=45,
              number=40010)
    flow.meas("read_pump",
              tnum=1070,
              bin=119,
              soft_bin=2,
              hi_limit=45,
              lo_limit=35,
              number=40020)
    flow.meas("read_pump",
              tnum=1080,
              bin=119,
              soft_bin=2,
              hi_limit=45,
              lo_limit=35,
              number=40030)
    flow.meas("read_pump",
              tnum=1090,
              bin=119,
              soft_bin=2,
              hi_limit=45 * mV,
              lo_limit=35 * mV,
              number=40040)
    flow.meas("read_pump",
              tnum=1100,
              bin=119,
              soft_bin=2,
              hi_limit=45 * mV,
              lo_limit=35 * mV,
              continue_on_fail=True,
              number=40050)
    flow.meas("read_pump",
              tnum=1110,
              bin=119,
              soft_bin=2,
              hi_limit=2000,
              lo_limit=0.01,
              continue_on_fail=True,
              number=40060)

    #unless tester.v93k? && tester.create_limits_file
    flow.meas("read_pump",
              tnum=1120,
              bin=119,
              soft_bin=2,
              hi_limit="_some_spec",
              lo_limit=0.01,
              continue_on_fail=True,
              number=40070)
    flow.meas("read_pump",
              tnum=1130,
              bin=119,
              soft_bin=2,
              hi_limit=[1, 2],
              number=40080)
    flow.meas("read_pump",
              tnum=1140,
              bin=119,
              soft_bin=2,
              lo_limit=[1 * uA, 2 * uA, 3 * uA],
              hi_limit=[4 * uA, 5e-06],
              units="A",
              defer_limits=True,
              number=40090)

    # TODO: implement this when test case is imported
    #with tester().eq("uflex"):
    #  meas_multi_limits("bin_now", tnum=3000, bin=119, soft_bin=2)
    #  meas_multi_limits("bin_later", tnum=3000, bin=119, soft_bin=2, defer_limits=True)
    #  log("Test of ultraflex render API")
    #  line = flow.ultraflex.use_limit
    #  line.units = "Hz"
