use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict, PyTuple};
use super::timesets::timeset::{Timeset};
use std::collections::HashMap;
use origen::error::Error;
use super::meta::py_like_apis::list_like_api::{ListLikeAPI, ListLikeIter};
use origen::core::tester::{StubNodes, Generators};
use pyo3::types::IntoPyDict;

#[pymodule]
pub fn tester(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyTester>()?;
    m.add_class::<StubPyAST>()?;
    m.add_class::<PyNode>()?;
    Ok(())
}

#[pyclass(subclass)]
#[derive(Debug)]
pub struct PyTester {
  python_generators: HashMap<String, PyObject>,
  metadata: Vec<PyObject>,
}

#[pymethods]
impl PyTester {
  #[new]
  fn new(obj: &PyRawObject) {
    origen::tester().reset().unwrap();
    obj.init({ PyTester {
      python_generators: HashMap::new(),
      metadata: vec!(),
      }
    });
  }

  fn reset(slf: PyRef<Self>) -> PyResult<PyObject> {
    origen::tester().reset()?;

    let gil = Python::acquire_gil();
    let py = gil.python();
    Ok(slf.to_object(py))
  }

  #[getter]
  fn get_timeset(&self) -> PyResult<PyObject> {
    let tester = origen::tester();
    let dut = origen::dut();
    let gil = Python::acquire_gil();
    let py = gil.python();
    if let Some(t) = tester.get_timeset(&dut) {
      Ok(Py::new(py, Timeset {
        name: t.name.clone(),
        model_id: t.model_id
      }).unwrap().to_object(py))
    } else {
      Ok(py.None())
    }
  }

  #[setter]
  fn timeset(&self, timeset: &PyAny) -> PyResult<PyObject> {
    let (model_id, timeset_name);

    // If the timeset is a string, assume its a timeset name on the DUT.
    // If not, it should be either None, to clear the timeset,
    // or a timeset object, in which case we'll look up the name and model ID and go from there.
    if let Ok(_timeset) = timeset.extract::<String>() {
      model_id = 0;
      timeset_name = _timeset;
    } else {
      if timeset.get_type().name().to_string() == "NoneType" {
        {
          let mut tester = origen::TESTER.lock().unwrap();
          tester.clear_timeset()?;
        }
        self.issue_callbacks("clear_timeset")?;
        return self.get_timeset();
      } else if timeset.get_type().name().to_string() == "Timeset" {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let obj = timeset.to_object(py);
        model_id = obj.getattr(py, "__origen__model_id__")?.extract::<usize>(py)?;
        timeset_name = obj.getattr(py, "name")?.extract::<String>(py)?;
      } else {
        return type_error!(format!("Could not interpret 'timeset' argument as String or _origen.dut.timesets.Timeset object! (class '{}')", timeset.get_type().name()));
      }
    }

    {
      {
        let mut tester = origen::TESTER.lock().unwrap();
        let dut = origen::DUT.lock().unwrap();
        tester.set_timeset(&dut, model_id, &timeset_name)?;
      }
      self.issue_callbacks("set_timeset")?;
    }
    self.get_timeset()
  }

  fn set_timeset(&self, timeset: &PyAny) -> PyResult<PyObject> {
    self.timeset(timeset)
  }

  fn cc(slf: PyRef<Self>, comment: &str) -> PyResult<PyObject> {
    {
      let mut tester = origen::tester();
      tester.cc(&comment)?;
    }
    slf.issue_callbacks("cc")?;

    let gil = Python::acquire_gil();
    let py = gil.python();
    Ok(slf.to_object(py))
  }

  fn issue_callbacks(&self, func: &str) -> PyResult<()> {
    // Get the current targeted generators
    let targets;
    {
      let tester = origen::tester();
      targets = tester.targets().clone();
    }

    // issue callbacks in the order which they were targeted
    for (i, t) in targets.iter().enumerate() {
      match t {
        Generators::External(g) => {
          // External generators which the backend can't generate itself. Need to generate them here.
          match self.python_generators.get(&(g.clone())) {
            Some(gen) => {
              // The generator here is a PyObject - a handle on the class itself.
              // Instantiate it and call its generate method with the AST.
              let gil = Python::acquire_gil();
              let py = gil.python();
              let inst = gen.call0(py)?;
              let args = PyTuple::new(py, &[Py::new(py, StubPyAST {})?.to_object(py)]);

              // Note: We could just try the callback on the generator and ignore an attribute error, but this ignores any attribute errors that
              // may occur inside of the callback itself. So, check first if the attribute exists, so we know we're calling it.
              let has_attr = inst.getattr(py, func);

              // the above attr either didn't throw an error or threw an attribute error, then carry on.
              // Otherwise, something unexpected occured. Throw that error.
              match has_attr {
                Err(e) => {
                  if !e.is_instance::<pyo3::exceptions::AttributeError>(py) {
                    return Err(PyErr::from(e));
                  }
                },
                _ => {
                  inst.call_method1(py, func, args)?;
                },
              }
            },
            None => return Err(PyErr::from(Error::new(&format!("Something's gone wrong and Python generator {} cannot be found!", g)))),
          }
        },
        _ => {
          let mut tester = origen::tester();
          let dut = origen::dut();
          tester.issue_callback_at(func, i, &dut)?;
        }
      }
    }
    Ok(())
  }

