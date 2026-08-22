use super::super::src_caller_meta;
use crate::prog_gen::{Group, PatternGroup, Test, TestInvocation};
use indexmap::IndexMap;
use origen_metal::prog_gen::{
    flow_api, GroupType, IGXLResourceKind, ParamValue, PatternGroupType, SupportedTester,
};
use origen_metal::{Error, Result};
use pyo3::types::{PyDict, PyTuple};
use pyo3::{exceptions, prelude::*};
use std::str::FromStr;

#[pyclass]
#[derive(Debug, Clone)]
pub struct Patset {
    pub name: String,
    pub tester: SupportedTester,
    pub id: usize,
}

#[pyclass(subclass)]
#[derive(Debug)]
pub struct IGXL {
    tester: SupportedTester,
}

#[pymethods]
impl IGXL {
    #[new]
    pub fn new(tester: Option<String>) -> PyResult<Self> {
        Ok(IGXL {
            tester: match &tester {
                None => SupportedTester::IGXL,
                Some(t) => {
                    let t = t.to_uppercase().replace("_", "");
                    match t.as_str() {
                        "IGXL" => SupportedTester::IGXL,
                        "J750" => SupportedTester::J750,
                        "ULTRAFLEX" => SupportedTester::ULTRAFLEX,
                        _ => {
                            return Err(PyErr::new::<exceptions::PyRuntimeError, _>(format!(
                                "IGXL tester must be 'J750' or 'ULTRAFLEX', '{}' is not supported",
                                t
                            )))
                        }
                    }
                }
            },
        })
    }

    #[pyo3(signature=(name, template, library=None, allow_missing=false, **kwargs))]
    fn new_test_instance(
        &mut self,
        name: String,
        template: String,
        library: Option<String>,
        allow_missing: bool,
        kwargs: Option<&PyDict>,
    ) -> PyResult<Test> {
        let library = match library {
            Some(x) => x,
            None => "std".to_string(),
        };

        let t = Test::new(
            name.clone(),
            self.tester,
            library,
            template,
            allow_missing,
            kwargs,
        )?;

        t._set_attr("test_name", Some(ParamValue::String(name)), allow_missing)?;

        Ok(t)
    }

    #[pyo3(signature=(name, proc_name, allow_missing=false, **kwargs))]
    fn new_custom_test_instance(
        &mut self,
        name: String,
        proc_name: String,
        allow_missing: bool,
        kwargs: Option<&PyDict>,
    ) -> PyResult<Test> {
        let test = Test::new(
            name.clone(),
            self.tester,
            "std".to_string(),
            "custom".to_string(),
            allow_missing,
            kwargs,
        )?;
        test._set_attr("test_name", Some(ParamValue::String(name)), allow_missing)?;
        test._set_attr(
            "proc_name",
            Some(ParamValue::String(proc_name)),
            allow_missing,
        )?;
        Ok(test)
    }

    #[pyo3(signature=(name, allow_missing=false, **kwargs))]
    fn functional(
        &mut self,
        name: String,
        allow_missing: bool,
        kwargs: Option<&PyDict>,
    ) -> PyResult<Test> {
        self.new_test_instance(name, "functional".to_string(), None, allow_missing, kwargs)
    }

    #[pyo3(signature=(name, allow_missing=false, **kwargs))]
    fn empty(
        &mut self,
        name: String,
        allow_missing: bool,
        kwargs: Option<&PyDict>,
    ) -> PyResult<Test> {
        self.new_test_instance(name, "empty".to_string(), None, allow_missing, kwargs)
    }

    #[pyo3(signature=(name, allow_missing=false, **kwargs))]
    fn other(
        &mut self,
        name: String,
        allow_missing: bool,
        kwargs: Option<&PyDict>,
    ) -> PyResult<Test> {
        self.new_test_instance(name, "other".to_string(), None, allow_missing, kwargs)
    }

    #[pyo3(signature=(name, allow_missing=false, **kwargs))]
    fn ppmu(
        &mut self,
        name: String,
        allow_missing: bool,
        kwargs: Option<&PyDict>,
    ) -> PyResult<Test> {
        self.new_test_instance(name, "pin_pmu".to_string(), None, allow_missing, kwargs)
    }

    #[pyo3(signature=(name, allow_missing=false, **kwargs))]
    fn pin_pmu(
        &mut self,
        name: String,
        allow_missing: bool,
        kwargs: Option<&PyDict>,
    ) -> PyResult<Test> {
        self.ppmu(name, allow_missing, kwargs)
    }

