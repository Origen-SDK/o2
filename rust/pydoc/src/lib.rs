extern crate proc_macro;
extern crate proc_macro2;
use crate::proc_macro::{TokenStream};
use syn::{parse_macro_input, DeriveInput, ItemFn, ItemImpl};
use syn;
use quote::{format_ident, quote};
use std::convert::TryFrom;

#[macro_use]
extern crate lazy_static;
use std::sync::{Mutex, MutexGuard};
use std::collections::HashMap;
use std::any::Any;

use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

mod pydoc;



lazy_static! {
    static ref DOC: Mutex<Doc> = Mutex::new(Doc::new());
}

struct Doc {
    pub modules: HashMap<String, DocModule>,
    pub functions: Vec<String>,
    classes: HashMap<String, DocClass>,
}

impl Doc {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            functions: vec!(), //HashMap::new(),
            classes: HashMap::new(),
        }
    }

    pub fn register_class(&mut self, name: String) -> Option<&mut DocClass> {
        self.classes.insert(name.clone(), DocClass::new(name.clone()));
        self.classes.get_mut(&name)
    }

    pub fn write_module(&self, mod_name: &str, path: &PathBuf) {
        // .. Write module
        let mut fname = path.clone();
        fname.push(format!("{}.py", mod_name));
        let mut f = File::create(&fname).expect(&format!("Could not create file {:?}", fname));
        for n in self.modules.get(mod_name).unwrap().classes.iter() {
            println!("class {}:", n);
            if let Some(class) = self.classes.get(n) {
                //println!("{}", class.pydoc());
                write!(f, "{}", class.pydoc()).expect(&format!("Could not write file {:?}", fname));
            } else {
                println!("Error: SubClass {} of module {} was built, but could not find associated docs for {}", n, mod_name, n);
            }
        }
        for n in self.modules.get(mod_name).unwrap().modules.iter() {
            println!("SubM: {}", n);
            let mut mod_path = path.clone();
            mod_path.push(n);
            std::fs::create_dir(&mod_path);
            self.write_module(n, &mod_path);
        }
    }

    pub fn write_file(&self) {
        let mut output_dir = std::env::current_dir().unwrap();
        output_dir.push("target");
        output_dir.push("pydoc");
        output_dir.push("_origen");
        std::fs::create_dir_all(&output_dir);
        println!("{:#?}", output_dir);
        //let mut file = File::create("")?;
        // Create the root file
        //let root_init = std::fs::create_file(output_dir.clone().push("__init__.py"));
        // literate through the modules
        for (n, m) in self.modules.iter() {
            if m.parent.is_none() {
                self.write_module(n, &output_dir);
            }
        }
    }
}

impl Drop for Doc {
    fn drop(&mut self) {
        println!("Destruction!!!");
    }
}

struct DocModule {
    name: String,
    classes: Vec<String>,
    modules: Vec<String>,
    parent: Option<String>,
}

impl DocModule {
    pub fn new(name: String) -> Self {
        Self {
            name: name,
            classes: vec!(),
            modules: vec!(),
            parent: None,
        }
    }
}

struct DocArg {
    identifier: String,
    default: Option<String>,
    args_list: bool,
    kwargs: bool,
}

impl DocArg {
    pub fn new(identifier: String) -> Self {
        Self {
            identifier: identifier,
            default: None,
            args_list: false,
            kwargs: false,
        }
    }
}

struct DocFunction {
    name: String,
    args: Vec<DocArg>,
    getter: bool,
    setter: bool,
    getter_setter_name: Option<String>,
    constructor: bool,
}

impl DocFunction {
    pub fn new(name: String) -> Self {
        Self {
            name: name,
            args: vec!(),
            getter: false,
            setter: false,
            getter_setter_name: None,
            constructor: false,
        }
    }

    pub fn arg_str(&self) -> String {
        self.args.iter().map( |arg| {
            if let Some(d) = arg.default.as_ref() {
                format!("{}={}", arg.identifier, d).to_string()
            } else if arg.args_list {
                format!("*{}", arg.identifier).to_string()
            } else if arg.kwargs {
                format!("**{}", arg.identifier).to_string()
            } else {
                arg.identifier.to_string()
            }
        }).collect::<Vec<String>>().join(", ")
    }

