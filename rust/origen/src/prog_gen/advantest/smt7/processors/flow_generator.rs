use crate::generator::ast::*;
use crate::generator::processor::*;
use crate::prog_gen::{BinType, GroupType, Model, ParamType, Test};
use std::collections::{BTreeMap, HashMap};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Does the final writing of the flow AST to a SMT7 flow file
pub struct FlowGenerator<'a> {
    output_dir: PathBuf,
    file_path: Option<PathBuf>,
    flow_header: Vec<String>,
    flow_body: Vec<String>,
    flow_footer: Vec<String>,
    indent: usize,
    model: &'a Model,
    test_methods: BTreeMap<String, &'a Test>,
    test_suites: BTreeMap<String, &'a Test>,
    test_method_names: HashMap<usize, String>,
    sig: String,
}

pub fn run(ast: &Node, output_dir: &Path, model: &Model) -> Result<PathBuf> {
    let mut p = FlowGenerator {
        output_dir: output_dir.to_owned(),
        file_path: None,
        flow_header: vec![],
        flow_body: vec![],
        flow_footer: vec![],
        indent: 0,
        model: model,
        test_methods: BTreeMap::new(),
        test_suites: BTreeMap::new(),
        test_method_names: HashMap::new(),
        sig: "_864CE8F".to_string(),
    };

    for (i, t) in model.tests.values().enumerate() {
        let name = format!("tm_{}", i + 1);
        p.test_method_names.insert(t.id, name.clone());
        p.test_methods.insert(name, t);
    }

    for (_, t) in model.test_invocations.values().enumerate() {
        p.test_suites
            .insert(format!("{}{}", t.get("name")?.unwrap(), p.sig), t);
    }
    ast.process(&mut p)?;
    Ok(p.file_path.unwrap())
}

impl<'a> FlowGenerator<'a> {
    fn push_body(&mut self, line: &str) {
        let ind = {
            if self.indent > 2 {
                4 + (3 * (self.indent - 2))
            } else {
                2 * self.indent
            }
        };
        if line == "" {
            self.flow_body.push(line.to_string());
        } else {
            self.flow_body
                .push(format!("{:indent$}{}", "", line, indent = ind));
        }
    }
}

/// Returns true if the given parameter has a value and it is true, returns false
/// for None or Some(false)
fn is_true(test_suite: &Test, param: &str) -> Result<bool> {
    match test_suite.get(param) {
        Err(e) => error!("An error occurred with parameter '{}': {}", param, e),
        Ok(v) => {
            if let Some(v) = v {
                v.to_bool()
            } else {
                Ok(false)
            }
        }
    }
}

fn flags(test_suite: &Test) -> Result<Vec<&'static str>> {
    let mut flags: Vec<&str> = vec![];
    if is_true(test_suite, "bypass")? {
        flags.push("bypass");
    }
    if is_true(test_suite, "set_pass")? {
        flags.push("set_pass");
    }
    if is_true(test_suite, "set_fail")? {
        flags.push("set_fail");
    }
    if is_true(test_suite, "hold")? {
        flags.push("hold");
    }
    if is_true(test_suite, "hold_on_fail")? {
        flags.push("hold_on_fail");
    }
    if is_true(test_suite, "output_on_pass")? {
        flags.push("output_on_pass");
    }
    if is_true(test_suite, "output_on_fail")? {
        flags.push("output_on_fail");
    }
    if is_true(test_suite, "pass_value")? {
        flags.push("value_on_pass");
    }
    if is_true(test_suite, "fail_value")? {
        flags.push("value_on_fail");
    }
    if is_true(test_suite, "per_pin_on_pass")? {
        flags.push("per_pin_on_pass");
    }
    if is_true(test_suite, "per_pin_on_fail")? {
        flags.push("per_pin_on_fail");
    }
    if is_true(test_suite, "log_mixed_signal_waveform")? {
        flags.push("mx_waves_enable");
    }
    if is_true(test_suite, "fail_per_label")? {
        flags.push("fail_per_label");
    }
    if is_true(test_suite, "ffc_enable")? {
        flags.push("ffc_enable");
    }
    if is_true(test_suite, "ffv_enable")? {
        flags.push("ffv_enable");
    }
    if is_true(test_suite, "frg_enable")? {
        flags.push("frg_enable");
    }
    if is_true(test_suite, "hardware_dsp_disable")? {
        flags.push("hw_dsp_disable");
    }
    if is_true(test_suite, "force_serial")? {
        flags.push("force_serial");
    }
    Ok(flags)
}