  /// Expecting more arguments/options to eventually be added here.
  #[args(kwargs="**")]
  fn cycle(slf: PyRef<Self>, kwargs: Option<&PyDict>) -> PyResult<PyObject> {
    let targets;
    {
      let mut tester = origen::tester();
      let mut repeat = None;
      if let Some(_kwargs) = kwargs {
        if let Some(_kwarg) = _kwargs.get_item("repeat") {
          repeat = Some(_kwarg.extract::<usize>()?);
        }
      }
      tester.cycle(repeat)?;
      targets = tester.targets().clone();
    }

    // issue callbacks
    for (i, t) in targets.iter().enumerate() {
      match t {
        Generators::External(g) => {
          // External generators which the backend can't generate itself. Need to generate them here.
          match slf.python_generators.get(&(g.clone())) {
            Some(gen) => {
              // The generator here is a PyObject - a handle on the class itself.
              // Instantiate it and call its generate method with the AST.
              let gil = Python::acquire_gil();
              let py = gil.python();
              let inst = gen.call0(py)?;
              let args = PyTuple::new(py, &[Py::new(py, StubPyAST {})?.to_object(py)]);
              let r = inst.call_method1(py, "cycle", args);
              match r {
                Err(p) => {
                  if !p.is_instance::<pyo3::exceptions::AttributeError>(py) {
                    return Err(PyErr::from(p));
                  }
                },
                _ => {},
              }
            },
            None => return Err(PyErr::from(Error::new(&format!("Something's gone wrong and Python generator {} cannot be found!", g)))),
          }
        },
        _ => {
          let mut tester = origen::tester();
          let dut = origen::dut();
          tester.issue_callback_at("cycle", i, &dut)?;
        }
      }
    }

    let gil = Python::acquire_gil();
    let py = gil.python();
    Ok(slf.to_object(py))
  }

  fn repeat(slf: PyRef<Self>, count: usize) -> PyResult<PyObject> {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let kwargs = PyDict::new(py);
    kwargs.set_item("repeat", count)?;
    Self::cycle(slf, Some(&kwargs))
  }

  fn register_generator(&mut self, g: &PyAny) -> PyResult<()> {
    let mut tester = origen::tester();
    let gil = Python::acquire_gil();
    let py = gil.python();

    let obj = g.to_object(py);
    let mut n = obj.getattr(py, "__module__")?.extract::<String>(py)?;
    n.push_str(&format!(".{}", obj.getattr(py, "__qualname__")?.extract::<String>(py)?));

    tester.register_external_generator(&n)?;
    self.python_generators.insert(n, obj);
    Ok(())
  }

  #[args(generators="*")]
  fn target(&self, generators: &PyTuple) -> PyResult<Vec<String>> {
    if generators.len() > 0 {
      let mut tester = origen::tester();
      for g in generators.iter() {
        // Accept either a string name or the actual class of the generator
        if let Ok(name) = g.extract::<String>() {
          tester.target(&name)?;
        } else {
          let gil = Python::acquire_gil();
          let py = gil.python();
      
          let obj = g.to_object(py);
          let mut n = obj.getattr(py, "__module__")?.extract::<String>(py)?;
          n.push_str(&format!(".{}", obj.getattr(py, "__qualname__")?.extract::<String>(py)?));
          tester.target(&n)?;
        }
      }
    }
    self.targets()
  }

  #[getter]
  fn targets(&self) -> PyResult<Vec<String>> {
    let tester = origen::tester();
    Ok(tester.targets_as_strs().clone())
  }

  fn clear_targets(slf: PyRef<Self>) -> PyResult<PyObject> {
    let mut tester = origen::tester();
    tester.clear_targets()?;

    let gil = Python::acquire_gil();
    let py = gil.python();
    Ok(slf.to_object(py))
  }

