use crate::_helpers::pypath_as_pathbuf;
use origen_metal::ijtag::icl::model as om;
use pyo3::exceptions::{PyIndexError, PyTypeError};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PySlice};
use std::sync::Arc;

pub(crate) fn define(py: Python, parent: &PyModule) -> PyResult<()> {
    let module = PyModule::new(py, "icl")?;
    module.add_wrapped(wrap_pyfunction!(load))?;
    module.add_class::<Model>()?;
    module.add_class::<Instance>()?;
    module.add_class::<Port>()?;
    module.add_class::<ScanRegister>()?;
    module.add_class::<DataRegister>()?;
    module.add_class::<Alias>()?;
    module.add_class::<AliasSegment>()?;
    module.add_class::<AliasBit>()?;
    module.add_class::<Connection>()?;
    module.add_class::<ConnectionSegment>()?;
    module.add_class::<InternalSignal>()?;
    module.add_class::<BitSelection>()?;
    module.add_class::<BitValue>()?;
    module.add_class::<InstanceResults>()?;
    module.add_class::<PortResults>()?;
    module.add_class::<ScanRegisterResults>()?;
    module.add_class::<DataRegisterResults>()?;
    module.add_class::<RegisterResults>()?;
    module.add_class::<AliasResults>()?;
    module.add_class::<ConnectionResults>()?;
    module.add_class::<ConnectionSegmentResults>()?;
    module.add_class::<AliasSegmentResults>()?;
    module.add_class::<AliasBitResults>()?;
    module.add_class::<ResultIterator>()?;
    parent.add_submodule(module)?;
    Ok(())
}

#[pyfunction]
#[pyo3(signature=(path, *, top=None, cache_dir=None, threads=None, preserve_comments=false))]
fn load(
    py: Python,
    path: &PyAny,
    top: Option<String>,
    cache_dir: Option<&PyAny>,
    threads: Option<usize>,
    preserve_comments: bool,
) -> PyResult<Model> {
    let source_path = pypath_as_pathbuf(path)?;
    let cache_dir = cache_dir.map(pypath_as_pathbuf).transpose()?;
    let model = py.allow_threads(move || {
        let mut parser = om::Parser::new();
        if preserve_comments {
            parser = parser.preserve_comments();
        }
        if let Some(threads) = threads {
            if threads == 0 {
                return Err(origen_metal::Error::new("threads must be greater than zero"));
            }
            parser = parser.threads(threads);
        }
        if let Some(cache_dir) = cache_dir {
            parser.load_or_elaborate(&source_path, top.as_deref(), &cache_dir)
        } else {
            let parsed = parser.from_file(&source_path)?;
            if let Some(top) = top.as_deref() {
                parsed.elaborate(top)
            } else {
                parsed.elaborate_unique_root()
            }
        }
    })?;
    Ok(Model {
        model: Arc::new(model),
    })
}

#[pyclass(name = "Model", module = "origen_metal.ijtag.icl")]
pub struct Model {
    model: Arc<om::IclModel>,
}

#[pymethods]
impl Model {
    #[getter]
    fn root(&self) -> Instance {
        Instance::new(self.model.clone(), self.model.root())
    }

    #[getter]
    fn module_count(&self) -> usize {
        self.model.modules().len()
    }

    #[getter]
    fn specialization_count(&self) -> usize {
        self.model.specializations().len()
    }

    #[getter]
    fn instance_count(&self) -> usize {
        self.model.instances().len()
    }

    #[getter]
    fn connection_count(&self) -> usize {
        self.model.connections().len()
    }

    fn resolve_path(&self, py: Python, path: &str) -> PyResult<Instance> {
        let id = py.allow_threads(|| self.model.resolve_path(path))?;
        Ok(Instance::new(self.model.clone(), id))
    }

    fn find_instances(&self, py: Python, pattern: &str) -> PyResult<InstanceResults> {
        let handles = py.allow_threads(|| self.model.find_instances(pattern))?;
        Ok(InstanceResults::new(self.model.clone(), handles))
    }

    fn find_instances_of(&self, py: Python, pattern: &str) -> PyResult<InstanceResults> {
        let handles = py.allow_threads(|| self.model.find_instances_of(pattern))?;
        Ok(InstanceResults::new(self.model.clone(), handles))
    }

    fn find_ports(&self, py: Python, pattern: &str) -> PyResult<PortResults> {
        let handles = py.allow_threads(|| self.model.find_ports(pattern))?;
        Ok(PortResults::new(self.model.clone(), handles))
    }

    fn find_scan_registers(&self, py: Python, pattern: &str) -> PyResult<ScanRegisterResults> {
        let handles = py.allow_threads(|| self.model.find_scan_registers(pattern))?;
        Ok(ScanRegisterResults::new(self.model.clone(), handles))
    }

    fn find_data_registers(&self, py: Python, pattern: &str) -> PyResult<DataRegisterResults> {
        let handles = py.allow_threads(|| self.model.find_data_registers(pattern))?;
        Ok(DataRegisterResults::new(self.model.clone(), handles))
    }

    fn find_registers(&self, py: Python, pattern: &str) -> PyResult<RegisterResults> {
        let handles = py.allow_threads(|| self.model.find_registers(pattern))?;
        Ok(RegisterResults::new(self.model.clone(), handles))
    }

