//use super::super::model::ParamType;
//use super::super::TestProgram;
//use crate::Result;
//
//// This is called at the start of a test program generation run to define
//// all of the built-in Teradyne test templates
//pub fn define_test_lib(prog: &mut TestProgram) -> Result<()> {
//    prog.create_test_library("igxl")?;
//
//    prog.create_test_template("igxl", "test_instance", |t, prog| {
//        t.add_param(
//            "test_name",
//            ParamType::String,
//            None,
//            Some(vec!["name"]),
//            None,
//        )?;
//        t.add_param("proc_type", ParamType::String, None, None, None)?;
//        t.add_param("proc_name", ParamType::String, None, None, None)?;
//        t.add_param("proc_called_as", ParamType::String, None, None, None)?;
//        t.add_param("dc_category", ParamType::String, None, None, None)?;
//        t.add_param("dc_selector", ParamType::String, None, None, None)?;
//        t.add_param("ac_category", ParamType::String, None, None, None)?;
//        t.add_param("ac_selector", ParamType::String, None, None, None)?;
//        t.add_param(
//            "time_sets",
//            ParamType::String,
//            None,
//            Some(vec!["time_set", "timesets", "timeset"]),
//            None,
//        )?;
//        t.add_param(
//            "edge_sets",
//            ParamType::String,
//            None,
//            Some(vec!["edge_set", "edgesets", "edgeset"]),
//            None,
//        )?;
//        t.add_param("pin_levels", ParamType::String, None, None, None)?;
//        t.add_param("overlay", ParamType::String, None, None, None)?;
//        for i in 0..80 {
//            t.add_param(&format!("arg{}", i), ParamType::String, None, None, None)?;
//        }
//        Ok(())
//    })?;
//    Ok(())
//}
//