  fn generate(&self) -> PyResult<()> {
    let targets;
    {
      let tester = origen::tester();
      targets = tester.targets().clone();
    }
    for (i, t) in targets.iter().enumerate() {
      match t {
        Generators::External(g) => {
          // External generators which the backend can't generate itself. Need to generate them here.
          match self.python_generators.get(g) {
            Some(gen) => {
              // The generator here is a PyObject - a handle on the class itself.
              // Instantiate it and call its generate method with the AST.
              let gil = Python::acquire_gil();
              let py = gil.python();
              let inst = gen.call0(py)?;
              let args = PyTuple::new(py, &[Py::new(py, StubPyAST {})?.to_object(py)]);
              inst.call_method1(py, "generate", args)?;
            },
            None => return Err(PyErr::from(Error::new(&format!("Something's gone wrong and Python generator {} cannot be found!", g)))),
          }
        },
        _ => {
          let mut tester = origen::tester();
          let dut = origen::DUT.lock().unwrap();
          tester.generate_target_at(i, &dut)?;
        }
      }
    }
    Ok(())
  }

  #[getter]
  fn generators(&self) -> PyResult<Vec<String>> {
    let tester = origen::tester();
    Ok(tester.generators())
  }

  #[getter]
  fn ast(&self) -> PyResult<PyObject> {
    let gil = Python::acquire_gil();
    let py = gil.python();
    Ok(Py::new(py, StubPyAST {})?.to_object(py))
  }
}

impl PyTester {
  pub fn push_metadata(&mut self, item: &PyAny) -> usize {
    let gil = Python::acquire_gil();
    let py = gil.python();

    self.metadata.push(item.to_object(py));
    self.metadata.len() - 1
  }

  pub fn override_metadata_at(&mut self, idx: usize, item: &PyAny) -> PyResult<()> {
    let gil = Python::acquire_gil();
    let py = gil.python();
    if self.metadata.len() > idx {
        self.metadata[idx] = item.to_object(py);
        Ok(())
    } else {
        Err(PyErr::from(Error::new(&format!(
            "Overriding metadata at {} exceeds the size of the current metadata vector!",
            idx
        ))))
    }
  }

  pub fn get_metadata(&self, idx: usize) -> PyResult<&PyObject> {
      Ok(&self.metadata[idx])
  }
}

#[pyclass]
#[derive(Debug, Clone)]
struct StubPyAST {}

#[pymethods]
impl StubPyAST {
  #[getter]
  fn cycle_count(&self) -> PyResult<usize> {
    let tester = origen::tester();
    let ast = tester.get_ast();

    Ok(ast.cycle_count())
  }

  #[getter]
  fn vector_count(&self) -> PyResult<usize> {
    let tester = origen::tester();
    let ast = tester.get_ast();

    Ok(ast.vector_count())
  }
}

impl ListLikeAPI for StubPyAST {
  fn item_ids(&self, _dut: &std::sync::MutexGuard<origen::core::dut::Dut>) -> Vec<usize> {
    // Todo: Turns this into a macro so we don't need the DUT.
    let tester = origen::tester();
    let ast = tester.get_ast();

    // The items ids won't actually be used here since this is structured differently than stuff on the DUT.
    // For prototyping purposes, and to find opportunities for improvement, just hammering in screws here.
    // All we really need from this is a vector that's the same size as the number of nodes in the AST.
    let mut dummy = vec!();
    dummy.resize_with(ast.len(), || { 0 });
    dummy
  }

  fn new_pyitem(&self, py: Python, idx: usize) -> PyResult<PyObject> {
    let tester = origen::tester();
    let ast = tester.get_ast();
    let node = &ast.nodes[idx];
    let dict = PyDict::new(py);
    match node {
      StubNodes::Comment {content, ..} => {
        dict.set_item("type", "comment")?;
        dict.set_item("content", content)?;
      },
      StubNodes::Vector {timeset_id, repeat, ..} => {
        let (model_id, name);
        {
          let dut = origen::dut();
          let tset = &dut.timesets[*timeset_id];
          model_id = tset.model_id;
          name = tset.name.clone();
        }
        let t = Py::new(py, Timeset {
          name: name,
          model_id: model_id
        }).unwrap().to_object(py);

        dict.set_item("timeset", t)?;
        dict.set_item("type", "vector")?;
        dict.set_item("repeat", repeat)?;
      },
      StubNodes::Node {..} => {
        dict.set_item("type", "node")?;
      },
    }
    let py_node = Py::new(py, PyNode::new(idx, dict.to_object(py))).unwrap();
    Ok(py_node.to_object(py))
  }

