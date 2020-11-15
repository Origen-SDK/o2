use crate::generator::ast::*;
use crate::generator::processor::*;
use crate::prog_gen::{BinType, GroupType, Model, Test};
use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Does the final writing of the flow AST to a SMT7 flow file
pub struct WriteToFile<'a> {
    output_dir: PathBuf,
    file_path: Option<PathBuf>,
    flow_header: Vec<String>,
    flow_body: Vec<String>,
    flow_footer: Vec<String>,
    indent: usize,
    model: &'a Model,
    test_methods: BTreeMap<String, &'a Test>,
    sig: Option<String>,
}

pub fn run(ast: &Node, output_dir: &Path, model: &Model) -> Result<PathBuf> {
    let mut p = WriteToFile {
        output_dir: output_dir.to_owned(),
        file_path: None,
        flow_header: vec![],
        flow_body: vec![],
        flow_footer: vec![],
        indent: 0,
        model: model,
        test_methods: BTreeMap::new(),
        sig: Some("864CE8F".to_string()),
    };

    for (i, t) in model.tests.values().enumerate() {
        p.test_methods.insert(format!("tm_{}", i + 1), t);
    }

    ast.process(&mut p)?;
    Ok(p.file_path.unwrap())
}

impl<'a> WriteToFile<'a> {
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

impl<'a> Processor for WriteToFile<'a> {
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
                        for (name, _kind, value) in tm.sorted_params() {
                            if let Some(v) = value {
                                writeln!(&mut f, r#"  "{}" = "{}";"#, name, v)?;
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
                    writeln!(&mut f, "end")?;
                    writeln!(
                        &mut f,
                        "-----------------------------------------------------------------"
                    )?;
                    writeln!(&mut f, "test_suites")?;
                    //{% for test_suite in test_suites %}
                    //<%= suite.name %>:
                    //%     suite.lines.each do |line|
                    //<%= line %>
                    //%     end
                    //{% endfor %}
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
            Attrs::PGMSubFlow(name) => {
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
                    self.push_body(&format!("}}, open,\"{}\",\"\"", &name));
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
                    if let Some(sig) = &self.sig {
                        let s = format!("run_and_branch({}_{})", test_name, sig);
                        self.push_body(&s);
                    } else {
                        self.push_body(&format!("run_and_branch({})", test_name));
                    }
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
                    if let Some(sig) = &self.sig {
                        let s = format!("run({}_{});", test_name, sig);
                        self.push_body(&s);
                    } else {
                        self.push_body(&format!("run({});", test_name));
                    }
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
            Attrs::PGMOnFailed(_) => Return::ProcessChildren,
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
            _ => Return::ProcessChildren,
        };
        Ok(result)
    }
}