    #[pyo3(signature=(name, allow_missing=false, **kwargs))]
    fn dcvi_powersupply(
        &mut self,
        name: String,
        allow_missing: bool,
        kwargs: Option<&PyDict>,
    ) -> PyResult<Test> {
        self.new_test_instance(
            name,
            "dcvi_powersupply".to_string(),
            None,
            allow_missing,
            kwargs,
        )
    }

    #[pyo3(signature=(allow_missing=false, **kwargs))]
    pub fn new_flow_line(
        &mut self,
        allow_missing: bool,
        kwargs: Option<&PyDict>,
    ) -> PyResult<TestInvocation> {
        let t = TestInvocation::new("_".to_owned(), self.tester, allow_missing, kwargs)?;
        Ok(t)
    }

    #[pyo3(signature=(name, pattern=None, patterns=None))]
    fn new_patset(
        &mut self,
        name: String,
        pattern: Option<&PyAny>,
        patterns: Option<&PyAny>,
    ) -> PyResult<PatternGroup> {
        let pg = PatternGroup::new(name, self.tester, Some(PatternGroupType::Patset))?;
        if let Some(p) = pattern {
            for pat in extract_vec_string("pattern", p)? {
                pg.append(pat, None)?;
            }
        }
        if let Some(p) = patterns {
            for pat in extract_vec_string("patterns", p)? {
                pg.append(pat, None)?;
            }
        }
        Ok(pg)
    }

    #[pyo3(signature=(name, pattern=None, patterns=None))]
    fn new_patgroup(
        &mut self,
        name: String,
        pattern: Option<&PyAny>,
        patterns: Option<&PyAny>,
    ) -> PyResult<PatternGroup> {
        Ok(self.new_pattern_group(name, PatternGroupType::Patgroup, pattern, patterns)?)
    }

    #[pyo3(signature=(name, pattern=None, patterns=None))]
    fn new_patsubr(
        &mut self,
        name: String,
        pattern: Option<&PyAny>,
        patterns: Option<&PyAny>,
    ) -> PyResult<PatternGroup> {
        Ok(self.new_pattern_group(name, PatternGroupType::Patsubr, pattern, patterns)?)
    }

    // Set the cpu wait flags for the given test instance
    #[pyo3(signature=(test_instance, *flags))]
    fn set_wait_flags(&mut self, test_instance: &Test, flags: &PyTuple) -> PyResult<()> {
        let mut clean_flags: Vec<String> = vec![];
        for fl in flags {
            let mut bad = true;
            if let Ok(f) = fl.extract::<String>() {
                match f.to_lowercase().as_str() {
                    "a" | "b" | "c" | "d" => {
                        clean_flags.push(f.to_lowercase().to_owned());
                        bad = false;
                    }
                    _ => {}
                }
            }
            if bad {
                return Err(PyErr::from(Error::new(&format!(
                "Illegal argument given to set_wait_flags '{}', should be a String flag name, e.g. \"a\", \"b\", etc.",
                fl
            ))));
            }
        }
        flow_api::set_wait_flags(test_instance.id, clean_flags, src_caller_meta())?;
        Ok(())
    }

    fn test_instance_group(&mut self, name: String) -> PyResult<Group> {
        let g = Group::new(name, Some(self.tester), GroupType::Test, None);
        Ok(g)
    }

    fn set_resource_filename(&mut self, kind: String, name: String) -> PyResult<()> {
        let kind = IGXLResourceKind::from_str(&kind).map_err(PyErr::from)?;
        flow_api::set_igxl_resources_filename(kind, name, src_caller_meta())?;
        Ok(())
    }

    #[pyo3(signature=(path, comment=None))]
    fn add_reference(&mut self, path: String, comment: Option<String>) -> PyResult<()> {
        let mut values = IndexMap::new();
        values.insert("comment".to_string(), vec![comment.unwrap_or_default()]);
        flow_api::define_igxl_resource("references".to_string(), path, values, src_caller_meta())?;
        Ok(())
    }