    fn find_aliases(&self, py: Python, pattern: &str) -> PyResult<AliasResults> {
        let handles = py.allow_threads(|| self.model.find_aliases(pattern))?;
        Ok(AliasResults::new(self.model.clone(), handles))
    }

    fn __repr__(&self) -> String {
        format!(
            "<origen_metal.ijtag.icl.Model modules={} instances={}>",
            self.module_count(),
            self.instance_count()
        )
    }
}

#[derive(Clone)]
enum ResultItems {
    Instances(Arc<Vec<om::InstanceId>>),
    Ports(Arc<Vec<om::PortHandle>>),
    ScanRegisters(Arc<Vec<om::ScanRegisterHandle>>),
    DataRegisters(Arc<Vec<om::DataRegisterHandle>>),
    Registers(Arc<Vec<om::RegisterHandle>>),
    Aliases(Arc<Vec<om::AliasHandle>>),
    Connections(Arc<Vec<ConnectionHandle>>),
    ConnectionSegments(Arc<Vec<ConnectionSegmentHandle>>),
    AliasSegments(Arc<Vec<AliasSegmentHandle>>),
    AliasBits(Arc<Vec<AliasBitData>>),
}

impl ResultItems {
    fn len(&self) -> usize {
        match self {
            Self::Instances(v) => v.len(),
            Self::Ports(v) => v.len(),
            Self::ScanRegisters(v) => v.len(),
            Self::DataRegisters(v) => v.len(),
            Self::Registers(v) => v.len(),
            Self::Aliases(v) => v.len(),
            Self::Connections(v) => v.len(),
            Self::ConnectionSegments(v) => v.len(),
            Self::AliasSegments(v) => v.len(),
            Self::AliasBits(v) => v.len(),
        }
    }

    fn item(&self, py: Python, model: Arc<om::IclModel>, index: usize) -> PyResult<PyObject> {
        match self {
            Self::Instances(v) => Py::new(py, Instance::new(model, v[index])).map(|v| v.to_object(py)),
            Self::Ports(v) => Py::new(py, Port::new(model, v[index])).map(|v| v.to_object(py)),
            Self::ScanRegisters(v) => {
                Py::new(py, ScanRegister::new(model, v[index])).map(|v| v.to_object(py))
            }
            Self::DataRegisters(v) => {
                Py::new(py, DataRegister::new(model, v[index])).map(|v| v.to_object(py))
            }
            Self::Registers(v) => register_to_object(py, model, v[index]),
            Self::Aliases(v) => Py::new(py, Alias::new(model, v[index])).map(|v| v.to_object(py)),
            Self::Connections(v) => {
                Py::new(py, Connection::new(model, v[index])).map(|v| v.to_object(py))
            }
            Self::ConnectionSegments(v) => Py::new(
                py,
                ConnectionSegment::new(model, v[index]),
            )
            .map(|v| v.to_object(py)),
            Self::AliasSegments(v) => {
                Py::new(py, AliasSegment::new(model, v[index])).map(|v| v.to_object(py))
            }
            Self::AliasBits(v) => {
                Py::new(py, AliasBit::new(model, v[index].clone())).map(|v| v.to_object(py))
            }
        }
    }
}

#[pyclass(module = "origen_metal.ijtag.icl")]
struct ResultIterator {
    model: Arc<om::IclModel>,
    items: ResultItems,
    index: usize,
}

#[pymethods]
impl ResultIterator {
    fn __iter__(slf: PyRef<Self>) -> Py<ResultIterator> {
        slf.into()
    }

    fn __next__(&mut self, py: Python) -> PyResult<Option<PyObject>> {
        if self.index >= self.items.len() {
            return Ok(None);
        }
        let item = self.items.item(py, self.model.clone(), self.index)?;
        self.index += 1;
        Ok(Some(item))
    }
}

fn normalize_index(index: isize, len: usize) -> PyResult<usize> {
    let normalized = if index < 0 { len as isize + index } else { index };
    if normalized < 0 || normalized >= len as isize {
        Err(PyIndexError::new_err("ICL result index out of range"))
    } else {
        Ok(normalized as usize)
    }
}

fn sliced_indices(slice: &PySlice, len: usize) -> PyResult<Vec<usize>> {
    let indices = slice.indices(len as i64)?;
    let mut output = Vec::with_capacity(indices.slicelength as usize);
    let mut index = indices.start;
    for _ in 0..indices.slicelength {
        output.push(index as usize);
        index += indices.step;
    }
    Ok(output)
}

