use super::pins::pin_header::PinHeader;
use super::timesets::timeset::Timeset;
use crate::pins::vec_to_ppin_ids;
use origen::core::tester::TesterSource;
use origen::testers::SupportedTester;
use origen::Error;
use origen::{Operation, STATUS, TEST};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict, PyTuple};
use std::collections::HashMap;

pub fn define(py: Python, m: &PyModule) -> PyResult<()> {
    let subm = PyModule::new(py, "tester")?;
    subm.add_class::<PyTester>()?;
    pyapi_metal::alias_method_apply_to_set!(subm, "PyTester", "timeset");
    pyapi_metal::alias_method_apply_to_set!(subm, "PyTester", "pin_header");

    m.add_submodule(subm)?;
    Ok(())
}

#[pyclass(subclass)]
/// Python interface for the tester backend.
pub struct PyTester {
    python_testers: HashMap<SupportedTester, PyObject>,
    instantiated_testers: HashMap<SupportedTester, PyObject>,
    // TODO support metadata on testers
    _metadata: Vec<PyObject>,
}

#[pymethods]
impl PyTester {
    #[new]
    fn new() -> PyResult<Self> {
        origen::tester().init()?;
        Ok(PyTester {
            python_testers: HashMap::new(),
            instantiated_testers: HashMap::new(),
            _metadata: vec![],
        })
    }

    fn _start_eq_block(&self, testers: Vec<&str>) -> PyResult<(usize, usize, Vec<String>)> {
        let mut ts: Vec<SupportedTester> = vec![];
        let mut clean_testers: Vec<String> = vec![];
        for t in testers {
            let st = SupportedTester::new(t)?;
            clean_testers.push(st.to_string());
            ts.push(st);
        }
        let refs = origen::tester().start_tester_eq_block(ts)?;
        Ok((refs.0, refs.1, clean_testers))
    }

    fn _end_eq_block(&self, pat_ref_id: usize, prog_ref_id: usize) -> PyResult<()> {
        origen::tester().end_tester_eq_block(pat_ref_id, prog_ref_id)?;
        Ok(())
    }

    fn _start_neq_block(&self, testers: Vec<&str>) -> PyResult<(usize, usize, Vec<String>)> {
        let mut ts: Vec<SupportedTester> = vec![];
        let mut clean_testers: Vec<String> = vec![];
        for t in testers {
            let st = SupportedTester::new(t)?;
            clean_testers.push(st.to_string());
            ts.push(st);
        }
        let refs = origen::tester().start_tester_neq_block(ts)?;
        Ok((refs.0, refs.1, clean_testers))
    }

    fn _end_neq_block(&self, pat_ref_id: usize, prog_ref_id: usize) -> PyResult<()> {
        origen::tester().end_tester_neq_block(pat_ref_id, prog_ref_id)?;
        Ok(())
    }

    /// Prints out the AST for the current flow to the console (for debugging)
    #[getter]
    fn ast(&self) -> PyResult<()> {
        if Operation::GenerateFlow == STATUS.operation() {
            println!("{}", origen_metal::FLOW.to_string());
        } else {
            println!("{}", origen::TEST.to_string());
        }
        Ok(())
    }

    /// Write out the AST to the given file (for debugging)
    fn ast_to_file(&self, file: &str) -> PyResult<()> {
        let contents = {
            if Operation::GenerateFlow == STATUS.operation() {
                origen_metal::FLOW.to_string()
            } else {
                origen::TEST.to_string()
            }
        };
        std::fs::write(file, contents)?;
        Ok(())
    }

    /// This resets the tester, clearing all loaded targets and any other state, making
    /// it ready for a fresh target load.
    /// This should only be called from Python code for testing, it will be called automatically
    /// by Origen before loading targets.
    fn reset(_self: PyRef<Self>) -> PyResult<()> {
        Ok(origen::tester().reset()?)
    }

    /// This is called by Origen at the start of a generate command, it should never be called by
    /// application code
    fn _prepare_for_generate(&self) -> PyResult<()> {
        origen::tester().prepare_for_generate()?;
        Ok(())
    }