    #[pyo3(signature=(name, **kwargs))]
    fn new_job(&mut self, name: String, kwargs: Option<&PyDict>) -> PyResult<()> {
        let mut values = IndexMap::new();
        if let Some(kwargs) = kwargs {
            for (key, value) in kwargs {
                let key = key.extract::<String>()?;
                let value = if let Ok(items) = value.extract::<Vec<String>>() {
                    items
                } else if let Ok(item) = value.extract::<String>() {
                    vec![item]
                } else {
                    return Err(PyErr::new::<exceptions::PyTypeError, _>(format!(
                        "UltraFLEX job attribute '{}' must be a String or list of Strings",
                        key
                    )));
                };
                values.insert(key, value);
            }
        }
        flow_api::define_igxl_resource("jobs".to_string(), name, values, src_caller_meta())?;
        Ok(())
    }

    #[pyo3(signature=(symbol, value, job="".to_string(), comment="".to_string()))]
    fn add_global_spec(
        &mut self,
        symbol: String,
        value: &PyAny,
        job: String,
        comment: String,
    ) -> PyResult<()> {
        let value = Self::format_spec_value("global_specs", &symbol, Some(value))?;
        let values = IndexMap::from([
            ("value".to_string(), vec![value]),
            ("job".to_string(), vec![job]),
            ("comment".to_string(), vec![comment]),
        ]);
        flow_api::define_igxl_resource(
            "global_specs".to_string(),
            symbol,
            values,
            src_caller_meta(),
        )?;
        Ok(())
    }

    #[pyo3(signature=(symbol, specset, selector="nom".to_string(), typ=None, min=None, max=None, comment="".to_string()))]
    fn add_ac_spec(
        &mut self,
        symbol: String,
        specset: String,
        selector: String,
        typ: Option<&PyAny>,
        min: Option<&PyAny>,
        max: Option<&PyAny>,
        comment: String,
    ) -> PyResult<()> {
        self.add_spec(
            "ac_specs", symbol, specset, selector, typ, min, max, comment,
        )
    }

    #[pyo3(signature=(symbol, specset, selector="nom".to_string(), typ=None, min=None, max=None, comment="".to_string()))]
    fn add_dc_spec(
        &mut self,
        symbol: String,
        specset: String,
        selector: String,
        typ: Option<&PyAny>,
        min: Option<&PyAny>,
        max: Option<&PyAny>,
        comment: String,
    ) -> PyResult<()> {
        self.add_spec(
            "dc_specs", symbol, specset, selector, typ, min, max, comment,
        )
    }

    #[pyo3(signature=(name, pin_type="I/O".to_string(), comment="".to_string()))]
    fn add_pin(&mut self, name: String, pin_type: String, comment: String) -> PyResult<()> {
        self.add_pin_resource("pin", name, None, pin_type, comment)
    }

    #[pyo3(signature=(name, comment="".to_string()))]
    fn add_power_pin(&mut self, name: String, comment: String) -> PyResult<()> {
        self.add_pin_resource("power", name, None, "Power".to_string(), comment)
    }

    #[pyo3(signature=(name, pin_type="Utility".to_string(), comment="".to_string()))]
    fn add_utility_pin(&mut self, name: String, pin_type: String, comment: String) -> PyResult<()> {
        self.add_pin_resource("utility", name, None, pin_type, comment)
    }

    #[pyo3(signature=(group, pin, pin_type="I/O".to_string(), comment="".to_string()))]
    fn add_group_pin(
        &mut self,
        group: String,
        pin: String,
        pin_type: String,
        comment: String,
    ) -> PyResult<()> {
        self.add_pin_resource("group", pin, Some(group), pin_type, comment)
    }

    #[pyo3(signature=(pin, parameter, value, comment="".to_string()))]
    fn add_level(
        &mut self,
        pin: String,
        parameter: String,
        value: String,
        comment: String,
    ) -> PyResult<()> {
        let values = IndexMap::from([
            ("parameter".to_string(), vec![parameter]),
            ("value".to_string(), vec![value]),
            ("comment".to_string(), vec![comment]),
        ]);
        flow_api::define_igxl_resource("levels".to_string(), pin, values, src_caller_meta())?;
        Ok(())
    }

