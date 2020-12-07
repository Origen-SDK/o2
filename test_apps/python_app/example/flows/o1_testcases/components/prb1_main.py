# import pdb; pdb.set_trace()
with Flow() as flow:
    # Instantiate tests via the
    # interface
    flow.func('program_ckbd',
              tname='PGM_CKBD',
              tnum=1000,
              bin=100,
              soft_bin=1100)
    flow.func('margin_read1_ckbd', number=1010)

    # Control the build process based on
    # the current target
    if dut().has_margin0_bug:
        flow.func('normal_read_ckbd', number=1020)
    else:
        flow.func('margin_read0_ckbd', number=1030)

    ## Include a sub flow, example of
    # parameter passing
    flow.include('../erase', pulses=6, number=2000)

    # Render an ERB template, or raw
    # text file
    with tester().eq("j750"):
        flow.render('templates/j750/vt_flow.txt.j2', include_tifr=True)

    flow.log('Should be v1')
    flow.func("program_ckbd", number=3000)
    flow.log('Should be v2')
    flow.func("program_ckbd", duration="dynamic", number=3010)
    flow.log('Should be v1')
    flow.func("program_ckbd", number=3020)
    flow.log('Should be v2')
    flow.func("program_ckbd", duration="dynamic", number=3030)

    flow.log('Should be a v1 test instance group')
    flow.func("program_ckbd", by_block=True, number=3040)
    flow.log('Should be a v2 test instance group')
    flow.func("program_ckbd", by_block=True, duration="dynamic", number=3050)
    flow.log('Should be a v1 test instance group')
    flow.func("program_ckbd", by_block=True, number=3060)
    flow.log('Should be a v2 test instance group')
    flow.func("program_ckbd", by_block=True, duration="dynamic", number=3070)

    # Test job conditions
    flow.func("p1_only_test", if_job="p1", number=3080)
    with flow.if_job(["p1", "p2"]):
        flow.func("p1_or_p2_only_test", number=3090)
    flow.func("not_p1_test", unless_job="p1", number=3100)
    flow.func("not_p1_or_p2_test", unless_job=["p1", "p2"], number=3110)
    with flow.unless_job(["p1", "p2"]):
        flow.func("another_not_p1_or_p2_test", number=3120)

    flow.log('Verify that a test with an external instance works')
    flow.por(number=3130)

    # Not supporting the current context feature from O1, doesn't seem to be the best
    # thought out thing and it can be achieved easily enough with app-side code.
    # The flow has been modified here vs. the O1 version to produce the same output.
    flow.log('Verify that a request to use the current context works')
    with flow.if_job("p1"):
        flow.func("erase_all", number=3140)  # Job should be P1
        flow.func("erase_all", number=3150)  # Job should be P1
        flow.func("erase_all", number=3160)  # Job should be P1
    with flow.unless_job("p2"):
        flow.func("erase_all", number=3170)  # Job should be !P2

    # Deliver an initial erase pulse
    flow.func("erase_all", number=3180)

    # Deliver additional erase pulses as required until it verifies, maximum of 5 additional pulses
    number = 3200
    for x in range(5):
        # Assign a unique id attribute to each verify so that we know which one we are talking about when
        # making other tests dependent on it.
        # When Origen sees the if_failed dependency on a future test it will be smart enough to inhibit the binning
        # on this test without having to explicitly declare that.
        flow.func("margin_read1_all1", id=f"erase_vfy_{x}", number=number)
        number += 10
        # Run this test only if the given verify failed
        flow.func("erase_all", if_failed=f"erase_vfy_{x}", number=number)
        number += 10

    # A final verify to set the binning
    flow.func("margin_read1_all1", number=4000)

    flow.log('Test if enable')
    flow.func("erase_all", if_enable='do_erase', number=4010)

    with flow.if_enable('do_erase'):
        flow.func("erase_all", number=4020)

    flow.log('Test unless enable')
    flow.func("erase_all", unless_enable='no_extra_erase', number=4030)

    with flow.unless_enable('no_extra_erase'):
        flow.func("erase_all", number=4040)
        flow.func("erase_all", number=4050)

    flow.func("erase_all", number=4060)
    flow.func("erase_all", number=4070)

    flow.log('Test if_passed')
    flow.func("erase_all", id='erase_passed_1', number=4080)
    flow.func("erase_all", id='erase_passed_2', number=4090)

    flow.func("margin_read1_all1", if_passed='erase_passed_1', number=4100)
    with flow.if_passed('erase_passed_2'):
        flow.func("margin_read1_all1", number=4110)

    flow.log('Test unless_passed')
    flow.func("erase_all", id='erase_passed_3', number=4120)
    flow.func("erase_all", id='erase_passed_4', number=4130)

    flow.func("margin_read1_all1", unless_passed='erase_passed_3', number=4140)
    with flow.unless_passed('erase_passed_4'):
        flow.func("margin_read1_all1", number=4150)

    flow.log('Test if_failed')
    flow.func("erase_all", id='erase_failed_1', number=4160)
    flow.func("erase_all", id='erase_failed_2', number=4170)

    flow.func("margin_read1_all1", if_failed='erase_failed_1', number=4180)
    with flow.if_failed('erase_failed_2'):
        flow.func("margin_read1_all1", number=4190)

    flow.log('Test unless_failed')
    flow.func("erase_all", id='erase_failed_3', number=4200)
    flow.func("erase_all", id='erase_failed_4', number=4210)

    flow.func("margin_read1_all1", unless_failed='erase_failed_3', number=4220)
    with flow.unless_failed('erase_failed_4'):
        flow.func("margin_read1_all1", number=4230)

    flow.log('Test if_ran')
    flow.func("erase_all", id='erase_ran_1', number=4240)
    flow.func("erase_all", id='erase_ran_2', number=4250)

    flow.func("margin_read1_all1", if_ran='erase_ran_1', number=4260)
    with flow.if_ran('erase_ran_2'):
        flow.func("margin_read1_all1", number=4270)

    flow.log('Test unless_ran')
    flow.func("erase_all", id='erase_ran_3', number=4280)
    flow.func("erase_all", id='erase_ran_4', number=4290)

    flow.func("margin_read1_all1", unless_ran='erase_ran_3', number=4300)
    with flow.unless_ran('erase_ran_4'):
        flow.func("margin_read1_all1", number=4310)

    flow.log('Verify that job context wraps import')
    with flow.if_job("fr"):
        flow.include('../erase', number=5000)

    flow.log('Verify that job context wraps enable block within an import')
    with flow.if_job("fr"):
        flow.include('../additional_erase', number=5500)
        flow.include('../additional_erase', force=True, number=5600)

    flow.log('Verify that flow.cz works...')
    flow.func("margin_read1_all1",
              pin_levels="cz",
              cz_setup='vbplus_sweep',
              number=5700)

    flow.log('Verify that flow.cz works with enable words')
    with flow.if_enable('usb_xcvr_cz'):
        flow.func("xcvr_fs_vilvih", cz_setup='usb_fs_vil_cz', number=5710)
        flow.func("xcvr_fs_vilvih", cz_setup='usb_fs_vih_cz', number=5720)

    flow.func("xcvr_fs_vilvih",
              cz_setup='usb_fs_vil_cz',
              if_enable='usb_xcvr_cz',
              number=5730)
    flow.func("xcvr_fs_vilvih",
              cz_setup='usb_fs_vih_cz',
              if_enable='usb_xcvr_cz',
              number=5740)

    flow.log('Verify that MTO template works...')
    flow.mto_memory("mto_read1_all1", number=5750)

    with tester().eq("uflex"):
        flow.log('import statement')
        flow.include('temp', number=5800)

        flow.log('direct call')

        flow.meas("bgap_voltage_meas",
                  tnum=1050,
                  bin=119,
                  soft_bin=2,
                  hi_limit=45,
                  number=5910)
        flow.meas("bgap_voltage_meas1", number=5920)

    with tester().eq("j750"):
        flow.meas("lo_voltage", tnum=1150, bin=95, soft_bin=5, number=5920)
        flow.meas("hi_voltage",
                  pins="hi_v",
                  tnum=1160,
                  bin=96,
                  soft_bin=6,
                  number=5930)
        flow.meas("ps_leakage",
                  pins="power",
                  tnum=1170,
                  bin=97,
                  soft_bin=6,
                  number=5940)

    flow.log('Speed binning example bug from video 5')
    with flow.group("200Mhz Tests", id="g200"):
        flow.add_test("test200_1", number=5950)
        flow.add_test("test200_2", number=5960)
        flow.add_test("test200_3", number=5970)

    with flow.group("100Mhz Tests", if_failed="g200", id="g100"):
        flow.add_test("test100_1", bin=5, number=5980)
        flow.add_test("test100_2", bin=5, number=5990)
        flow.add_test("test100_3", bin=5, number=6000)

    flow.good_die(2, if_ran="g100")

    flow.log('Test node optimization within an if_failed branch')
    flow.func("some_func_test", id="sft1", number=6010)

    with flow.if_failed("sft1"):
        flow.bin(10, if_flag="Alarm")
        flow.bin(11, unless_flag="Alarm")
        flow.bin(12, if_enable="AlarmEnabled")
        flow.bin(13, unless_enable="AlarmEnabled")

    for i in range(3):
        #flow.cc(f"cc test {i}")
        flow.func(f"cc_test_{i}", number=7000 + i)

    # Ensure that mixed flag types work
    with tester().eq("v93k_smt7"):
        flow.log('Passing test flags of mixed types works as expected')
        with flow.if_enable(["sym_flag", "$StringFLag"]):
            flow.func("mixed_flag_check")

    flow.include('deep_nested')

    with tester().eq("v93k_smt7"):
        flow.log('Passing test flags works as expected')
        flow.func("test_with_no_flags",
                  bypass=False,
                  output_on_pass=False,
                  output_on_fail=False,
                  value_on_pass=False,
                  value_on_fail=False,
                  per_pin_on_pass=False,
                  per_pin_on_fail=False,
                  number=6020)
        flow.func("test_with_flags",
                  bypass=True,
                  output_on_pass=True,
                  output_on_fail=True,
                  value_on_pass=True,
                  value_on_fail=True,
                  per_pin_on_pass=True,
                  per_pin_on_fail=True,
                  number=6030)

    with tester().eq("v93k_smt7"):
        flow.log('force_serial test method parameter can be programmed')
        flow.func("force_serial_true_test", force_serial=True)
        flow.func("force_serial_false_test", force_serial=False)