macro_rules! define_results {
    ($name:ident, $py_name:literal, $handle:ty, $variant:ident) => {
        #[pyclass(name = $py_name, module = "origen_metal.ijtag.icl")]
        pub struct $name {
            model: Arc<om::IclModel>,
            handles: Arc<Vec<$handle>>,
        }

        impl $name {
            fn new(model: Arc<om::IclModel>, handles: Vec<$handle>) -> Self {
                Self {
                    model,
                    handles: Arc::new(handles),
                }
            }
        }

        #[pymethods]
        impl $name {
            fn __len__(&self) -> usize {
                self.handles.len()
            }

            fn __iter__(&self) -> ResultIterator {
                ResultIterator {
                    model: self.model.clone(),
                    items: ResultItems::$variant(self.handles.clone()),
                    index: 0,
                }
            }

            fn __getitem__(&self, py: Python, key: &PyAny) -> PyResult<PyObject> {
                if let Ok(index) = key.extract::<isize>() {
                    return ResultItems::$variant(self.handles.clone()).item(
                        py,
                        self.model.clone(),
                        normalize_index(index, self.handles.len())?,
                    );
                }
                if let Ok(slice) = key.downcast::<PySlice>() {
                    let selected = sliced_indices(slice, self.handles.len())?
                        .into_iter()
                        .map(|index| self.handles[index].clone())
                        .collect();
                    return Py::new(py, Self::new(self.model.clone(), selected))
                        .map(|value| value.to_object(py));
                }
                Err(PyTypeError::new_err("ICL result indices must be integers or slices"))
            }
        }
    };
}

define_results!(InstanceResults, "InstanceResults", om::InstanceId, Instances);
define_results!(PortResults, "PortResults", om::PortHandle, Ports);
define_results!(ScanRegisterResults, "ScanRegisterResults", om::ScanRegisterHandle, ScanRegisters);
define_results!(DataRegisterResults, "DataRegisterResults", om::DataRegisterHandle, DataRegisters);
define_results!(RegisterResults, "RegisterResults", om::RegisterHandle, Registers);
define_results!(AliasResults, "AliasResults", om::AliasHandle, Aliases);

#[derive(Clone, Copy)]
struct ConnectionHandle {
    instance: om::InstanceId,
    connection: om::ConnectionId,
}

#[derive(Clone, Copy)]
struct AliasSegmentHandle {
    instance: om::InstanceId,
    segment: om::AliasSegmentId,
}

#[derive(Clone)]
struct AliasBitData {
    instance: om::InstanceId,
    relative_instance_path: Vec<om::SymbolId>,
    target: om::AliasEndpoint,
    target_index: u32,
    inverted: bool,
    alias_bit_offset: u32,
}

#[derive(Clone, Copy)]
struct ConnectionSegmentHandle {
    instance: om::InstanceId,
    connection: om::ConnectionId,
    index: usize,
}

define_results!(ConnectionResults, "ConnectionResults", ConnectionHandle, Connections);
define_results!(
    ConnectionSegmentResults,
    "ConnectionSegmentResults",
    ConnectionSegmentHandle,
    ConnectionSegments
);
define_results!(AliasSegmentResults, "AliasSegmentResults", AliasSegmentHandle, AliasSegments);
define_results!(AliasBitResults, "AliasBitResults", AliasBitData, AliasBits);

#[pyclass(module = "origen_metal.ijtag.icl")]
struct Instance {
    model: Arc<om::IclModel>,
    id: om::InstanceId,
}

impl Instance {
    fn new(model: Arc<om::IclModel>, id: om::InstanceId) -> Self {
        Self { model, id }
    }
}

#[pymethods]
impl Instance {
    #[getter]
    fn id(&self) -> usize { self.id.as_usize() }
    #[getter]
    fn name(&self) -> String { self.model.instance_name(self.id).to_string() }
    #[getter]
    fn path(&self) -> String { self.model.instance_path(self.id) }
    #[getter]
    fn module_type(&self) -> String {
        self.model.parsed().symbol(self.model.instance_module(self.id).name).to_string()
    }
    #[getter]
    fn qualified_module_type(&self) -> String { self.model.instance_module(self.id).qualified_name.clone() }
    #[getter]
    fn parent(&self) -> Option<Self> {
        self.model.instance(self.id).parent.map(|id| Self::new(self.model.clone(), id))
    }
    #[getter]
    fn children(&self) -> InstanceResults {
        InstanceResults::new(self.model.clone(), self.model.scope(self.id).child_instances().collect())
    }
    #[getter]
    fn ports(&self) -> PortResults {
        PortResults::new(self.model.clone(), self.model.scope(self.id).ports().collect())
    }
    #[getter]
    fn scan_registers(&self) -> ScanRegisterResults {
        ScanRegisterResults::new(self.model.clone(), self.model.scope(self.id).scan_registers().collect())
    }
    #[getter]
    fn data_registers(&self) -> DataRegisterResults {
        DataRegisterResults::new(self.model.clone(), self.model.scope(self.id).data_registers().collect())
    }
    #[getter]
    fn registers(&self) -> RegisterResults {
        RegisterResults::new(self.model.clone(), self.model.scope(self.id).registers().collect())
    }
    #[getter]
    fn aliases(&self) -> AliasResults {
        AliasResults::new(self.model.clone(), self.model.scope(self.id).aliases().collect())
    }
    fn find_instances(&self, pattern: &str) -> PyResult<InstanceResults> {
        Ok(InstanceResults::new(self.model.clone(), self.model.scope(self.id).find_child_instances(pattern)?))
    }
    fn find_instances_of(&self, pattern: &str) -> PyResult<InstanceResults> {
        Ok(InstanceResults::new(self.model.clone(), self.model.scope(self.id).find_child_instances_of(pattern)?))
    }
    fn find_ports(&self, pattern: &str) -> PyResult<PortResults> {
        Ok(PortResults::new(self.model.clone(), self.model.scope(self.id).find_ports(pattern)?))
    }
    fn find_scan_registers(&self, pattern: &str) -> PyResult<ScanRegisterResults> {
        Ok(ScanRegisterResults::new(self.model.clone(), self.model.scope(self.id).find_scan_registers(pattern)?))
    }
    fn find_data_registers(&self, pattern: &str) -> PyResult<DataRegisterResults> {
        Ok(DataRegisterResults::new(self.model.clone(), self.model.scope(self.id).find_data_registers(pattern)?))
    }
    fn find_registers(&self, pattern: &str) -> PyResult<RegisterResults> {
        Ok(RegisterResults::new(self.model.clone(), self.model.scope(self.id).find_registers(pattern)?))
    }
    fn find_aliases(&self, pattern: &str) -> PyResult<AliasResults> {
        Ok(AliasResults::new(self.model.clone(), self.model.scope(self.id).find_aliases(pattern)?))
    }
    fn __repr__(&self) -> String { format!("<Instance {} type={}>", self.path(), self.qualified_module_type()) }
}