    pub fn pydoc(&self, tabs: Option<usize>) -> Vec<String> {
        let mut out = vec!();
        if self.getter {
            out.push(format!("\t@getter"));
            out.push(format!("\tdef {}(self):", self.name));
        } else if self.setter {
            out.push(format!("\t@getter"));
            out.push(format!("\tdef {}(self, value):", self.name));
        } else if self.constructor {
            out.push(format!("\tdef __init__(self, {}):", self.arg_str()));
        } else {
            out.push(format!("\tdef {}(self, {}):", self.name, self.arg_str()));
        }
        out.push(format!("\t\tpass"));
        out
    }
}

struct DocClass {
    name: String,
    methods: Vec<DocFunction>
}

impl DocClass {
    pub fn new(name: String) -> Self {
        Self {
            name: name,
            methods: vec!(),
        }
    }

    pub fn pydoc(&self) -> String {
        let mut strs = vec!();
        strs.push(format!("class {}:", self.name).to_string());
        for m in self.methods.iter() {
            strs.extend(m.pydoc(Some(1)));
        }
        strs.join("\n")
    }
}

#[proc_macro_attribute]
pub fn pydoc(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut doc = DOC.lock().unwrap();
    if let Ok(ast) = syn::parse::<ItemFn>(item.clone()) {
        // Check the leading attributes. This could denote either a pyfunction, or a pymodule
        for attr in ast.attrs.iter() {
            if let Ok(macro_attr) = attr.parse_meta() {
                match macro_attr {
                    syn::Meta::Path(a) => {
                        if let Some(name) = a.get_ident() {
                            if name == "pymodule" {
                                let top_name = ast.sig.ident.to_string();
                                let mut added_modules: Vec<DocModule> = vec!();
                                {
                                    let mut doc_module;
                                    if doc.modules.contains_key(&top_name) {
                                        println!("CONTAINS!! {}", top_name);
                                        doc_module = doc.modules.get_mut(&top_name).unwrap();
                                    } else {
                                        println!("Added!! {}", top_name);
                                        doc.modules.insert(top_name.clone(), DocModule::new(top_name.clone()));
                                        doc_module = doc.modules.get_mut(&top_name).unwrap()
                                    }
                                    for stmt in ast.block.stmts.iter() {
                                        // A statement here can be pretty much anything but we're looking for something pretty specific:
                                        // One with a trailing semiconductor and of type ExprMethodCall whose method is either 'add_class' or 'add_wrapped'
                                        // https://docs.rs/syn/1.0.14/syn/struct.ExprMethodCall.html
                                        match stmt {
                                            syn::Stmt::Semi(try_exp, ..) => {
                                                match try_exp {
                                                    syn::Expr::Try(exp) => {
                                                        match &*exp.expr {
                                                            syn::Expr::MethodCall(call) => {
                                                                if call.method == "add_class" {
                                                                    // Expecting something like m.add_class::<TimesetContainer>?
                                                                    // The argument we're currently interested in is the <TimesetContainer>
                                                                    match call.turbofish.as_ref().unwrap().args.first().unwrap() {
                                                                        syn::GenericMethodArgument::Type(t_) => {
                                                                            match t_ {
                                                                                syn::Type::Path(t__) => {
                                                                                    doc_module.classes.push(t__.path.segments.first().unwrap().ident.to_string());
                                                                                },
                                                                                _ => panic!("Unknown syn::Type. Expected syn::Type::Path")
                                                                            }
                                                                        },
                                                                        _ => panic!("Unknown identifier token in 'add_class'!")
                                                                    }
                                                                } else if call.method == "add_wrapped" {
                                                                    // Expecting something like 'm.add_wrapped(wrap_pymodule!(pins))?;'
                                                                    // The argument we're currently interested in is the nested 'pins'
                                                                    match call.args.first().unwrap() {
                                                                        syn::Expr::Macro(subcall) => {
                                                                            let wrapped_name = subcall.mac.tokens.to_string();
                                                                            println!("Wrapped Name: {}", wrapped_name);
                                                                            doc_module.modules.push(wrapped_name.clone());
                                                                            let mut wrapped_mod = DocModule::new(wrapped_name);
                                                                            wrapped_mod.parent = Some(top_name.clone());
                                                                            added_modules.push(wrapped_mod);
                                                                        },
                                                                        _ => panic!("Unknown nested expression in add_wrapped")
                                                                    }
                                                                }
                                                            },
                                                            _ => {
                                                                panic!("Expected syn::Expr::MethodCall")
                                                            }
                                                        }
                                                    },
                                                    _ => {
                                                        // Ignoring anything that's not a try expression, including 'unwrap'
                                                    }
                                                }
                                            },
                                            _ => {
                                                // Ignoring everythign but semicolon-terminated expressions
                                                // Note: this means that add_class or add_wrapped invocations that appear inside blocks WON'T be covered.
                                            }
                                        }
                                    }
                                }
                                for wrapped_mod in added_modules {
                                    doc.modules.insert(wrapped_mod.name.clone(), wrapped_mod);
                                }
                            } else if name == "pyfunction" {
                                panic!("pyfunction not supported yet!!!");
                            }
                        }
                    },
                    _ => {
                        // Ignore other stuff
                    }
                }
            }
        }
        quote!(#ast).into()
    } else if let Ok(ast) = syn::parse::<ItemImpl>(item.clone()) {
        //let name = &ast.self_ty;
        let mut klass;
        match &*ast.self_ty {
            syn::Type::Path(arg_type) => {
                //println!("{}", arg_type.path.segments.last().unwrap().ident);
                //klass = doc.register_class(arg_type.path.segments.last().unwrap().ident.to_string()).unwrap();
                let class_name = arg_type.path.segments.last().unwrap().ident.to_string();
                if doc.classes.contains_key(&class_name) {
                    println!("getting class {}", class_name);
                    klass = doc.classes.get_mut(&class_name).unwrap();
                } else {
                    println!("new class!");
                    doc.classes.insert(class_name.clone(), DocClass::new(class_name.clone()));
                    klass = doc.classes.get_mut(&class_name).unwrap();
                    println!("got class!");
                }
            },
            _ => panic!("No type!")
        };

        for i in ast.items.iter() {
            match i {
                syn::ImplItem::Method(m) => {
                    let mut pymethod = DocFunction::new(m.sig.ident.to_string());
                    // Handle any attributes we find. Particularly interested in:
                    //  getter/setting
                    //  args, either default, *, or **
                    //  pydocstring <- Need to add this...
                    let mut arg_details: HashMap<String, String> = HashMap::new();
                    for attr in m.attrs.iter() {
                        // Only supporting outer attributes right now. Not aware of any relevant inner ones at the moment.
                        // ...
                        if let Ok(macro_attr) = attr.parse_meta() {
                            match macro_attr {
                                syn::Meta::Path(a) => {
                                    if let Some(name) = a.get_ident() {
                                        if name == "getter" {
                                            panic!("Getter!");
                                        } else if name == "setter" {
                                            panic!("Setter!");
                                        } else if name == "new" {
                                            //panic!("New not implemented yet!");
                                            pymethod.constructor = true;
                                        }
                                    }
                                },
                                syn::Meta::List(a) => {
                                    if let Some(name) = a.path.get_ident() {
                                        if name == "args" {
                                            // For 'args', expecting something of the form: args(arg_name = "val")
                                            //  where arg_name is a literal (no quotes) and "val" is always in quotes and casted
                                            //  appropriately during invocation.
                                            // https://pyo3.rs/v0.9.0-alpha.1/class.html#method-arguments
                                            // Note that the args here are only providing more information on the args that'll appear in the
                                            // method signature. Just noting any details for the to-be-encountered args here.
                                            for a_ in a.nested {
                                                match a_ {
                                                    syn::NestedMeta::Meta(a__) => {
                                                        match a__ {
                                                            syn::Meta::NameValue(a___) => {
                                                                //let mut doc_arg = DocArg::new(a___.path.get_ident().unwrap().to_string());
                                                                match a___.lit {
                                                                    syn::Lit::Str(l) => {
                                                                        arg_details.insert(a___.path.get_ident().unwrap().to_string(), l.value());
                                                                    },
                                                                    _ => panic!("a___.lit is something else! {:#?}", a___.lit)
                                                                }
                                                                // match a___.lit {
                                                                //     syn::Lit::Str(l) => {
                                                                //         if l.value() == "*" {
                                                                //             doc_arg.args_list = true;
                                                                //         } else if l.value() == "**" {
                                                                //             doc_arg.kwargs = true;
                                                                //         } else {
                                                                //             doc_arg.default = Some(l.value().clone());
                                                                //         }
                                                                //     }
                                                                //     _ => panic!("a___.lit is something else! {:#?}", a___.lit)
                                                                // }
                                                                // args.push((
                                                                //     a___.path.get_ident().unwrap().to_string(),
                                                                //     match a___.lit {
                                                                //         syn::Lit::Str(l) => l.value(),
                                                                //         _ => panic!("a___.lit is something else! {:#?}", a___.lit)
                                                                //     }
                                                                // ));
                                                                //pymethod.args.push(doc_arg);
                                                            },
                                                            _ => panic!("a__ is something else!")
                                                        }
                                                    },
                                                    _ => panic!("a_ is something else!")
                                                }
                                            }
                                            //pymethod.args.push(args);
                                        } else if name == "setter" {
                                            // Expecting either #[setter] or #[setter(property_name)] where 'property name' is a literal (no qootes)
                                            // https://pyo3.rs/v0.9.0-alpha.1/class.html#object-properties
                                            //pymethod.setter = true;
                                        } else if name == "getter" {
                                            // See above.
                                            //pymethod.getter = true;
                                        } else if name == "pydocstring" {
                                            // ...
                                        }
                                    }
                                },
                                syn::Meta::NameValue(a) => {
                                    // ...
                                }
                            }
                            // let p = syn::Path::parse_mod_style(a.path());
                            // if a.ident == "pydocstring" {
                            //     // ...
                            // }
                        }
                    }

                    // Handle the actual function declaration.
                    for ins in m.sig.inputs.iter() {
                        match ins {
                            syn::FnArg::Typed(arg) => {
                                //let pat = arg.pat.downcast::<syn::Pat>().unwrap();
                                //let ty = arg.ty.downcast::<syn::Type>().unwrap();
                                match &*arg.pat {
                                    syn::Pat::Ident(arg_id) => {
                                        //println!("{}", arg_id.ident);
                                        let arg_id_str = arg_id.ident.to_string();
                                        let mut doc_arg = DocArg::new(arg_id_str.clone());
                                        if let Some(default) = arg_details.get(&arg_id_str) {
                                            if default == "*" {
                                                doc_arg.args_list = true;
                                            } else if default == "**" {
                                                doc_arg.kwargs = true;
                                            } else {
                                                doc_arg.default = Some(default.clone());
                                            }
                                        }
                                        pymethod.args.push(doc_arg);
                                    },
                                    _ => println!("No arg!")
                                };
                                /*
                                // Messing with type stuff here. Python is dynamically typed, so this material doesn't actually appear in
                                // its docs, but since pyo3 will raise errors if the type bounds are not meant, might be more user friendly to
                                // parse this in the future and throw it into the docs.
                                // Note: default parameters, arg list (*) and keyword args (**) are handled by pyo3 attributes which we parse elsewhere
                                // and do not in the method declaration itself.
                                match &*arg.ty {
                                    syn::Type::Path(arg_type) => {
                                        println!("{}", arg_type.path.segments.last().unwrap().ident);
                                    },
                                    _ => println!("No type!")
                                };
                                */
                            }
                            _ => {
                                // ... not sure on these yet either
                            }
                        }
                    }
                    klass.methods.push(pymethod);
                },
                _ => {
                    // Ignoring non-methods for now. Not sure if other things in impl (e.g. associated types) have pyo3 implications
                },
            }
            //println!("{:#?}", i);
            println!("");
        }
        doc.write_file();
        let q = quote!(
            #ast
        ).into();
        //println!("AST!! {}", q);
        q
    } else {
        item
    }

    //println!("attr: \"{}\"", attr.to_string());
    //println!("item: \"{}\"", &ast.to_string());
    // let q = quote!(
    //     #ast
    // ).into();
    // println!("AST!! {}", q);
    // q
}