    #[pyo3(signature=(name, pin, src="PAT".to_string(), format="NR".to_string(), drive_on="".to_string(), drive_data="".to_string(), drive_return="".to_string(), drive_off="".to_string(), compare_mode="Edge".to_string(), compare_open="".to_string(), compare_close="".to_string(), resolution="".to_string(), timing_mode="Machine".to_string(), comment="".to_string()))]
    fn add_edgeset(
        &mut self,
        name: String,
        pin: String,
        src: String,
        format: String,
        drive_on: String,
        drive_data: String,
        drive_return: String,
        drive_off: String,
        compare_mode: String,
        compare_open: String,
        compare_close: String,
        resolution: String,
        timing_mode: String,
        comment: String,
    ) -> PyResult<()> {
        let values = IndexMap::from([
            ("edgeset".to_string(), vec![name]),
            ("src".to_string(), vec![src]),
            ("format".to_string(), vec![format]),
            ("drive_on".to_string(), vec![drive_on]),
            ("drive_data".to_string(), vec![drive_data]),
            ("drive_return".to_string(), vec![drive_return]),
            ("drive_off".to_string(), vec![drive_off]),
            ("compare_mode".to_string(), vec![compare_mode]),
            ("compare_open".to_string(), vec![compare_open]),
            ("compare_close".to_string(), vec![compare_close]),
            ("resolution".to_string(), vec![resolution]),
            ("timing_mode".to_string(), vec![timing_mode]),
            ("comment".to_string(), vec![comment]),
        ]);
        flow_api::define_igxl_resource("edgesets".to_string(), pin, values, src_caller_meta())?;
        Ok(())
    }

    #[pyo3(signature=(name, period, pin, edgeset, clock_period="".to_string(), setup="i/o".to_string(), timing_mode="Machine".to_string(), comment="".to_string()))]
    fn add_timeset(
        &mut self,
        name: String,
        period: String,
        pin: String,
        edgeset: String,
        clock_period: String,
        setup: String,
        timing_mode: String,
        comment: String,
    ) -> PyResult<()> {
        let values = IndexMap::from([
            ("period".to_string(), vec![period]),
            ("pin".to_string(), vec![pin]),
            ("edgeset".to_string(), vec![edgeset]),
            ("clock_period".to_string(), vec![clock_period]),
            ("setup".to_string(), vec![setup]),
            ("timing_mode".to_string(), vec![timing_mode]),
            ("comment".to_string(), vec![comment]),
        ]);
        flow_api::define_igxl_resource("timesets".to_string(), name, values, src_caller_meta())?;
        Ok(())
    }

    #[pyo3(signature=(name, period, pin, clock_period="".to_string(), setup="i/o".to_string(), src="PAT".to_string(), format="NR".to_string(), drive_on="".to_string(), drive_data="".to_string(), drive_return="".to_string(), drive_off="".to_string(), compare_mode="Edge".to_string(), compare_open="".to_string(), compare_close="".to_string(), resolution="".to_string(), timing_mode="Machine".to_string(), comment="".to_string()))]
    fn add_timeset_basic(
        &mut self,
        name: String,
        period: String,
        pin: String,
        clock_period: String,
        setup: String,
        src: String,
        format: String,
        drive_on: String,
        drive_data: String,
        drive_return: String,
        drive_off: String,
        compare_mode: String,
        compare_open: String,
        compare_close: String,
        resolution: String,
        timing_mode: String,
        comment: String,
    ) -> PyResult<()> {
        let values = IndexMap::from([
            ("period".to_string(), vec![period]),
            ("pin".to_string(), vec![pin]),
            ("clock_period".to_string(), vec![clock_period]),
            ("setup".to_string(), vec![setup]),
            ("src".to_string(), vec![src]),
            ("format".to_string(), vec![format]),
            ("drive_on".to_string(), vec![drive_on]),
            ("drive_data".to_string(), vec![drive_data]),
            ("drive_return".to_string(), vec![drive_return]),
            ("drive_off".to_string(), vec![drive_off]),
            ("compare_mode".to_string(), vec![compare_mode]),
            ("compare_open".to_string(), vec![compare_open]),
            ("compare_close".to_string(), vec![compare_close]),
            ("resolution".to_string(), vec![resolution]),
            ("timing_mode".to_string(), vec![timing_mode]),
            ("comment".to_string(), vec![comment]),
        ]);
        flow_api::define_igxl_resource(
            "timesets_basic".to_string(),
            name,
            values,
            src_caller_meta(),
        )?;
        Ok(())
    }
}

impl IGXL {
    fn add_pin_resource(
        &mut self,
        kind: &str,
        name: String,
        group: Option<String>,
        pin_type: String,
        comment: String,
    ) -> PyResult<()> {
        let values = IndexMap::from([
            ("kind".to_string(), vec![kind.to_string()]),
            ("group".to_string(), vec![group.unwrap_or_default()]),
            ("type".to_string(), vec![pin_type]),
            ("comment".to_string(), vec![comment]),
        ]);
        flow_api::define_igxl_resource("pinmap".to_string(), name, values, src_caller_meta())?;
        Ok(())
    }