#[pyclass(module = "origen_metal.ijtag.icl")]
struct Port { model: Arc<om::IclModel>, handle: om::PortHandle }
impl Port { fn new(model: Arc<om::IclModel>, handle: om::PortHandle) -> Self { Self { model, handle } } }

#[pymethods]
impl Port {
    #[getter] fn id(&self) -> usize { self.handle.port.as_usize() }
    #[getter] fn name(&self) -> String { self.model.parsed().symbol(self.model.port(self.handle).name).to_string() }
    #[getter] fn path(&self) -> String { self.model.port_path(self.handle) }
    #[getter] fn kind(&self) -> &'static str { port_type_name(self.model.port(self.handle).kind) }
    #[getter] fn width(&self) -> u32 { self.model.port(self.handle).width }
    #[getter] fn first_index(&self) -> u32 { self.model.port(self.handle).first_index }
    #[getter] fn last_index(&self) -> u32 { self.model.port(self.handle).last_index }
    #[getter] fn owner(&self) -> Instance { Instance::new(self.model.clone(), self.handle.instance) }
    #[getter] fn active_polarity(&self) -> Option<bool> { self.model.port(self.handle).active_polarity }
    #[getter] fn default_load_value(&self, py: Python) -> PyResult<Option<PyObject>> { optional_value_to_py(py, &self.model, self.model.port(self.handle).default_load_value.as_ref()) }
    #[getter] fn enum_ref(&self) -> Option<String> { self.model.port(self.handle).enum_ref.map(|v| self.model.parsed().symbol(v).to_string()) }
    #[getter] fn connections(&self) -> ConnectionResults { connections_for(self.model.clone(), self.handle.instance, om::ConnectionOwner::Port(self.handle.port)) }
}

#[pyclass(module = "origen_metal.ijtag.icl")]
struct ScanRegister { model: Arc<om::IclModel>, handle: om::ScanRegisterHandle }
impl ScanRegister { fn new(model: Arc<om::IclModel>, handle: om::ScanRegisterHandle) -> Self { Self { model, handle } } }

#[pymethods]
impl ScanRegister {
    #[getter] fn id(&self) -> usize { self.handle.register.as_usize() }
    #[getter] fn name(&self) -> String { self.model.parsed().symbol(self.model.scan_register(self.handle).name).to_string() }
    #[getter] fn path(&self) -> String { self.model.scan_register_path(self.handle) }
    #[getter] fn kind(&self) -> &'static str { "scan" }
    #[getter] fn width(&self) -> u32 { self.model.scan_register(self.handle).width }
    #[getter] fn first_index(&self) -> u32 { self.model.scan_register(self.handle).first_index }
    #[getter] fn last_index(&self) -> u32 { self.model.scan_register(self.handle).last_index }
    #[getter] fn owner(&self) -> Instance { Instance::new(self.model.clone(), self.handle.instance) }
    #[getter] fn default_load_value(&self, py: Python) -> PyResult<Option<PyObject>> { optional_value_to_py(py, &self.model, self.model.scan_register(self.handle).default_load_value.as_ref()) }
    #[getter] fn reset_value(&self, py: Python) -> PyResult<Option<PyObject>> { optional_value_to_py(py, &self.model, self.model.scan_register(self.handle).reset_value.as_ref()) }
    #[getter] fn enum_ref(&self) -> Option<String> { self.model.scan_register(self.handle).enum_ref.map(|v| self.model.parsed().symbol(v).to_string()) }
    #[getter] fn connections(&self) -> ConnectionResults { connections_for(self.model.clone(), self.handle.instance, om::ConnectionOwner::ScanRegister(self.handle.register)) }
}

#[pyclass(module = "origen_metal.ijtag.icl")]
struct DataRegister { model: Arc<om::IclModel>, handle: om::DataRegisterHandle }
impl DataRegister { fn new(model: Arc<om::IclModel>, handle: om::DataRegisterHandle) -> Self { Self { model, handle } } }

