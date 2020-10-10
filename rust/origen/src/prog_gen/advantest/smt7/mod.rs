//use super::super::model::{Constraint, ParamType, ParamValue};
//use super::TestProgram;
//use crate::Result;
//
//// This is called at the start of a test program generation run to define the J750 test templates,
//// it will be called after base::define_test_lib so any tests defined in there can be
//// referred to as parents of these tests.
//pub fn define_test_lib(prog: &mut TestProgram) -> Result<()> {
//    prog.add_test_library("smt7_dc_tml")?;
//
//    prog.add_test_template("smt7_dc_tml", "continuity", |t, prog| {
//        t.add_param(
//            "pinlist",
//            ParamType::String,
//            Some(ParamValue::String("@".to_string())),
//            None,
//            None,
//        )?;
//        t.add_param(
//            "test_current",
//            ParamType::Current,
//            Some(ParamValue::Current(10e-6)),
//            None,
//            None,
//        )?;
//        t.add_param(
//            "settling_time",
//            ParamType::Time,
//            Some(ParamValue::Time(1e-3)),
//            None,
//            None,
//        )?;
//        t.add_param(
//            "measurement_mode",
//            ParamType::String,
//            Some(ParamValue::String("PPMUpar".to_string())),
//            None,
//            Some(vec![Constraint::In(vec![
//                ParamValue::String("PPMUpar".to_string()),
//                ParamValue::String("ProgLoad".to_string()),
//            ])]),
//        )?;
//        t.add_param(
//            "polarity",
//            ParamType::String,
//            Some(ParamValue::String("SPOL".to_string())),
//            None,
//            Some(vec![Constraint::In(vec![
//                ParamValue::String("SPOL".to_string()),
//                ParamValue::String("BPOL".to_string()),
//            ])]),
//        )?;
//
//        //    precharge_to_zero_vol: [:string, 'ON', %w(ON OFF)],
//        //    test_name:             [:string, 'passVolt_mv'],
//        //    output:                [:string, 'None', %w(None ReportUI ShowFailOnly)]
//        //  },
//
//        Ok(())
//    })?;
//
//    Ok(())
//}
//