    fn add_spec(
        &mut self,
        kind: &str,
        symbol: String,
        specset: String,
        selector: String,
        typ: Option<&PyAny>,
        min: Option<&PyAny>,
        max: Option<&PyAny>,
        comment: String,
    ) -> PyResult<()> {
        let typ = Self::format_spec_value(kind, &symbol, typ)?;
        let min = Self::format_spec_value(kind, &symbol, min)?;
        let max = Self::format_spec_value(kind, &symbol, max)?;
        let values = IndexMap::from([
            ("specset".to_string(), vec![specset]),
            ("selector".to_string(), vec![selector]),
            ("typ".to_string(), vec![typ]),
            ("min".to_string(), vec![min]),
            ("max".to_string(), vec![max]),
            ("comment".to_string(), vec![comment]),
        ]);
        flow_api::define_igxl_resource(kind.to_string(), symbol, values, src_caller_meta())?;
        Ok(())
    }

    fn format_spec_value(kind: &str, symbol: &str, value: Option<&PyAny>) -> PyResult<String> {
        let Some(value) = value else {
            return Ok(String::new());
        };
        if let Ok(value) = value.extract::<String>() {
            return Ok(value);
        }
        let value = value.extract::<f64>().map_err(|_| {
            PyErr::new::<exceptions::PyTypeError, _>(
                "UltraFLEX spec values must be numeric, String, or None",
            )
        })?;
        if value == 0.0 {
            return Ok("0".to_string());
        }
        let abs = value.abs();
        let (scaled, unit) = if kind == "ac_specs" {
            if abs < 1e-9 {
                (value * 1e12, "ps")
            } else if abs < 1e-6 {
                (value * 1e9, "ns")
            } else if abs < 1e-3 {
                (value * 1e6, "us")
            } else if abs < 1.0 {
                (value * 1e3, "ms")
            } else {
                (value, "")
            }
        } else {
            let lower = symbol.to_ascii_lowercase();
            let voltage = if kind == "global_specs" {
                lower.contains("vgb_")
            } else {
                ["voh", "vol", "vt", "vcl", "vch", "vdd"]
                    .iter()
                    .any(|token| lower.contains(token))
            };
            let current =
                kind == "dc_specs" && ["ioh", "iol"].iter().any(|token| lower.contains(token));
            if voltage {
                if abs < 1e-6 {
                    (value * 1e9, "nV")
                } else if abs < 1e-3 {
                    (value * 1e6, "uV")
                } else if kind == "global_specs" && abs < 1.0 {
                    (value * 1e3, "mV")
                } else {
                    (value, "V")
                }
            } else if current {
                if abs < 1e-6 {
                    (value * 1e9, "nA")
                } else if abs < 1e-3 {
                    (value * 1e6, "uA")
                } else if abs < 1.0 {
                    (value * 1e3, "mA")
                } else {
                    (value, "A")
                }
            } else {
                (value, "")
            }
        };
        let scaled = (scaled * 10_000.0).round() / 10_000.0;
        Ok(if unit.is_empty() {
            format!("={}", scaled)
        } else {
            format!("={}*{}", scaled, unit)
        })
    }

    fn new_pattern_group(
        &mut self,
        name: String,
        kind: PatternGroupType,
        pattern: Option<&PyAny>,
        patterns: Option<&PyAny>,
    ) -> Result<PatternGroup> {
        let group = PatternGroup::new(name, self.tester, Some(kind))?;
        if let Some(pattern) = pattern {
            for path in extract_vec_string("pattern", pattern)? {
                group.append(path, None)?;
            }
        }
        if let Some(patterns) = patterns {
            for path in extract_vec_string("patterns", patterns)? {
                group.append(path, None)?;
            }
        }
        Ok(group)
    }
}

fn extract_vec_string(arg_name: &str, val: &PyAny) -> Result<Vec<String>> {
    if let Ok(v) = val.extract::<String>() {
        Ok(vec![v])
    } else if let Ok(v) = val.extract::<Vec<String>>() {
        Ok(v)
    } else {
        bail!(
            "Illegal value for argument '{}', expected a String or a List of Strings, got: {}",
            arg_name,
            val
        )
    }
}