#[pymethods]
impl DataRegister {
    #[getter] fn id(&self) -> usize { self.handle.register.as_usize() }
    #[getter] fn name(&self) -> String { self.model.parsed().symbol(self.model.data_register(self.handle).name).to_string() }
    #[getter] fn path(&self) -> String { self.model.data_register_path(self.handle) }
    #[getter] fn kind(&self) -> &'static str { "data" }
    #[getter] fn width(&self) -> u32 { self.model.data_register(self.handle).width }
    #[getter] fn first_index(&self) -> u32 { self.model.data_register(self.handle).first_index }
    #[getter] fn last_index(&self) -> u32 { self.model.data_register(self.handle).last_index }
    #[getter] fn owner(&self) -> Instance { Instance::new(self.model.clone(), self.handle.instance) }
    #[getter] fn default_load_value(&self, py: Python) -> PyResult<Option<PyObject>> { optional_value_to_py(py, &self.model, self.model.data_register(self.handle).default_load_value.as_ref()) }
    #[getter] fn reset_value(&self, py: Python) -> PyResult<Option<PyObject>> { optional_value_to_py(py, &self.model, self.model.data_register(self.handle).reset_value.as_ref()) }
    #[getter] fn enum_ref(&self) -> Option<String> { self.model.data_register(self.handle).enum_ref.map(|v| self.model.parsed().symbol(v).to_string()) }
    #[getter] fn connections(&self) -> ConnectionResults { connections_for(self.model.clone(), self.handle.instance, om::ConnectionOwner::DataRegister(self.handle.register)) }
}

#[pyclass(module = "origen_metal.ijtag.icl")]
struct InternalSignal {
    model: Arc<om::IclModel>,
    instance: om::InstanceId,
    id: om::InternalSignalId,
}

impl InternalSignal {
    fn new(model: Arc<om::IclModel>, instance: om::InstanceId, id: om::InternalSignalId) -> Self {
        Self { model, instance, id }
    }
}

#[pymethods]
impl InternalSignal {
    #[getter]
    fn id(&self) -> usize { self.id.as_usize() }
    #[getter]
    fn name(&self) -> String { self.model.parsed().symbol(self.model.internal_signal(self.id).name).to_string() }
    #[getter]
    fn path(&self) -> String { self.model.internal_signal_path(self.instance, self.id) }
    #[getter]
    fn kind(&self) -> &'static str { internal_signal_type_name(self.model.internal_signal(self.id).kind) }
    #[getter]
    fn width(&self) -> u32 { self.model.internal_signal(self.id).width }
    #[getter]
    fn first_index(&self) -> u32 { self.model.internal_signal(self.id).first_index }
    #[getter]
    fn last_index(&self) -> u32 { self.model.internal_signal(self.id).last_index }
    #[getter]
    fn owner(&self) -> Instance { Instance::new(self.model.clone(), self.instance) }
    #[getter]
    fn connections(&self) -> ConnectionResults { connections_for(self.model.clone(), self.instance, om::ConnectionOwner::InternalSignal(self.id)) }
}

#[pyclass(module = "origen_metal.ijtag.icl")]
struct Alias {
    model: Arc<om::IclModel>,
    handle: om::AliasHandle,
}

impl Alias {
    fn new(model: Arc<om::IclModel>, handle: om::AliasHandle) -> Self { Self { model, handle } }
}

#[pymethods]
impl Alias {
    #[getter] fn id(&self) -> usize { self.handle.alias.as_usize() }
    #[getter] fn name(&self) -> String { self.model.parsed().symbol(self.model.alias(self.handle).name).to_string() }
    #[getter] fn path(&self) -> String { self.model.alias_path(self.handle) }
    #[getter] fn width(&self) -> u32 { self.model.alias(self.handle).width }
    #[getter] fn first_index(&self) -> u32 { self.model.alias(self.handle).first_index }
    #[getter] fn last_index(&self) -> u32 { self.model.alias(self.handle).last_index }
    #[getter] fn owner(&self) -> Instance { Instance::new(self.model.clone(), self.handle.instance) }
    #[getter]
    fn segments(&self) -> AliasSegmentResults {
        let handles = self.model.alias(self.handle).segments.iter().map(|segment| AliasSegmentHandle { instance: self.handle.instance, segment: *segment }).collect();
        AliasSegmentResults::new(self.model.clone(), handles)
    }
    #[getter]
    fn bits(&self) -> AliasBitResults {
        let bits = self.model.alias_bits(self.handle).map(|bit| AliasBitData {
            instance: self.handle.instance,
            relative_instance_path: bit.relative_instance_path.to_vec(),
            target: bit.target,
            target_index: bit.target_index,
            inverted: bit.inverted,
            alias_bit_offset: bit.alias_bit_offset,
        }).collect();
        AliasBitResults::new(self.model.clone(), bits)
    }
}

#[pyclass(module = "origen_metal.ijtag.icl")]
struct AliasSegment {
    model: Arc<om::IclModel>,
    handle: AliasSegmentHandle,
}

impl AliasSegment {
    fn new(model: Arc<om::IclModel>, handle: AliasSegmentHandle) -> Self { Self { model, handle } }
}