  fn __iter__(&self) -> PyResult<ListLikeIter> {
    Ok(ListLikeIter {parent: Box::new((*self).clone()), i: 0})
  }
}

#[pyproto]
impl pyo3::class::mapping::PyMappingProtocol for StubPyAST {
  fn __getitem__(&self, idx: &PyAny) -> PyResult<PyObject> {
    ListLikeAPI::__getitem__(self, idx)
  }

  fn __len__(&self) -> PyResult<usize> {
    ListLikeAPI::__len__(self)
  }
}

#[pyproto]
impl pyo3::class::iter::PyIterProtocol for StubPyAST {
  fn __iter__(slf: PyRefMut<Self>) -> PyResult<ListLikeIter> {
    ListLikeAPI::__iter__(&*slf)
  }
}

#[pyclass]
#[derive(Debug)]
pub struct PyNode {
  pub fields: PyObject,
  pub idx: usize,
}

#[pymethods]
impl PyNode {
  #[getter]
  fn fields(&self) -> PyResult<&PyObject> {
    Ok(&self.fields)
  }
}

#[pymethods]
impl PyNode {
  fn add_metadata(&self, id_str: &str, obj: &PyAny) -> PyResult<()> {
    let mut tester = origen::tester();
    let ast = tester.get_mut_ast();
    let node = &mut ast.nodes[self.idx];

    let gil = Python::acquire_gil();
    let py = gil.python();
    let locals = [("origen", py.import("origen")?)].into_py_dict(py);
    let pytester = py
        .eval("origen.tester", None, Some(&locals))
        .unwrap()
        .downcast_mut::<PyTester>()?;
    let idx = pytester.push_metadata(obj);

    node.add_metadata_id(id_str, idx)?;
    Ok(())
  }

  fn get_metadata(&self, id_str: &str) -> PyResult<PyObject> {
    let tester = origen::tester();
    let ast = tester.get_ast();
    let node = &ast.nodes[self.idx];

    let gil = Python::acquire_gil();
    let py = gil.python();
    match node.get_metadata_id(id_str) {
        Some(idx) => {
          let locals = [("origen", py.import("origen")?)].into_py_dict(py);
          let pytester = py
            .eval("origen.tester", None, Some(&locals))
            .unwrap()
            .downcast_mut::<PyTester>()?;
          let obj = pytester.get_metadata(idx)?;
          Ok(obj.to_object(py))
        }
        None => Ok(py.None()),
    }
  }

  fn set_metadata(&self, id_str: &str, obj: &PyAny) -> PyResult<bool> {
    let mut tester = origen::tester();
    let ast = tester.get_mut_ast();
    let node = &mut ast.nodes[self.idx];

    let gil = Python::acquire_gil();
    let py = gil.python();
    let locals = [("origen", py.import("origen")?)].into_py_dict(py);
    let pytester = py
        .eval("origen.tester", None, Some(&locals))
        .unwrap()
        .downcast_mut::<PyTester>()?;
    match node.get_metadata_id(id_str) {
        Some(idx) => {
            pytester.override_metadata_at(idx, obj)?;
            Ok(true)
        }
        None => {
            let idx = pytester.push_metadata(obj);
            node.add_metadata_id(id_str, idx)?;
            Ok(false)
        }
    }
  }

  // This is more of just a prototype at this point to ensure things like this will work.
  fn set(&self, field: &str, value: &PyAny) -> PyResult<()> {
    let mut tester = origen::tester();
    let ast = tester.get_mut_ast();
    let node = &mut ast.nodes[self.idx];
    let node_;

    match node {
      StubNodes::Comment {content: _, meta} => {
        match field {
          "content" => {
            node_ = StubNodes::Comment {
              content: value.extract::<String>()?,
              meta: meta.clone(),
            };
          },
          _ => return Err(PyErr::from(Error::new(&format!("Node type 'comment' does not have field '{}'", field))))
        }
      },
      StubNodes::Vector {timeset_id: _, repeat: _, meta: _} => {
        return Err(PyErr::from(Error::new(&format!("Node type 'vector' does not have field '{}'", field))))
      },
      StubNodes::Node {..} => {
        return Err(PyErr::from(Error::new(&format!("Node type 'node' does not have field '{}'", field))))
      },
    }
    drop(node);
    ast.nodes[self.idx] = node_;
    Ok(())
  }
}

impl PyNode {
  pub fn new(idx: usize, dict: PyObject) -> Self {
    Self {
      fields: dict,
      idx: idx,
    }
  }
}