    fn _stats(&self) -> PyResult<Vec<u8>> {
        Ok(origen::tester().stats.to_pickle())
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
    /// * :meth:`set_timeset`
    /// * :class:`_origen.dut.timesets.Timeset`
    /// * :ref:`Timing <guides/testers/timing:Timing>`
    fn get_timeset(&self, py: Python) -> PyResult<PyObject> {
        let tester = origen::tester();
        let dut = origen::dut();
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
    fn timeset(&self, py: Python, timeset: &PyAny) -> PyResult<()> {
        let (model_id, timeset_name);

        // If the timeset is a string, assume its a timeset name on the DUT.
        // If not, it should be either None, to clear the timeset,
        // or a timeset object, in which case we'll look up the name and model ID and go from there.
        if let Ok(_timeset) = timeset.extract::<String>() {
            model_id = 0;
            timeset_name = _timeset;
        } else {
            if timeset.get_type().name()?.to_string() == "NoneType" {
                {
                    let mut tester = origen::TESTER.lock().unwrap();
                    tester.clear_timeset()?;
                }
                self.issue_callbacks(py, "clear_timeset")?;
                return Ok(());
            } else if timeset.get_type().name()?.to_string() == "Timeset" {
                let obj = timeset.to_object(py);
                model_id = obj
                    .getattr(py, "__origen__model_id__")?
                    .extract::<usize>(py)?;
                timeset_name = obj.getattr(py, "name")?.extract::<String>(py)?;
            } else {
                return type_error!(format!("Could not interpret 'timeset' argument as String or _origen.dut.timesets.Timeset object! (class '{}')", timeset.get_type().name()?));
            }
        }

        {
            {
                let mut tester = origen::TESTER.lock().unwrap();
                let dut = origen::DUT.lock().unwrap();
                tester.set_timeset(&dut, model_id, &timeset_name)?;
            }
            self.issue_callbacks(py, "set_timeset")?;
        }
        Ok(())
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
    /// * :meth:`timeset`
    /// * :class:`_origen.dut.timesets.Timeset`
    /// * :ref:`Timing <guides/testers/timing:Timing>`
    fn apply_timeset(&self, py: Python, timeset: &PyAny) -> PyResult<PyObject> {
        self.timeset(py, timeset)?;
        self.get_timeset(py)
    }

    #[getter]
    fn get_pin_header(&self, py: Python) -> PyResult<PyObject> {
        let tester = origen::tester();
        let dut = origen::dut();

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
    fn pin_header(&self, py: Python, pin_header: &PyAny) -> PyResult<()> {
        let (model_id, pin_header_name);

        if pin_header.get_type().name()?.to_string() == "NoneType" {
            {
                let mut tester = origen::TESTER.lock().unwrap();
                tester.clear_pin_header()?;
            }
            self.issue_callbacks(py, "clear_pin_header")?;
            return Ok(());
        } else if pin_header.get_type().name()?.to_string() == "PinHeader" {
            let obj = pin_header.to_object(py);
            model_id = obj
                .getattr(py, "__origen__model_id__")?
                .extract::<usize>(py)?;
            pin_header_name = obj.getattr(py, "name")?.extract::<String>(py)?;
        } else {
            return type_error!(format!("Could not interpret 'pin_header' argument as _origen.dut.Pins.PinHeader object! (class '{}')", pin_header.get_type().name()?));
        }

        {
            {
                let mut tester = origen::TESTER.lock().unwrap();
                let dut = origen::DUT.lock().unwrap();
                tester.set_pin_header(&dut, model_id, &pin_header_name)?;
            }
            self.issue_callbacks(py, "set_pin_header")?;
        }
        Ok(())
    }

    fn apply_pin_header(&self, py: Python, pin_header: &PyAny) -> PyResult<PyObject> {
        self.pin_header(py, pin_header)?;
        self.get_pin_header(py)
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
    /// * {{ link_to('prog-gen:comments', 'Commenting pattern source') }}
    /// * {{ link_to('pat-gen:comments', 'Commenting program source') }}
    fn cc(slf: PyRef<Self>, py: Python, comment: &str) -> PyResult<Py<Self>> {
        {
            let mut tester = origen::tester();
            tester.cc(&comment)?;
        }
        slf.issue_callbacks(py, "cc")?;
        Ok(slf.into())
    }

    #[pyo3(text_signature = "($self, header_comments)")]
    pub fn generate_pattern_header(&self, header_comments: &PyDict) -> PyResult<()> {
        let tester = origen::tester();
        Ok(tester.generate_pattern_header(
            match header_comments.get_item("app") {
                Some(comments) => Some(comments.extract::<Vec<String>>()?),
                None => None,
            },
            match header_comments.get_item("pattern") {
                Some(comments) => Some(comments.extract::<Vec<String>>()?),
                None => None,
            },
        )?)
    }

    fn end_pattern(&self) -> PyResult<()> {
        let tester = origen::tester();
        Ok(tester.end_pattern()?)
    }

    fn issue_callbacks(&self, py: Python, func: &str) -> PyResult<()> {
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
    #[pyo3(signature=(**kwargs))]
    fn cycle(slf: PyRef<Self>, py: Python, kwargs: Option<&PyDict>) -> PyResult<Py<Self>> {
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
        slf.issue_callbacks(py, "cycle")?;

        Ok(slf.into())
    }

    fn repeat(slf: PyRef<Self>, py: Python, count: usize) -> PyResult<Py<Self>> {
        let kwargs = PyDict::new(py);
        kwargs.set_item("repeat", count)?;
        Self::cycle(slf, py, Some(&kwargs))
    }

    #[pyo3(signature=(
        label = None,
        symbol = None,
        pins = None,
        cycles = None,
        mask = None,
    ))]
    fn overlay(
        slf: PyRef<Self>,
        py: Python,
        label: Option<String>,
        symbol: Option<String>,
        pins: Option<Vec<&PyAny>>,
        cycles: Option<usize>,
        mask: Option<num_bigint::BigUint>,
    ) -> PyResult<Py<Self>> {
        let pin_ids;
        {
            if let Some(p) = pins {
                crate::dut::PyDUT::ensure_pins("dut")?;
                let dut = origen::dut();
                pin_ids = Some(vec_to_ppin_ids(&dut, p)?);
            } else {
                pin_ids = None
            }
        }
        {
            let tester = origen::tester();
            tester.overlay(&origen::Overlay::new(label, symbol, cycles, mask, pin_ids)?)?;
        }
        slf.issue_callbacks(py, "overlay")?;
        Ok(slf.into())
    }

    #[pyo3(signature=(symbol=None, cycles=None, mask=None, pins=None))]
    fn capture(
        slf: PyRef<Self>,
        py: Python,
        symbol: Option<String>,
        cycles: Option<usize>,
        mask: Option<num_bigint::BigUint>,
        pins: Option<Vec<&PyAny>>,
    ) -> PyResult<Py<Self>> {
        let pin_ids;
        {
            if let Some(p) = pins {
                crate::dut::PyDUT::ensure_pins("dut")?;
                let dut = origen::dut();
                pin_ids = Some(vec_to_ppin_ids(&dut, p)?);
            } else {
                pin_ids = None
            }
        }
        {
            let tester = origen::tester();
            tester.capture(&origen::Capture::new(symbol, cycles, mask, pin_ids)?)?;
        }
        slf.issue_callbacks(py, "capture")?;
        Ok(slf.into())
    }

    fn register_tester(&mut self, py: Python, g: &PyAny) -> PyResult<()> {
        let mut tester = origen::tester();

        let obj = g.to_object(py);
        let mut n = obj.getattr(py, "__module__")?.extract::<String>(py)?;
        n.push_str(&format!(
            ".{}",
            obj.getattr(py, "__qualname__")?.extract::<String>(py)?
        ));

        let t_id = tester.register_external_tester(&n)?;
        self.python_testers.insert(t_id, obj);
        Ok(())
    }

    #[pyo3(signature=(*testers))]
    fn target(&mut self, py: Python, testers: &PyTuple) -> PyResult<Vec<String>> {
        if testers.len() > 0 {
            let mut tester = origen::tester();
            for g in testers.iter() {
                // Accept either a string name or the actual class of the tester
                if let Ok(name) = g.extract::<String>() {
                    tester.target(SupportedTester::new(&name)?)?;
                } else {
                    let obj = g.to_object(py);
                    let mut n = obj.getattr(py, "__module__")?.extract::<String>(py)?;
                    n.push_str(&format!(
                        ".{}",
                        obj.getattr(py, "__qualname__")?.extract::<String>(py)?
                    ));
                    // Assume a tester loaded via a class is a custom tester
                    let t = tester.target(SupportedTester::new(&format!("CUSTOM::{}", n))?)?;
                    match t {
                        TesterSource::External(gen) => {
                            let klass = self.python_testers.get(gen).unwrap();
                            let inst = klass.call0(py)?;
                            self.instantiated_testers.insert(gen.to_owned(), inst);
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

    /// Attempts to render the pattern on all targeted testers and returns paths to the
    /// output files that have been created.
    /// There is no need for the Python side to do anything with those, but they are returned
    /// in case they are useful in future.
    /// Continue on fail means that any errors will be logged but Origen will continue, if false
    /// it will blow up and immediately return an error to Python.
    #[pyo3(signature=(continue_on_fail=false))]
    fn render_pattern(&self, py: Python, continue_on_fail: bool) -> PyResult<Vec<String>> {
        if origen::LOGGER.has_keyword("show_unprocessed_ast") {
            origen::LOGGER.info("Showing Unprocessed AST");
            origen::LOGGER.info(&format!("{:?}", origen::TEST));
        }
        let mut rendered_patterns: Vec<String> = vec![];
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
                            let _pat = inst.call_method0(py, "render_pattern")?;
                            // TODO - How do we convert this to a path to do the diffing?
                        }
                        None => {
                            // Don't bother masking this type of error, this should be fatal
                            let msg = format!(
                                "Something's gone wrong and Python tester {} cannot be found!",
                                g
                            );
                            return Err(PyErr::from(Error::new(&msg)));
                        }
                    }
                }
                _ => {
                    let mut tester = origen::tester();
                    let pat = tester.render_pattern_for_target_at(i, true);
                    match pat {
                        Err(e) => {
                            let msg = e.to_string();
                            if continue_on_fail {
                                STATUS.inc_unhandled_error_count();
                                log_error!("{}", &msg);
                            } else {
                                return Err(PyErr::from(Error::new(&msg)));
                            }
                        }
                        Ok(paths) => {
                            for path in &paths {
                                rendered_patterns.push(format!("{}", path.display()));
                            }
                        }
                    }
                }
            }
        }
        Ok(rendered_patterns)
    }

    #[getter]
    fn testers(&self) -> PyResult<Vec<String>> {
        Ok(SupportedTester::all_names())
    }
}