#[pymethods]
impl AliasSegment {
    #[getter] fn relative_path(&self) -> Vec<String> {
        self.segment().relative_instance_path.iter().map(|s| self.model.parsed().symbol(*s).to_string()).collect()
    }
    #[getter] fn target(&self, py: Python) -> PyResult<PyObject> {
        alias_target_to_py(py, self.model.clone(), self.handle.instance, &self.segment().relative_instance_path, self.segment().target)
    }
    #[getter] fn selection(&self) -> BitSelection { BitSelection::new(self.segment().selection, self.endpoint_width()) }
    #[getter] fn inverted(&self) -> bool { self.segment().inverted }
    #[getter] fn alias_bit_offset(&self) -> u32 { self.segment().alias_bit_offset }
}

impl AliasSegment {
    fn segment(&self) -> &om::AliasSegment { self.model.alias_segment(self.handle.segment) }
    fn endpoint_width(&self) -> u32 { endpoint_width(&self.model, self.segment().target) }
}

#[pyclass(module = "origen_metal.ijtag.icl")]
struct AliasBit {
    model: Arc<om::IclModel>,
    data: AliasBitData,
}

impl AliasBit { fn new(model: Arc<om::IclModel>, data: AliasBitData) -> Self { Self { model, data } } }

#[pymethods]
impl AliasBit {
    #[getter] fn relative_path(&self) -> Vec<String> { self.data.relative_instance_path.iter().map(|s| self.model.parsed().symbol(*s).to_string()).collect() }
    #[getter] fn target(&self, py: Python) -> PyResult<PyObject> { alias_target_to_py(py, self.model.clone(), self.data.instance, &self.data.relative_instance_path, self.data.target) }
    #[getter] fn target_index(&self) -> u32 { self.data.target_index }
    #[getter] fn inverted(&self) -> bool { self.data.inverted }
    #[getter] fn alias_bit_offset(&self) -> u32 { self.data.alias_bit_offset }
}

#[pyclass(module = "origen_metal.ijtag.icl")]
struct BitSelection {
    selection: om::BitSelection,
    whole_width: u32,
}

impl BitSelection { fn new(selection: om::BitSelection, whole_width: u32) -> Self { Self { selection, whole_width } } }

#[pymethods]
impl BitSelection {
    #[getter] fn kind(&self) -> &'static str { match self.selection { om::BitSelection::Whole => "whole", om::BitSelection::Index(_) => "index", om::BitSelection::Range { .. } => "range" } }
    #[getter] fn first(&self) -> Option<u32> { match self.selection { om::BitSelection::Whole => None, om::BitSelection::Index(i) => Some(i), om::BitSelection::Range { first, .. } => Some(first) } }
    #[getter] fn last(&self) -> Option<u32> { match self.selection { om::BitSelection::Whole => None, om::BitSelection::Index(i) => Some(i), om::BitSelection::Range { last, .. } => Some(last) } }
    #[getter] fn descending(&self) -> Option<bool> { match self.selection { om::BitSelection::Range { first, last } => Some(first >= last), _ => None } }
    #[getter] fn width(&self) -> u32 { self.selection.width(self.whole_width) }
}

#[pyclass(module = "origen_metal.ijtag.icl")]
struct BitValue {
    width: u32,
    value: num_bigint::BigUint,
    unknown: num_bigint::BigUint,
}

#[pymethods]
impl BitValue {
    #[getter] fn width(&self) -> u32 { self.width }
    #[getter] fn value(&self, py: Python) -> PyObject { self.value.clone().to_object(py) }
    #[getter] fn unknown_mask(&self, py: Python) -> PyObject { self.unknown.clone().to_object(py) }
    #[getter] fn is_fully_known(&self) -> bool { self.unknown == num_bigint::BigUint::from(0u8) }
    fn __int__(&self, py: Python) -> PyResult<PyObject> {
        if self.is_fully_known() { Ok(self.value(py)) } else { Err(pyo3::exceptions::PyValueError::new_err("Cannot convert a bit value containing unknown bits to int")) }
    }
    fn __repr__(&self) -> String { format!("<BitValue width={} unknown={}>", self.width, !self.is_fully_known()) }
}

#[pyclass(module = "origen_metal.ijtag.icl")]
struct Connection {
    model: Arc<om::IclModel>,
    handle: ConnectionHandle,
}

impl Connection { fn new(model: Arc<om::IclModel>, handle: ConnectionHandle) -> Self { Self { model, handle } } fn connection(&self) -> &om::ResolvedConnection { self.model.connection(self.handle.connection) } }

#[pymethods]
impl Connection {
    #[getter] fn id(&self) -> usize { self.handle.connection.as_usize() }
    #[getter] fn kind(&self) -> &'static str { connection_kind_name(self.connection().kind) }
    #[getter] fn source_span(&self) -> (u32, u32) { let s = self.connection().source; (s.start, s.end) }
    #[getter] fn owner(&self, py: Python) -> PyResult<PyObject> { connection_owner_to_py(py, self.model.clone(), self.handle.instance, self.connection().owner) }
    #[getter]
    fn segments(&self) -> ConnectionSegmentResults {
        ConnectionSegmentResults::new(self.model.clone(), (0..self.connection().segments.len()).map(|index| ConnectionSegmentHandle { instance: self.handle.instance, connection: self.handle.connection, index }).collect())
    }
}

