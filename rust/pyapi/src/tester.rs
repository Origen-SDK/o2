use super::pins::pin_header::PinHeader;
use super::timesets::timeset::Timeset;
use origen::core::tester::TesterSource;
use origen::error::Error;
use origen::TEST;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict, PyTuple};
use std::collections::HashMap;

#[pymodule]
pub fn tester(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyTester>()?;
    Ok(())
}

#[pyclass(subclass)]
#[derive(Debug)]
/// Python interface for the tester backend.
pub struct PyTester {
    python_testers: HashMap<String, PyObject>,
    instantiated_testers: HashMap<String, PyObject>,
    metadata: Vec<PyObject>,
}

#[pymethods]
impl PyTester {
    #[new]
    fn new(obj: &PyRawObject) {
        origen::tester().reset(None).unwrap();
        obj.init({
            PyTester {
                python_testers: HashMap::new(),
                instantiated_testers: HashMap::new(),
                metadata: vec![],
            }
        });
    }

    #[args(kwargs = "**")]
    fn reset(slf: PyRef<Self>, kwargs: Option<&PyDict>) -> PyResult<PyObject> {
        let mut ast_name = None;
        if let Some(args) = kwargs {
            if let Some(ast) = args.get_item("ast_name") {
                ast_name = Some(ast.extract::<String>()?);
            }
        }
        origen::tester().reset(ast_name)?;

        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(slf.to_object(py))
    }

    #[args(kwargs = "**")]
    fn clear_dut_dependencies(slf: PyRef<Self>, kwargs: Option<&PyDict>) -> PyResult<PyObject> {
        let mut ast_name = None;
        if let Some(args) = kwargs {
            if let Some(ast) = args.get_item("ast_name") {
                ast_name = Some(ast.extract::<String>()?);
            }
        }
        origen::tester().clear_dut_dependencies(ast_name)?;

        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(slf.to_object(py))
    }

    fn reset_external_testers(slf: PyRef<Self>) -> PyResult<PyObject> {
        origen::tester().reset_external_testers()?;

        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(slf.to_object(py))
    }

    fn reset_targets(slf: PyRef<Self>) -> PyResult<PyObject> {
        origen::tester().reset_targets()?;

        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(slf.to_object(py))
    }

    #[getter]
    /// Property for the current :class:`_origen.dut.timesets.Timeset` or None, if no timeset has been set.
    /// Set to ``None`` to clear the current timeset.
    ///
    /// Returns:
    ///     :class:`_origen.dut.timesets.Timeset` or ``None``
    ///
    /// >>> # Initially no timeset has been set
    /// >>> origen.tester.timeset
    /// None
    /// >>> origen.tester.timeset = origen.dut.timesets.Timeset['my_timeset']
    /// origen.dut.timesets.Timeset['my_timeset']
    /// >>> origen.tester.timeset
    /// origen.dut.timesets.Timeset['my_timeset']
    /// >>> # Clear the current timeset
    /// >>> origen.tester.timeset = None
    /// None
    /// >>> origen.tester.timeset
    /// None
    ///
    /// See Also
    /// --------
    /// :meth:`set_timeset`
    ///
    /// :class:`_origen.dut.timesets.Timeset`
    ///
    /// :ref:`Timing <guides/testers/timing:Timing>`
    fn get_timeset(&self) -> PyResult<PyObject> {
        let tester = origen::tester();
        let dut = origen::dut();
        let gil = Python::acquire_gil();
        let py = gil.python();
        if let Some(t) = tester.get_timeset(&dut) {
            Ok(Py::new(
                py,
                Timeset {
                    name: t.name.clone(),
                    model_id: t.model_id,
                },
            )
            .unwrap()
            .to_object(py))
        } else {
            Ok(py.None())
        }
    }

    #[setter]
    // Note - do not add doc strings here. Add to get_timeset above.
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
                model_id = obj
                    .getattr(py, "__origen__model_id__")?
                    .extract::<usize>(py)?;
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

