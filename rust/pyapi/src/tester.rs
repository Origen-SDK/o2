use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict, PyTuple};
use super::timesets::timeset::{Timeset};
use std::collections::HashMap;
use origen::error::Error;
use super::meta::py_like_apis::list_like_api::{ListLikeAPI, ListLikeIter};
use origen::core::tester::{StubNodes, Generators};
//use pyo3::type_object::PyTypeObject;

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
  //py_ast: PyObject,
}

#[pymethods]
impl PyTester {
  #[new]
  fn new(obj: &PyRawObject) {
    origen::tester().reset().unwrap();
    obj.init({ PyTester {
      python_generators: HashMap::new(),
      metadata: vec!(),
      // py_ast: {
      //     let gil = Python::acquire_gil();
      //     let py = gil.python();
      //     Py::new(py, StubPyAST {}).unwrap().to_object(py)
      //   }
      }
    });
  }

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
        return self.get_timeset();
        //return type_error!(format!("Test! (class '{}')", timeset.get_type().name()));
      } else if timeset.get_type().name().to_string() == "Timeset" {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let obj = timeset.to_object(py);
        //let m = obj.getattr(py, "__module__")?.extract::<String>(py)?;

        //if m == "_origen.dut.timesets" {
          model_id = obj.getattr(py, "__origen__model_id__")?.extract::<usize>(py)?;
          timeset_name = obj.getattr(py, "name")?.extract::<String>(py)?;
        //} else {
        //  return type_error!(format!("Could not interpret 'timeset' argument as _origen.dut.timesets.Timeset object! It appears to be of type 'Timeset', but of module '{}'", m));
        //}
      } else {
        //let obj = timeset.to_object(); // timeset.get_type().name().to_string() == "_origen.dut.timesets.Timeset"
        //if obj.call0()
        //let t = pyo3::type_object::PyTypeObject::type_object(timeset);
        //timeset.get_type().is_instance();
        return type_error!(format!("Could not interpret 'timeset' argument as String or _origen.dut.timesets.Timeset object! (class '{}')", timeset.get_type().name()));
      }
    }

    {
      let mut tester = origen::TESTER.lock().unwrap();
      let dut = origen::DUT.lock().unwrap();
      tester.set_timeset(&dut, model_id, &timeset_name)?;
    }
    self.get_timeset()
    // let model = dut.get_model(model_id).unwrap();
    // let gil = Python::acquire_gil();
    // let py = gil.python();
    // pytimeset!(py, model, model_id, &timeset_name)
  }

  fn set_timeset(&self, timeset: &PyAny) -> PyResult<PyObject> {
    self.timeset(timeset)
  }

  fn cc(slf: PyRef<Self>, comment: &str) -> PyResult<PyObject> {
    let mut tester = origen::tester();
    tester.cc(&comment)?;

    let gil = Python::acquire_gil();
    let py = gil.python();
    Ok(slf.to_object(py))
  }

  /// Expecting more arguments/options to eventually be added here.
  #[args(kwargs="**")]
  fn cycle(slf: PyRef<Self>, kwargs: Option<&PyDict>) -> PyResult<PyObject> {
    let mut tester = origen::tester();
    let mut repeat = None;
    if let Some(_kwargs) = kwargs {
      if let Some(_kwarg) = _kwargs.get_item("repeat") {
        repeat = Some(_kwarg.extract::<usize>()?);
      }
    }
    tester.cycle(repeat)?;

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
    //let name = generator.get_type().name().to_string();
    //let klass = g.downcast_ref::<PyType>()?;
    //let name = klass.name().to_string(); //g.name().to_string();
    let gil = Python::acquire_gil();
    let py = gil.python();

    let obj = g.to_object(py);
    let mut n = obj.getattr(py, "__module__")?.extract::<String>(py)?;
    n.push_str(&format!(".{}", obj.getattr(py, "__qualname__")?.extract::<String>(py)?));
    //let name = klass_name.extract::<String>(py)?;

    tester.register_external_generator(&n)?;
    //let t = g.downcast_ref::<PyType>()?;
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
          // let gil = Python::acquire_gil();
          // let py = gil.python();
          // let klass = g.to_object(py);
          // let klass = g.downcast_ref::<PyType>()?;
          // let name = klass.name().to_string(); //g.name().to_string();
          // tester.target(&name)?;
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

  // fn active_generator(&self) -> PyResult<String> {
  //   // ...
  // }

  fn generate(&self) -> PyResult<()> {
    //let stat;
    let mut targets: Vec<Generators> = vec!();
    {
      let mut tester = origen::tester();
      //stat = tester.generate()?;
      targets = tester.targets().clone();
    }
    //for g in stat.external {
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
              //let klass = gen.to_object(py);
              //let obj = gen.call_method0(py, "__new__", )?;
              //gen.call_method0(py, "__init__")?;
              let inst = gen.call0(py)?;
              let args = PyTuple::new(py, &[Py::new(py, StubPyAST {})?.to_object(py)]);
              inst.call_method1(py, "generate", args)?;
            },
            None => return Err(PyErr::from(Error::new(&format!("Something's gone wrong and Python generator {} cannot be found!", g)))),
          }
        },
        _ => {
          let mut tester = origen::tester();
          tester.generate_target_at(i)?;
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
    //Ok(self.py_ast)
    let gil = Python::acquire_gil();
    let py = gil.python();
    Ok(Py::new(py, StubPyAST {})?.to_object(py))
  }
}

impl PyTester {

  // /// Retrieves a built timeset object from the Origen backend.
  // fn origen_timeset(&self) -> Timeset {
  //   // ...
  // }

  // /// Retrieves a built timeset object from the PyAPI.
  // fn pyorigen_timeset(&self) -> Timeset {
  //   // ...
  // }
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
      StubNodes::Comment {content, meta} => {
        dict.set_item("type", "comment")?;
        dict.set_item("content", content)?;
      },
      StubNodes::Vector {timeset_id, repeat, meta} => {
        dict.set_item("type", "vector")?;
        dict.set_item("repeat", repeat)?;
      },
      StubNodes::Node {meta} => {
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

impl PyNode {
  pub fn new(idx: usize, dict: PyObject) -> Self {
    Self {
      fields: dict,
      idx: idx,
    }
  }
}