#[pyclass(module = "origen_metal.ijtag.icl")]
struct ConnectionSegment { model: Arc<om::IclModel>, handle: ConnectionSegmentHandle }
impl ConnectionSegment { fn new(model: Arc<om::IclModel>, handle: ConnectionSegmentHandle) -> Self { Self { model, handle } } fn segment(&self) -> &om::ConnectionSegment { &self.model.connection(self.handle.connection).segments[self.handle.index] } }

#[pymethods]
impl ConnectionSegment {
    #[getter] fn relative_path(&self) -> Vec<String> { self.segment().relative_instance_path.iter().map(|s| self.model.parsed().symbol(*s).to_string()).collect() }
    #[getter] fn target(&self, py: Python) -> PyResult<PyObject> { connection_target_to_py(py, self.model.clone(), self.handle.instance, self.segment()) }
    #[getter] fn selection(&self) -> BitSelection {
        let width = match &self.segment().target { om::ConnectionTarget::Object(target) => connection_endpoint_width(&self.model, *target), om::ConnectionTarget::Constant(om::ParameterValue::Bits { width, .. }) => *width, _ => 1 };
        BitSelection::new(self.segment().selection, width)
    }
    #[getter] fn inverted(&self) -> bool { self.segment().inverted }
}

fn connections_for(model: Arc<om::IclModel>, instance: om::InstanceId, owner: om::ConnectionOwner) -> ConnectionResults {
    let handles = model.connection_ids_for(owner).map(|connection| ConnectionHandle { instance, connection }).collect();
    ConnectionResults::new(model, handles)
}

fn register_to_object(py: Python, model: Arc<om::IclModel>, handle: om::RegisterHandle) -> PyResult<PyObject> {
    match handle {
        om::RegisterHandle::Scan(handle) => Py::new(py, ScanRegister::new(model, handle)).map(|v| v.to_object(py)),
        om::RegisterHandle::Data(handle) => Py::new(py, DataRegister::new(model, handle)).map(|v| v.to_object(py)),
    }
}

fn alias_target_to_py(py: Python, model: Arc<om::IclModel>, instance: om::InstanceId, path: &[om::SymbolId], target: om::AliasEndpoint) -> PyResult<PyObject> {
    let target_instance = model.resolve_relative_path(instance, path)?;
    match target {
        om::AliasEndpoint::Port(id) => Py::new(py, Port::new(model, om::PortHandle { instance: target_instance, port: id })).map(|v| v.to_object(py)),
        om::AliasEndpoint::ScanRegister(id) => Py::new(py, ScanRegister::new(model, om::ScanRegisterHandle { instance: target_instance, register: id })).map(|v| v.to_object(py)),
        om::AliasEndpoint::DataRegister(id) => Py::new(py, DataRegister::new(model, om::DataRegisterHandle { instance: target_instance, register: id })).map(|v| v.to_object(py)),
        om::AliasEndpoint::InternalSignal(id) => Py::new(py, InternalSignal::new(model, target_instance, id)).map(|v| v.to_object(py)),
    }
}

fn connection_target_to_py(py: Python, model: Arc<om::IclModel>, instance: om::InstanceId, segment: &om::ConnectionSegment) -> PyResult<PyObject> {
    match &segment.target {
        om::ConnectionTarget::Constant(value) => value_to_py(py, &model, value),
        om::ConnectionTarget::Object(target) => {
            let target_instance = model.resolve_relative_path(instance, &segment.relative_instance_path)?;
            match target {
                om::ConnectionEndpoint::Instance(definition) => {
                    let child = model.child_for_definition(target_instance, *definition).ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("Unable to resolve connection instance"))?;
                    Py::new(py, Instance::new(model, child)).map(|v| v.to_object(py))
                }
                om::ConnectionEndpoint::Port(id) => Py::new(py, Port::new(model, om::PortHandle { instance: target_instance, port: *id })).map(|v| v.to_object(py)),
                om::ConnectionEndpoint::ScanRegister(id) => Py::new(py, ScanRegister::new(model, om::ScanRegisterHandle { instance: target_instance, register: *id })).map(|v| v.to_object(py)),
                om::ConnectionEndpoint::DataRegister(id) => Py::new(py, DataRegister::new(model, om::DataRegisterHandle { instance: target_instance, register: *id })).map(|v| v.to_object(py)),
                om::ConnectionEndpoint::Alias(id) => Py::new(py, Alias::new(model, om::AliasHandle { instance: target_instance, alias: *id })).map(|v| v.to_object(py)),
                om::ConnectionEndpoint::InternalSignal(id) => Py::new(py, InternalSignal::new(model, target_instance, *id)).map(|v| v.to_object(py)),
            }
        }
    }
}