    /// set_timeset(timeset)
    ///
    /// Sets the timeset.
    ///
    /// >>> origen.tester.set_timeset(origen.dut.timesets['my_timeset'])
    /// origen.tester.timesets['my_timeset']
    ///
    /// Parameters:
    ///     timeset (_origen.dut.timesets.Timeset, None): Timeset to set as current, or ``None`` to clear
    ///
    /// See Also
    /// --------
    /// :meth:`timeset`
    ///
    /// :class:`_origen.dut.timesets.Timeset`
    ///
    /// :ref:`Timing <guides/testers/timing:Timing>`
    fn set_timeset(&self, timeset: &PyAny) -> PyResult<PyObject> {
        self.timeset(timeset)
    }

    #[getter]
    fn get_pin_header(&self) -> PyResult<PyObject> {
        let tester = origen::tester();
        let dut = origen::dut();
        let gil = Python::acquire_gil();
        let py = gil.python();

        if let Some(header) = tester.get_pin_header(&dut) {
            Ok(Py::new(
                py,
                PinHeader {
                    name: header.name.clone(),
                    model_id: header.model_id,
                },
            )
            .unwrap()
            .to_object(py))
        } else {
            Ok(py.None())
        }
    }

    #[setter]
    fn pin_header(&self, pin_header: &PyAny) -> PyResult<PyObject> {
        let (model_id, pin_header_name);

        if pin_header.get_type().name().to_string() == "NoneType" {
            {
                let mut tester = origen::TESTER.lock().unwrap();
                tester.clear_pin_header()?;
            }
            self.issue_callbacks("clear_pin_header")?;
            return self.get_timeset();
        } else if pin_header.get_type().name().to_string() == "PinHeader" {
            let gil = Python::acquire_gil();
            let py = gil.python();
            let obj = pin_header.to_object(py);
            model_id = obj
                .getattr(py, "__origen__model_id__")?
                .extract::<usize>(py)?;
            pin_header_name = obj.getattr(py, "name")?.extract::<String>(py)?;
        } else {
            return type_error!(format!("Could not interpret 'pin_header' argument as _origen.dut.Pins.PinHeader object! (class '{}')", pin_header.get_type().name()));
        }

        {
            {
                let mut tester = origen::TESTER.lock().unwrap();
                let dut = origen::DUT.lock().unwrap();
                tester.set_pin_header(&dut, model_id, &pin_header_name)?;
            }
            self.issue_callbacks("set_pin_header")?;
        }
        self.get_pin_header()
    }

    fn set_pin_header(&self, pin_header: &PyAny) -> PyResult<PyObject> {
        self.pin_header(pin_header)
    }

    /// cc(comment: str) -> self
    ///
    /// Inserts a single-line comment into the AST.
    ///
    /// >>> origen.tester.cc("my comment")
    /// <self>
    /// >>> origen.tester.cc("my first comment").cc("my second comment")
    /// <self>
    ///
    /// See Also
    /// --------
    /// {{ ref_for('pattern_api_comments', 'Commenting pattern source') }}
    /// {{ ref_for('program_api_comments', 'Commenting program source') }}
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

    fn end_pattern(&self) -> PyResult<()> {
        let tester = origen::tester();
        Ok(tester.end_pattern()?)
    }

    fn issue_callbacks(&self, func: &str) -> PyResult<()> {
        // Get the current targeted testers
        let targets;
        {
            let tester = origen::tester();
            targets = tester.targets().clone();
        }

        // issue callbacks in the order which they were targeted
        for (i, t) in targets.iter().enumerate() {
            match t {
                TesterSource::External(g) => {
                    // External testers which the backend can't render itself. Need to render them here.
                    match self.instantiated_testers.get(g) {
                        Some(inst) => {
                            // The tester here is a PyObject - a handle on the class itself.
                            // Instantiate it and call its render method with the AST.
                            let gil = Python::acquire_gil();
                            let py = gil.python();
                            let last_node = TEST.get(0).unwrap().to_pickle();
                            let args =
                                PyTuple::new(py, &[func.to_object(py), last_node.to_object(py)]);

                            // The issue callback function is located in origen.generator.tester_api.TesterAPI
                            // Easier to handle the actual calls there and since its all happening in the Python domain, doesn't really matter
                            // whether it happens here or there.
                            inst.call_method1(py, "__origen__issue_callback__", args)?;
                        }
                        None => {
                            return Err(PyErr::from(Error::new(&format!(
                                "Something's gone wrong and Python tester {} cannot be found!",
                                g
                            ))))
                        }
                    }
                }
                _ => {
                    let mut tester = origen::tester();
                    tester.issue_callback_at(i)?;
                }
            }
        }
        Ok(())
    }