impl<'a> Processor for FlowGenerator<'a> {
    fn on_node(&mut self, node: &Node) -> Result<Return> {
        let result = match &node.attrs {
            Attrs::PGMFlow(name) => {
                {
                    self.file_path = Some(self.output_dir.join(&format!("{}.tf", name)));

                    self.indent += 1;

                    self.push_body("{");
                    self.indent += 1;

                    self.push_body("{");
                    self.indent += 1;
                    // Generate flow init vars here
                    self.indent -= 1;
                    self.push_body("}, open,\"Init Flow Control Vars\", \"\"");

                    let _ = node.process_children(self);
                    self.indent -= 1;
                    self.push_body("");
                    self.push_body(&format!("}}, open,\"{}\",\"\"", &name.to_uppercase()));

                    self.indent -= 1;

                    let mut f = std::fs::File::create(&self.file_path.as_ref().unwrap())?;
                    writeln!(&mut f, "hp93000,testflow,0.1")?;
                    writeln!(&mut f, "language_revision = 1;")?;
                    writeln!(&mut f, "")?;
                    writeln!(&mut f, "testmethodparameters")?;
                    writeln!(&mut f, "")?;
                    for (name, tm) in &self.test_methods {
                        writeln!(&mut f, "{}:", name)?;
                        for (name, kind, value) in tm.sorted_params() {
                            if let Some(v) = value {
                                match kind {
                                    ParamType::Voltage => {
                                        writeln!(&mut f, r#"  "{}" = "{}[V]";"#, name, v)?
                                    }
                                    ParamType::Current => {
                                        writeln!(&mut f, r#"  "{}" = "{}[A]";"#, name, v)?
                                    }
                                    ParamType::Time => {
                                        writeln!(&mut f, r#"  "{}" = "{}[s]";"#, name, v)?
                                    }
                                    _ => writeln!(&mut f, r#"  "{}" = "{}";"#, name, v)?,
                                }
                            }
                        }
                    }
                    writeln!(&mut f, "")?;
                    writeln!(&mut f, "end")?;
                    writeln!(
                        &mut f,
                        "-----------------------------------------------------------------"
                    )?;
                    writeln!(&mut f, "testmethodlimits")?;
                    //{% if inline_limits %}
                    //{%   for (_, test_method) in test_methods %}
                    //%   test_methods.sorted_collection.each do |method|
                    //%     if method.respond_to?(:limits) && method.limits && method.limits.render?
                    //<%= method.id %>:
                    //  <%= method.limits %>;
                    //%     end
                    //{%   endfor %}
                    //{% endif %}
                    writeln!(&mut f, "")?;
                    writeln!(&mut f, "end")?;
                    writeln!(
                        &mut f,
                        "-----------------------------------------------------------------"
                    )?;
                    writeln!(&mut f, "testmethods")?;
                    //{% for test_method in test_methods %}
                    //<%= method.id %>:
                    //  testmethod_class = "<%= method.klass %>";
                    //{% endfor %}
                    writeln!(&mut f, "")?;
                    for (name, tm) in &self.test_methods {
                        writeln!(&mut f, "{}:", name)?;
                        if let Some(class) = &tm.class_name {
                            writeln!(&mut f, r#"  testmethod_class = "{}";"#, class)?;
                        }
                    }
                    writeln!(&mut f, "")?;
                    writeln!(&mut f, "end")?;
                    writeln!(
                        &mut f,
                        "-----------------------------------------------------------------"
                    )?;
                    writeln!(&mut f, "test_suites")?;
                    writeln!(&mut f, "")?;

                    for (name, ts) in &self.test_suites {
                        writeln!(&mut f, "{}:", name)?;
                        if let Some(v) = ts.get("comment")? {
                            writeln!(&mut f, "  comment = \"{}\";", v)?;
                        }
                        if is_true(ts, "log_first")? {
                            writeln!(&mut f, "  ffc_on_fail = 1;")?;
                        }
                        let fls = flags(ts)?;
                        if !fls.is_empty() {
                            writeln!(&mut f, "  local_flags = {};", fls.join(", "))?;
                        }
                        writeln!(&mut f, "  override = 1;")?;
                        if let Some(v) = ts.get("analog_set")? {
                            writeln!(&mut f, "  override_anaset = \"{}\";", v)?;
                        }
                        if let Some(v) = ts.get("level_equation")? {
                            writeln!(&mut f, "  override_lev_equ_set = \"{}\";", v)?;
                        }
                        if let Some(v) = ts.get("level_spec")? {
                            writeln!(&mut f, "  override_lev_spec_set = \"{}\";", v)?;
                        }
                        if let Some(v) = ts.get("level_set")? {
                            writeln!(&mut f, "  override_levset = \"{}\";", v)?;
                        }
                        if let Some(v) = ts.get("pattern")? {
                            writeln!(&mut f, "  override_seqlbl = \"{}\";", v)?;
                        }
                        if let Some(v) = ts.get("test_number")? {
                            writeln!(&mut f, "  override_test_number = \"{}\";", v)?;
                        }
                        if let Some(v) = ts.test_id {
                            writeln!(&mut f, "  override_testf = {};", self.test_method_names[&v])?;
                        }
                        if let Some(v) = ts.get("timing_equation")? {
                            writeln!(&mut f, "  override_tim_equ_set = \"{}\";", v)?;
                        }
                        if let Some(v) = ts.get("timing_spec")? {
                            writeln!(&mut f, "  override_tim_spec_set = \"{}\";", v)?;
                        }
                        if let Some(v) = ts.get("timing_set")? {
                            writeln!(&mut f, "  override_timset = \"{}\";", v)?;
                        }
                        if let Some(v) = ts.get("site_control")? {
                            writeln!(&mut f, "  site_control = \"{}\";", v)?;
                        }
                        if let Some(v) = ts.get("site_match")? {
                            writeln!(&mut f, "  site_match = {};", v)?;
                        }
                        if let Some(v) = ts.get("test_level")? {
                            writeln!(&mut f, "  test_level = \"{}\";", v)?;
                        }
                    }
                    writeln!(&mut f, "")?;
                    writeln!(&mut f, "end")?;
                    writeln!(
                        &mut f,
                        "-----------------------------------------------------------------"
                    )?;
                    writeln!(&mut f, "test_flow")?;
                    writeln!(&mut f, "")?;
                    for line in &self.flow_header {
                        writeln!(&mut f, "{}", line)?;
                    }
                    for line in &self.flow_body {
                        writeln!(&mut f, "{}", line)?;
                    }
                    for line in &self.flow_footer {
                        writeln!(&mut f, "{}", line)?;
                    }
                    writeln!(&mut f, "")?;
                    writeln!(&mut f, "end")?;
                    writeln!(
                        &mut f,
                        "-----------------------------------------------------------------"
                    )?;
                    writeln!(&mut f, "binning")?;
                    writeln!(&mut f, "end")?;
                    writeln!(
                        &mut f,
                        "-----------------------------------------------------------------"
                    )?;
                    writeln!(&mut f, "oocrule")?;
                    writeln!(&mut f, "")?;
                    writeln!(&mut f, "")?;
                    writeln!(&mut f, "end")?;
                    writeln!(
                        &mut f,
                        "-----------------------------------------------------------------"
                    )?;
                    writeln!(&mut f, "context")?;
                    writeln!(&mut f, "")?;
                    writeln!(&mut f, "")?;
                    writeln!(&mut f, "end")?;
                    writeln!(
                        &mut f,
                        "-----------------------------------------------------------------"
                    )?;
                    writeln!(&mut f, "hardware_bin_descriptions")?;
                    writeln!(&mut f, "")?;
                    //{% for bin in hard_bins %}
                    //  {{bin.number}} = {{bin.description}};
                    //{% endfor %}
                    writeln!(&mut f, "")?;
                    writeln!(&mut f, "end")?;
                    writeln!(
                        &mut f,
                        "-----------------------------------------------------------------"
                    )?;
                }
                Return::None
            }
            Attrs::PGMSubFlow(name, _fid) => {
                self.push_body("{");
                self.indent += 1;
                let _ = node.process_children(self);
                self.indent -= 1;
                self.push_body(&format!("}}, open,\"{}\",\"\"", &name));
                Return::None
            }
            Attrs::PGMGroup(name, _, kind, _) => {
                if kind == &GroupType::Flow {
                    self.push_body("{");
                    self.indent += 1;
                    let _ = node.process_children(self);
                    self.indent -= 1;
                    self.push_body(&format!("}}, open,\"{}\", \"\"", &name));
                } else {
                    let _ = node.process_children(self);
                }
                Return::None
            }
            Attrs::PGMLog(msg) => {
                self.push_body(&format!("print_dl(\"{}\");", msg));
                Return::None
            }
            Attrs::PGMTest(id, _flow_id) => {
                let test = &self.model.test_invocations[id];
                let test_name = test.get("name")?.unwrap();

                if node
                    .children
                    .iter()
                    .any(|n| matches!(n.attrs, Attrs::PGMOnFailed(_)| Attrs::PGMOnPassed(_)))
                {
                    self.push_body(&format!("run_and_branch({}{})", test_name, self.sig));
                    self.push_body("then");
                    self.push_body("{");
                    self.indent += 1;
                    for n in &node.children {
                        if matches!(n.attrs, Attrs::PGMOnPassed(_)) {
                            let _ = n.process_children(self);
                        }
                    }
                    self.indent -= 1;
                    self.push_body("}");
                    self.push_body("else");
                    self.push_body("{");
                    self.indent += 1;
                    for n in &node.children {
                        if matches!(n.attrs, Attrs::PGMOnFailed(_)) {
                            let _ = n.process_children(self);
                        }
                    }
                    self.indent -= 1;
                    self.push_body("}");
                } else {
                    self.push_body(&format!("run({}{});", test_name, self.sig));
                }
                Return::ProcessChildren
            }
            Attrs::PGMTestStr(name, _flow_id) => {
                if node
                    .children
                    .iter()
                    .any(|n| matches!(n.attrs, Attrs::PGMOnFailed(_)| Attrs::PGMOnPassed(_)))
                {
                    self.push_body(&format!("run_and_branch({})", name));
                    self.push_body("then");
                    self.push_body("{");
                    self.indent += 1;
                    for n in &node.children {
                        if matches!(n.attrs, Attrs::PGMOnPassed(_)) {
                            let _ = n.process_children(self);
                        }
                    }
                    self.indent -= 1;
                    self.push_body("}");
                    self.push_body("else");
                    self.push_body("{");
                    self.indent += 1;
                    for n in &node.children {
                        if matches!(n.attrs, Attrs::PGMOnFailed(_)) {
                            let _ = n.process_children(self);
                        }
                    }
                    self.indent -= 1;
                    self.push_body("}");
                } else {
                    self.push_body(&format!("run({});", name));
                }
                Return::ProcessChildren
            }
            Attrs::PGMOnFailed(_) => Return::None, // Done manually within the PGMTest handler
            Attrs::PGMOnPassed(_) => Return::None, // Done manually within the PGMTest handler
            Attrs::PGMBin(bin, softbin, kind) => {
                let softbin = match softbin {
                    None => "".to_string(),
                    Some(s) => format!("{}", s),
                };
                let t = match kind {
                    BinType::Bad => ("fail", "bad", "red"),
                    BinType::Good => ("", "good", "green"),
                };
                self.push_body(&format!(
                    r#"stop_bin "{}", "{}", , {}, noreprobe, {}, {}, over_on;"#,
                    softbin, t.0, t.1, t.2, bin
                ));
                Return::None
            }
            Attrs::PGMResources => Return::None,
            _ => Return::ProcessChildren,
        };
        Ok(result)
    }
}