fn connection_owner_to_py(py: Python, model: Arc<om::IclModel>, instance: om::InstanceId, owner: om::ConnectionOwner) -> PyResult<PyObject> {
    match owner {
        om::ConnectionOwner::Port(id) => Py::new(py, Port::new(model, om::PortHandle { instance, port: id })).map(|v| v.to_object(py)),
        om::ConnectionOwner::ScanRegister(id) => Py::new(py, ScanRegister::new(model, om::ScanRegisterHandle { instance, register: id })).map(|v| v.to_object(py)),
        om::ConnectionOwner::DataRegister(id) => Py::new(py, DataRegister::new(model, om::DataRegisterHandle { instance, register: id })).map(|v| v.to_object(py)),
        om::ConnectionOwner::InternalSignal(id) => Py::new(py, InternalSignal::new(model, instance, id)).map(|v| v.to_object(py)),
        om::ConnectionOwner::InstanceInput { instance: definition, port } => {
            let child = model.child_for_definition(instance, definition).ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("Unable to resolve connection owner instance"))?;
            Py::new(py, Port::new(model, om::PortHandle { instance: child, port })).map(|v| v.to_object(py))
        }
    }
}

fn optional_value_to_py(py: Python, model: &om::IclModel, value: Option<&om::ParameterValue>) -> PyResult<Option<PyObject>> { value.map(|value| value_to_py(py, model, value)).transpose() }

fn value_to_py(py: Python, model: &om::IclModel, value: &om::ParameterValue) -> PyResult<PyObject> {
    match value {
        om::ParameterValue::Integer(value) => Ok(value.clone().to_object(py)),
        om::ParameterValue::String(value) => Ok(value.to_object(py)),
        om::ParameterValue::Symbol(value) => Ok(model.parsed().symbol(*value).to_object(py)),
        om::ParameterValue::Bits { width, value, unknown } => Py::new(py, BitValue { width: *width, value: value.clone(), unknown: unknown.clone() }).map(|v| v.to_object(py)),
    }
}

fn endpoint_width(model: &om::IclModel, target: om::AliasEndpoint) -> u32 {
    match target {
        om::AliasEndpoint::Port(id) => model.port(om::PortHandle { instance: model.root(), port: id }).width,
        om::AliasEndpoint::ScanRegister(id) => model.scan_register(om::ScanRegisterHandle { instance: model.root(), register: id }).width,
        om::AliasEndpoint::DataRegister(id) => model.data_register(om::DataRegisterHandle { instance: model.root(), register: id }).width,
        om::AliasEndpoint::InternalSignal(id) => model.internal_signal(id).width,
    }
}

fn connection_endpoint_width(model: &om::IclModel, target: om::ConnectionEndpoint) -> u32 {
    match target {
        om::ConnectionEndpoint::Instance(_) => 1,
        om::ConnectionEndpoint::Port(id) => model.port(om::PortHandle { instance: model.root(), port: id }).width,
        om::ConnectionEndpoint::ScanRegister(id) => model.scan_register(om::ScanRegisterHandle { instance: model.root(), register: id }).width,
        om::ConnectionEndpoint::DataRegister(id) => model.data_register(om::DataRegisterHandle { instance: model.root(), register: id }).width,
        om::ConnectionEndpoint::Alias(id) => model.alias(om::AliasHandle { instance: model.root(), alias: id }).width,
        om::ConnectionEndpoint::InternalSignal(id) => model.internal_signal(id).width,
    }
}

fn port_type_name(kind: origen_metal::ijtag::icl::PortType) -> &'static str {
    use origen_metal::ijtag::icl::PortType::*;
    match kind { ScanIn=>"scan_in",ScanOut=>"scan_out",ShiftEn=>"shift_enable",CaptureEn=>"capture_enable",UpdateEn=>"update_enable",DataIn=>"data_in",DataOut=>"data_out",ToShiftEn=>"to_shift_enable",ToUpdateEn=>"to_update_enable",ToCaptureEn=>"to_capture_enable",Select=>"select",ToSelect=>"to_select",Reset=>"reset",ToReset=>"to_reset",Tms=>"tms",ToTms=>"to_tms",Tck=>"tck",ToTck=>"to_tck",Clock=>"clock",ToClock=>"to_clock",Trst=>"trst",ToTrst=>"to_trst",ToIrSelect=>"to_ir_select",Address=>"address",WriteEn=>"write_enable",ReadEn=>"read_enable" }
}

fn internal_signal_type_name(kind: om::InternalSignalType) -> &'static str { match kind { om::InternalSignalType::Logic=>"logic",om::InternalSignalType::Mux(origen_metal::ijtag::icl::MuxType::Scan)=>"scan_mux",om::InternalSignalType::Mux(origen_metal::ijtag::icl::MuxType::Data)=>"data_mux",om::InternalSignalType::Mux(origen_metal::ijtag::icl::MuxType::Clock)=>"clock_mux",om::InternalSignalType::OneHotScan=>"one_hot_scan",om::InternalSignalType::OneHotData=>"one_hot_data" } }

fn connection_kind_name(kind: om::ConnectionKind) -> &'static str { match kind { om::ConnectionKind::Source=>"source",om::ConnectionKind::Enable=>"enable",om::ConnectionKind::ScanInSource=>"scan_in_source",om::ConnectionKind::CaptureSource=>"capture_source",om::ConnectionKind::WriteEnSource=>"write_enable_source",om::ConnectionKind::WriteDataSource=>"write_data_source",om::ConnectionKind::ReadDataSource=>"read_data_source",om::ConnectionKind::InstanceInput=>"instance_input",om::ConnectionKind::MuxSelect=>"mux_select",om::ConnectionKind::MuxSelection=>"mux_selection",om::ConnectionKind::LogicExpression=>"logic_expression" } }
