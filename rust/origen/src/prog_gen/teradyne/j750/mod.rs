use super::super::model::ParamValue;
use super::TestProgram;
use crate::Result;

// This is called at the start of a test program generation run to define the J750 test templates,
// it will be called after base::define_test_lib so any tests defined in there can be
// referred to as parents of these tests.
pub fn define_test_lib(prog: &mut TestProgram) -> Result<()> {
    prog.add_test_library("j750")?;

    let base_test_id = prog.get_test_template_id("igxl", "test_instance")?;

    prog.add_test_template("j750", "other", |t, prog| {
        t.parent_id = Some(base_test_id);

        t.set("proc_type", ParamValue::String("Other".to_string()), prog)?;
        t.set(
            "proc_called_as",
            ParamValue::String("Excel Macro".to_string()),
            prog,
        )?;

        Ok(())
    })?;

    prog.add_test_template("j750", "empty", |t, prog| {
        t.parent_id = Some(base_test_id);

        t.add_alias("start_func", "arg0", prog)?;
        t.add_alias("start_of_body_f", "arg0", prog)?;
        t.add_alias("pre_pat_func", "arg1", prog)?;
        t.add_alias("pre_pat_f", "arg1", prog)?;
        t.add_alias("pre_test_func", "arg2", prog)?;
        t.add_alias("pre_test_f", "arg2", prog)?;
        t.add_alias("post_test_func", "arg3", prog)?;
        t.add_alias("post_test_f", "arg3", prog)?;
        t.add_alias("post_pat_func", "arg4", prog)?;
        t.add_alias("post_pat_f", "arg4", prog)?;
        t.add_alias("end_func", "arg5", prog)?;
        t.add_alias("end_of_body_f", "arg5", prog)?;
        t.add_alias("start_func_args", "arg6", prog)?;
        t.add_alias("start_of_body_f_args", "arg6", prog)?;
        t.add_alias("pre_pat_func_args", "arg7", prog)?;
        t.add_alias("pre_pat_f_args", "arg7", prog)?;
        t.add_alias("pre_test_func_args", "arg8", prog)?;
        t.add_alias("pre_test_f_args", "arg8", prog)?;
        t.add_alias("post_test_func_args", "arg9", prog)?;
        t.add_alias("post_test_f_args", "arg9", prog)?;
        t.add_alias("post_pat_func_args", "arg10", prog)?;
        t.add_alias("post_pat_f_args", "arg10", prog)?;
        t.add_alias("end_func_args", "arg11", prog)?;
        t.add_alias("end_of_body_f_args", "arg11", prog)?;
        t.add_alias("utility_pins_1", "arg12", prog)?;
        t.add_alias("utility_pins_0", "arg13", prog)?;
        t.add_alias("init_lo", "arg14", prog)?;
        t.add_alias("start_lo", "arg14", prog)?;
        t.add_alias("init_hi", "arg15", prog)?;
        t.add_alias("start_hi", "arg15", prog)?;
        t.add_alias("init_hiz", "arg16", prog)?;
        t.add_alias("start_hiz", "arg16", prog)?;
        t.add_alias("float_pins", "arg17", prog)?;

        t.set(
            "proc_type",
            ParamValue::String("IG-XL Template".to_string()),
            prog,
        )?;
        t.set("proc_name", ParamValue::String("Empty_T".to_string()), prog)?;
        t.set(
            "proc_called_as",
            ParamValue::String("Excel Macro".to_string()),
            prog,
        )?;

        Ok(())
    })?;

    prog.add_test_template("j750", "functional", |t, prog| {
        t.parent_id = Some(base_test_id);

        t.add_alias("pattern", "arg0", prog)?;
        t.add_alias("patterns", "arg0", prog)?;
        t.add_alias("start_func", "arg1", prog)?;
        t.add_alias("start_of_body_f", "arg1", prog)?;
        t.add_alias("pre_pat_func", "arg2", prog)?;
        t.add_alias("pre_pat_f", "arg2", prog)?;
        t.add_alias("pre_test_func", "arg3", prog)?;
        t.add_alias("pre_test_f", "arg3", prog)?;
        t.add_alias("post_test_func", "arg4", prog)?;
        t.add_alias("post_test_f", "arg4", prog)?;
        t.add_alias("post_pat_func", "arg5", prog)?;
        t.add_alias("post_pat_f", "arg5", prog)?;
        t.add_alias("end_func", "arg6", prog)?;
        t.add_alias("end_of_body_f", "arg6", prog)?;
        t.add_alias("set_pass_fail", "arg7", prog)?;
        t.add_alias("init_lo", "arg8", prog)?;
        t.add_alias("start_lo", "arg8", prog)?;
        t.add_alias("init_hi", "arg9", prog)?;
        t.add_alias("start_hi", "arg9", prog)?;
        t.add_alias("init_hiz", "arg10", prog)?;
        t.add_alias("start_hiz", "arg10", prog)?;
        t.add_alias("float_pins", "arg11", prog)?;
        t.add_alias("start_func_args", "arg13", prog)?;
        t.add_alias("start_of_body_f_args", "arg13", prog)?;
        t.add_alias("pre_pat_func_args", "arg14", prog)?;
        t.add_alias("pre_pat_f_args", "arg14", prog)?;
        t.add_alias("pre_test_func_args", "arg15", prog)?;
        t.add_alias("pre_test_f_args", "arg15", prog)?;
        t.add_alias("post_test_func_args", "arg16", prog)?;
        t.add_alias("post_test_f_args", "arg16", prog)?;
        t.add_alias("post_pat_func_args", "arg17", prog)?;
        t.add_alias("post_pat_f_args", "arg17", prog)?;
        t.add_alias("end_func_args", "arg18", prog)?;
        t.add_alias("end_of_body_f_args", "arg18", prog)?;
        t.add_alias("utility_pins_1", "arg19", prog)?;
        t.add_alias("utility_pins_0", "arg20", prog)?;
        t.add_alias("wait_flags", "arg21", prog)?;
        t.add_alias("wait_time", "arg22", prog)?;
        t.add_alias("pattern_timeout", "arg22", prog)?;
        t.add_alias("pat_flag_func", "arg23", prog)?;
        t.add_alias("pat_flag_f", "arg23", prog)?;
        t.add_alias("PatFlagF", "arg23", prog)?;
        t.add_alias("pat_flag_func_args", "arg24", prog)?;
        t.add_alias("pat_flag_f_args", "arg24", prog)?;
        t.add_alias("relay_mode", "arg25", prog)?;
        t.add_alias("threading", "arg26", prog)?;
        t.add_alias("match_all_sites", "arg27", prog)?;
        t.add_alias("capture_mode", "arg30", prog)?;
        t.add_alias("capture_what", "arg31", prog)?;
        t.add_alias("capture_memory", "arg32", prog)?;
        t.add_alias("capture_size", "arg33", prog)?;
        t.add_alias("datalog_mode", "arg34", prog)?;
        t.add_alias("data_type", "arg35", prog)?;

        t.set(
            "proc_type",
            ParamValue::String("IG-XL Template".to_string()),
            prog,
        )?;
        t.set(
            "proc_name",
            ParamValue::String("Functional_T".to_string()),
            prog,
        )?;
        t.set(
            "proc_called_as",
            ParamValue::String("VB DLL".to_string()),
            prog,
        )?;
        t.set("set_pass_fail", ParamValue::UInt(1), prog)?;

        //set_pass_fail:   1,
        //wait_flags:      'XXXX',
        //wait_time:       30,
        //relay_mode:      1,
        //threading:       0,
        //match_all_sites: 0,
        //capture_mode:    0,
        //capture_what:    0,
        //capture_memory:  0,
        //capture_size:    256,
        //datalog_mode:    0,
        //data_type:       0

        Ok(())
    })?;

    Ok(())
}