    /// cycle(**kwargs) -> self
    #[args(kwargs = "**")]
    fn cycle(slf: PyRef<Self>, kwargs: Option<&PyDict>) -> PyResult<PyObject> {
        {
            let mut tester = origen::tester();
            let mut repeat = None;
            if let Some(_kwargs) = kwargs {
                if let Some(_kwarg) = _kwargs.get_item("repeat") {
                    repeat = Some(_kwarg.extract::<usize>()?);
                }
            }
            tester.cycle(repeat)?;
        }
        slf.issue_callbacks("cycle")?;

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

    fn register_tester(&mut self, g: &PyAny) -> PyResult<()> {
        let mut tester = origen::tester();
        let gil = Python::acquire_gil();
        let py = gil.python();

        let obj = g.to_object(py);
        let mut n = obj.getattr(py, "__module__")?.extract::<String>(py)?;
        n.push_str(&format!(
            ".{}",
            obj.getattr(py, "__qualname__")?.extract::<String>(py)?
        ));

        tester.register_external_tester(&n)?;
        self.python_testers.insert(n, obj);
        Ok(())
    }

    #[args(testers = "*")]
    fn target(&mut self, testers: &PyTuple) -> PyResult<Vec<String>> {
        if testers.len() > 0 {
            let mut tester = origen::tester();
            for g in testers.iter() {
                // Accept either a string name or the actual class of the tester
                if let Ok(name) = g.extract::<String>() {
                    tester.target(&name)?;
                } else {
                    let gil = Python::acquire_gil();
                    let py = gil.python();

                    let obj = g.to_object(py);
                    let mut n = obj.getattr(py, "__module__")?.extract::<String>(py)?;
                    n.push_str(&format!(
                        ".{}",
                        obj.getattr(py, "__qualname__")?.extract::<String>(py)?
                    ));
                    let t = tester.target(&n)?;
                    match t {
                        TesterSource::External(gen) => {
                            let klass = self.python_testers.get(gen).unwrap();
                            let inst = klass.call0(py)?;
                            self.instantiated_testers.insert(gen.to_string(), inst);
                        }
                        _ => {}
                    }
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

    fn render(&self) -> PyResult<()> {
        let targets;
        {
            let tester = origen::tester();
            targets = tester.targets().clone();
        }
        for (i, t) in targets.iter().enumerate() {
            match t {
                TesterSource::External(g) => {
                    // External testers which the backend can't render itself. Need to render them here.
                    match self.instantiated_testers.get(g) {
                        Some(inst) => {
                            // The tester here is a PyObject - a handle on the class itself.
                            // Instantiate it and call its render method with the AST.
                            let gil = Python::acquire_gil();
                            let py = gil.python();
                            inst.call_method0(py, "render")?;
                        }
                        None => {
                            return Err(PyErr::from(Error::new(&format!(
                                "Something's gone wrong and Python tester {} cannot be found!",
                                g
                            ))))
                        }
                    }
                }
                _ => {
                    let mut tester = origen::tester();
                    //let dut = origen::DUT.lock().unwrap();
                    tester.render_target_at(i)?;
                }
            }
        }
        Ok(())
    }

    #[getter]
    fn testers(&self) -> PyResult<Vec<String>> {
        let tester = origen::tester();
        Ok(tester.testers())
    }
}
