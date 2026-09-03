//! Compact parsed and elaborated ICL models optimized for repeated queries.

mod cache;
mod syntax;

use super::{MuxType, PortType};
use crate::{Error, Result};
use ahash::AHashMap as HashMap;
use glob::Pattern;
use num_bigint::BigInt;

pub use syntax::{ParsedIcl, Parser, SourceSpan, SymbolId, SyntaxId, SyntaxKind};

/// Parse an ICL file with the default compact-model parser options.
pub fn from_file(path: &std::path::Path) -> Result<ParsedIcl> {
    Parser::new().from_file(path)
}

/// Parse ICL source text with the default compact-model parser options.
pub fn from_str(source: &str) -> Result<ParsedIcl> {
    Parser::new().from_str(source)
}

macro_rules! id_type {
    ($name:ident) => {
        #[derive(
            Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize,
        )]
        pub struct $name(u32);

        impl $name {
            pub fn as_usize(self) -> usize {
                self.0 as usize
            }
        }
    };
}

id_type!(ModuleDefId);
id_type!(SpecializationId);
id_type!(InstanceId);
id_type!(PortId);
id_type!(ScanRegisterId);
id_type!(DataRegisterId);
id_type!(AliasId);
id_type!(AliasSegmentId);
id_type!(InternalSignalId);
id_type!(ConnectionId);

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum ParameterValue {
    Integer(BigInt),
    String(String),
    Symbol(SymbolId),
    Bits {
        width: u32,
        value: num_bigint::BigUint,
        unknown: num_bigint::BigUint,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ModuleDef {
    pub name: SymbolId,
    pub namespace: Option<SymbolId>,
    pub qualified_name: String,
    pub syntax: SyntaxId,
    pub instances: Vec<InstanceDefId>,
    pub ports: Vec<PortDefId>,
    pub scan_registers: Vec<ScanRegisterDefId>,
    pub data_registers: Vec<DataRegisterDefId>,
    pub aliases: Vec<AliasDefId>,
    pub internal_signals: Vec<InternalSignalDefId>,
    pub enum_values: Vec<EnumValueDefId>,
    pub parameters: Vec<ParameterDefId>,
}

id_type!(InstanceDefId);
id_type!(PortDefId);
id_type!(ScanRegisterDefId);
id_type!(DataRegisterDefId);
id_type!(AliasDefId);
id_type!(ParameterDefId);
id_type!(InternalSignalDefId);
id_type!(EnumValueDefId);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InstanceDef {
    pub name: SymbolId,
    pub module_type: ModuleReference,
    pub syntax: SyntaxId,
    pub overrides: Vec<ParameterDefId>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ModuleReference {
    pub namespace: Option<SymbolId>,
    pub name: SymbolId,
    pub explicitly_qualified: bool,
    pub use_namespace: Option<SymbolId>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PortDef {
    pub name: SymbolId,
    pub kind: PortType,
    pub syntax: SyntaxId,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ScanRegisterDef {
    pub name: SymbolId,
    pub syntax: SyntaxId,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DataRegisterDef {
    pub name: SymbolId,
    pub syntax: SyntaxId,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AliasDef {
    pub name: SymbolId,
    pub syntax: SyntaxId,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum InternalSignalType {
    Logic,
    Mux(MuxType),
    OneHotScan,
    OneHotData,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InternalSignalDef {
    pub name: SymbolId,
    pub kind: InternalSignalType,
    pub syntax: SyntaxId,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EnumValueDef {
    pub name: SymbolId,
    pub syntax: SyntaxId,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ParameterDef {
    pub name: SymbolId,
    pub syntax: SyntaxId,
    pub local: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Specialization {
    pub module: ModuleDefId,
    pub parameters: Vec<(SymbolId, ParameterValue)>,
    pub ports: Vec<PortId>,
    pub scan_registers: Vec<ScanRegisterId>,
    pub data_registers: Vec<DataRegisterId>,
    pub aliases: Vec<AliasId>,
    pub internal_signals: Vec<InternalSignalId>,
    pub enum_values: Vec<(SymbolId, ParameterValue)>,
    pub connections: Vec<ConnectionId>,
    pub child_specializations: Vec<(InstanceDefId, SpecializationId)>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Instance {
    pub name: SymbolId,
    pub parent: Option<InstanceId>,
    pub specialization: SpecializationId,
    pub children: Vec<InstanceId>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ResolvedPort {
    pub definition: PortDefId,
    pub specialization: SpecializationId,
    pub name: SymbolId,
    pub kind: PortType,
    pub width: u32,
    pub first_index: u32,
    pub last_index: u32,
    pub default_load_value: Option<ParameterValue>,
    pub enum_ref: Option<SymbolId>,
    pub active_polarity: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ResolvedScanRegister {
    pub definition: ScanRegisterDefId,
    pub specialization: SpecializationId,
    pub name: SymbolId,
    pub width: u32,
    pub first_index: u32,
    pub last_index: u32,
    pub default_load_value: Option<ParameterValue>,
    pub reset_value: Option<ParameterValue>,
    pub enum_ref: Option<SymbolId>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ResolvedDataRegister {
    pub definition: DataRegisterDefId,
    pub specialization: SpecializationId,
    pub name: SymbolId,
    pub width: u32,
    pub first_index: u32,
    pub last_index: u32,
    pub default_load_value: Option<ParameterValue>,
    pub reset_value: Option<ParameterValue>,
    pub enum_ref: Option<SymbolId>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ResolvedInternalSignal {
    pub definition: InternalSignalDefId,
    pub specialization: SpecializationId,
    pub name: SymbolId,
    pub kind: InternalSignalType,
    pub width: u32,
    pub first_index: u32,
    pub last_index: u32,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum BitSelection {
    Whole,
    Index(u32),
    Range { first: u32, last: u32 },
}

impl BitSelection {
    pub fn width(self, whole_width: u32) -> u32 {
        match self {
            Self::Whole => whole_width,
            Self::Index(_) => 1,
            Self::Range { first, last } => first.abs_diff(last) + 1,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum AliasEndpoint {
    Port(PortId),
    ScanRegister(ScanRegisterId),
    DataRegister(DataRegisterId),
    InternalSignal(InternalSignalId),
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ConnectionEndpoint {
    Instance(InstanceDefId),
    Port(PortId),
    ScanRegister(ScanRegisterId),
    DataRegister(DataRegisterId),
    Alias(AliasId),
    InternalSignal(InternalSignalId),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ConnectionTarget {
    Object(ConnectionEndpoint),
    Constant(ParameterValue),
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum ConnectionOwner {
    Port(PortId),
    ScanRegister(ScanRegisterId),
    DataRegister(DataRegisterId),
    InternalSignal(InternalSignalId),
    InstanceInput {
        instance: InstanceDefId,
        port: PortId,
    },
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ConnectionKind {
    Source,
    Enable,
    ScanInSource,
    CaptureSource,
    WriteEnSource,
    WriteDataSource,
    ReadDataSource,
    InstanceInput,
    MuxSelect,
    MuxSelection,
    LogicExpression,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConnectionSegment {
    pub relative_instance_path: Vec<SymbolId>,
    pub target: ConnectionTarget,
    pub selection: BitSelection,
    pub inverted: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ResolvedConnection {
    pub specialization: SpecializationId,
    pub owner: ConnectionOwner,
    pub kind: ConnectionKind,
    pub source: SourceSpan,
    pub segments: Vec<ConnectionSegment>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AliasSegment {
    pub relative_instance_path: Vec<SymbolId>,
    pub target: AliasEndpoint,
    pub selection: BitSelection,
    pub inverted: bool,
    pub alias_bit_offset: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ResolvedAlias {
    pub definition: AliasDefId,
    pub specialization: SpecializationId,
    pub name: SymbolId,
    pub width: u32,
    pub first_index: u32,
    pub last_index: u32,
    pub segments: Vec<AliasSegmentId>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct PortHandle {
    pub instance: InstanceId,
    pub port: PortId,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ScanRegisterHandle {
    pub instance: InstanceId,
    pub register: ScanRegisterId,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct DataRegisterHandle {
    pub instance: InstanceId,
    pub register: DataRegisterId,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct AliasHandle {
    pub instance: InstanceId,
    pub alias: AliasId,
}

#[derive(Clone, Copy, Debug)]
pub struct AliasBit<'a> {
    pub relative_instance_path: &'a [SymbolId],
    pub target: AliasEndpoint,
    pub target_index: u32,
    pub inverted: bool,
    pub alias_bit_offset: u32,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum RegisterHandle {
    Scan(ScanRegisterHandle),
    Data(DataRegisterHandle),
}

#[derive(Debug)]
pub struct IclModel {
    parsed: ParsedIcl,
    modules: Vec<ModuleDef>,
    instances_def: Vec<InstanceDef>,
    ports_def: Vec<PortDef>,
    scan_registers_def: Vec<ScanRegisterDef>,
    data_registers_def: Vec<DataRegisterDef>,
    aliases_def: Vec<AliasDef>,
    internal_signals_def: Vec<InternalSignalDef>,
    enum_values_def: Vec<EnumValueDef>,
    parameters_def: Vec<ParameterDef>,
    specializations: Vec<Specialization>,
    instances: Vec<Instance>,
    ports: Vec<ResolvedPort>,
    scan_registers: Vec<ResolvedScanRegister>,
    data_registers: Vec<ResolvedDataRegister>,
    aliases: Vec<ResolvedAlias>,
    internal_signals: Vec<ResolvedInternalSignal>,
    alias_segments: Vec<AliasSegment>,
    connections: Vec<ResolvedConnection>,
    connections_by_owner: HashMap<ConnectionOwner, Vec<ConnectionId>>,
    root: InstanceId,
    module_by_name: HashMap<SymbolId, ModuleDefId>,
    child_index: HashMap<(InstanceId, SymbolId), InstanceId>,
    instances_by_name: HashMap<SymbolId, Vec<InstanceId>>,
    instances_by_type: HashMap<ModuleDefId, Vec<InstanceId>>,
    ports_by_name: HashMap<SymbolId, Vec<PortHandle>>,
    scan_registers_by_name: HashMap<SymbolId, Vec<ScanRegisterHandle>>,
    data_registers_by_name: HashMap<SymbolId, Vec<DataRegisterHandle>>,
    aliases_by_name: HashMap<SymbolId, Vec<AliasHandle>>,
}

impl ParsedIcl {
    pub fn elaborate(&self, top: &str) -> Result<IclModel> {
        IclModel::build(self.clone(), Some(top))
    }

    pub fn elaborate_unique_root(&self) -> Result<IclModel> {
        IclModel::build(self.clone(), None)
    }
}

impl IclModel {
    fn build(parsed: ParsedIcl, top: Option<&str>) -> Result<Self> {
        elaborate::build(parsed, top)
    }

    pub fn root(&self) -> InstanceId {
        self.root
    }

    pub fn modules(&self) -> &[ModuleDef] {
        &self.modules
    }

    pub fn specializations(&self) -> &[Specialization] {
        &self.specializations
    }

    pub fn instances(&self) -> &[Instance] {
        &self.instances
    }

    pub fn connections(&self) -> &[ResolvedConnection] {
        &self.connections
    }

    pub fn connection(&self, id: ConnectionId) -> &ResolvedConnection {
        &self.connections[id.as_usize()]
    }

    pub fn connections_for(
        &self,
        owner: ConnectionOwner,
    ) -> impl Iterator<Item = &ResolvedConnection> {
        self.connections_by_owner
            .get(&owner)
            .into_iter()
            .flatten()
            .map(|id| &self.connections[id.as_usize()])
    }

    pub fn parsed(&self) -> &ParsedIcl {
        &self.parsed
    }

    pub fn module_definition(&self, name: &str) -> Option<&ModuleDef> {
        let symbol = self.parsed.symbol_id(name)?;
        let id = self.module_by_name.get(&symbol)?;
        self.modules.get(id.as_usize())
    }

    pub fn port(&self, handle: PortHandle) -> &ResolvedPort {
        &self.ports[handle.port.as_usize()]
    }

    pub fn port_path(&self, handle: PortHandle) -> String {
        self.object_path(handle.instance, self.port(handle).name)
    }

    pub fn scan_register(&self, handle: ScanRegisterHandle) -> &ResolvedScanRegister {
        &self.scan_registers[handle.register.as_usize()]
    }

    pub fn scan_register_path(&self, handle: ScanRegisterHandle) -> String {
        self.object_path(handle.instance, self.scan_register(handle).name)
    }

    pub fn data_register(&self, handle: DataRegisterHandle) -> &ResolvedDataRegister {
        &self.data_registers[handle.register.as_usize()]
    }

    pub fn data_register_path(&self, handle: DataRegisterHandle) -> String {
        self.object_path(handle.instance, self.data_register(handle).name)
    }

    pub fn alias(&self, handle: AliasHandle) -> &ResolvedAlias {
        &self.aliases[handle.alias.as_usize()]
    }

    pub fn alias_path(&self, handle: AliasHandle) -> String {
        self.object_path(handle.instance, self.alias(handle).name)
    }

    pub fn alias_segments(&self, handle: AliasHandle) -> impl Iterator<Item = &AliasSegment> {
        self.alias(handle)
            .segments
            .iter()
            .map(|id| &self.alias_segments[id.as_usize()])
    }

    pub fn alias_bits(&self, handle: AliasHandle) -> AliasBits<'_> {
        AliasBits {
            model: self,
            segment_ids: self.alias(handle).segments.iter(),
            current: None,
        }
    }

    fn endpoint_shape(&self, endpoint: AliasEndpoint) -> (u32, u32, u32) {
        match endpoint {
            AliasEndpoint::Port(id) => {
                let value = &self.ports[id.as_usize()];
                (value.first_index, value.last_index, value.width)
            }
            AliasEndpoint::ScanRegister(id) => {
                let value = &self.scan_registers[id.as_usize()];
                (value.first_index, value.last_index, value.width)
            }
            AliasEndpoint::DataRegister(id) => {
                let value = &self.data_registers[id.as_usize()];
                (value.first_index, value.last_index, value.width)
            }
            AliasEndpoint::InternalSignal(id) => {
                let value = &self.internal_signals[id.as_usize()];
                (value.first_index, value.last_index, value.width)
            }
        }
    }

    pub fn resolve_path(&self, path: &str) -> Result<InstanceId> {
        let mut current = self.root;
        let mut parts = path.split('.').filter(|part| !part.is_empty()).peekable();
        if let Some(first) = parts.peek().copied() {
            if self.instance_name(current) == first {
                parts.next();
            }
        }
        for part in parts {
            let symbol = self
                .parsed
                .symbol_id(part)
                .ok_or_else(|| Error::new(&format!("Unknown hierarchy component: {part}")))?;
            current = *self.child_index.get(&(current, symbol)).ok_or_else(|| {
                Error::new(&format!(
                    "No child named {part} below the requested hierarchy"
                ))
            })?;
        }
        Ok(current)
    }

    pub fn resolve_relative_path(
        &self,
        mut instance: InstanceId,
        path: &[SymbolId],
    ) -> Result<InstanceId> {
        for component in path {
            instance = *self
                .child_index
                .get(&(instance, *component))
                .ok_or_else(|| {
                    Error::new(&format!(
                        "No child named {} below the requested hierarchy",
                        self.parsed.symbol(*component)
                    ))
                })?;
        }
        Ok(instance)
    }

    pub fn scope(&self, instance: InstanceId) -> ScopeView<'_> {
        ScopeView {
            model: self,
            instance,
        }
    }

    pub fn instance_name(&self, id: InstanceId) -> &str {
        self.parsed.symbol(self.instances[id.as_usize()].name)
    }

    pub fn instance_module(&self, id: InstanceId) -> &ModuleDef {
        let specialization = self.instances[id.as_usize()].specialization;
        let module = self.specializations[specialization.as_usize()].module;
        &self.modules[module.as_usize()]
    }

    pub fn instance_path(&self, id: InstanceId) -> String {
        let mut names = Vec::new();
        let mut current = Some(id);
        while let Some(instance) = current {
            let record = &self.instances[instance.as_usize()];
            names.push(self.parsed.symbol(record.name));
            current = record.parent;
        }
        names.reverse();
        names.join(".")
    }

    fn object_path(&self, instance: InstanceId, name: SymbolId) -> String {
        format!(
            "{}.{}",
            self.instance_path(instance),
            self.parsed.symbol(name)
        )
    }

    pub fn find_instances(&self, pattern: &str) -> Result<Vec<InstanceId>> {
        find_symbols(&self.parsed, pattern, &self.instances_by_name)
    }

    pub fn find_instances_of(&self, pattern: &str) -> Result<Vec<InstanceId>> {
        let glob = compile_pattern(pattern)?;
        let exact = !has_glob(pattern);
        let mut results = Vec::new();
        for (module_id, instances) in &self.instances_by_type {
            let module = &self.modules[module_id.as_usize()];
            let unqualified = self.parsed.symbol(module.name);
            let matched = if exact {
                module.qualified_name == pattern || unqualified == pattern
            } else {
                glob.as_ref().is_some_and(|glob| {
                    glob.matches(&module.qualified_name) || glob.matches(unqualified)
                })
            };
            if matched {
                results.extend_from_slice(instances);
            }
        }
        results.sort_unstable();
        results.dedup();
        Ok(results)
    }

    pub fn find_ports(&self, pattern: &str) -> Result<Vec<PortHandle>> {
        find_symbols(&self.parsed, pattern, &self.ports_by_name)
    }

    pub fn find_scan_registers(&self, pattern: &str) -> Result<Vec<ScanRegisterHandle>> {
        find_symbols(&self.parsed, pattern, &self.scan_registers_by_name)
    }

    pub fn find_data_registers(&self, pattern: &str) -> Result<Vec<DataRegisterHandle>> {
        find_symbols(&self.parsed, pattern, &self.data_registers_by_name)
    }

    pub fn find_registers(&self, pattern: &str) -> Result<Vec<RegisterHandle>> {
        let mut results: Vec<_> = self
            .find_scan_registers(pattern)?
            .into_iter()
            .map(RegisterHandle::Scan)
            .collect();
        results.extend(
            self.find_data_registers(pattern)?
                .into_iter()
                .map(RegisterHandle::Data),
        );
        results.sort_by_key(|result| match result {
            RegisterHandle::Scan(handle) => (handle.instance, 0, handle.register.0),
            RegisterHandle::Data(handle) => (handle.instance, 1, handle.register.0),
        });
        Ok(results)
    }

    pub fn find_aliases(&self, pattern: &str) -> Result<Vec<AliasHandle>> {
        find_symbols(&self.parsed, pattern, &self.aliases_by_name)
    }
}

pub struct ScopeView<'a> {
    model: &'a IclModel,
    instance: InstanceId,
}

pub struct AliasBits<'a> {
    model: &'a IclModel,
    segment_ids: std::slice::Iter<'a, AliasSegmentId>,
    current: Option<(&'a AliasSegment, u32)>,
}

impl<'a> Iterator for AliasBits<'a> {
    type Item = AliasBit<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some((segment, position)) = self.current {
                let (object_first, object_last, object_width) =
                    self.model.endpoint_shape(segment.target);
                let width = segment.selection.width(object_width);
                if position < width {
                    let (first, last) = match segment.selection {
                        BitSelection::Whole => (object_first, object_last),
                        BitSelection::Index(index) => (index, index),
                        BitSelection::Range { first, last } => (first, last),
                    };
                    let target_index = if first >= last {
                        first - position
                    } else {
                        first + position
                    };
                    self.current = Some((segment, position + 1));
                    return Some(AliasBit {
                        relative_instance_path: &segment.relative_instance_path,
                        target: segment.target,
                        target_index,
                        inverted: segment.inverted,
                        alias_bit_offset: segment.alias_bit_offset + width - position - 1,
                    });
                }
                self.current = None;
            }
            let id = self.segment_ids.next()?;
            self.current = Some((&self.model.alias_segments[id.as_usize()], 0));
        }
    }
}

impl<'a> ScopeView<'a> {
    pub fn instance(&self) -> &'a Instance {
        &self.model.instances[self.instance.as_usize()]
    }

    pub fn child_instances(&self) -> impl Iterator<Item = InstanceId> + '_ {
        self.instance().children.iter().copied()
    }

    pub fn ports(&self) -> impl Iterator<Item = PortHandle> + '_ {
        let specialization = &self.model.specializations[self.instance().specialization.as_usize()];
        specialization.ports.iter().copied().map(|port| PortHandle {
            instance: self.instance,
            port,
        })
    }

    pub fn scan_registers(&self) -> impl Iterator<Item = ScanRegisterHandle> + '_ {
        let specialization = &self.model.specializations[self.instance().specialization.as_usize()];
        specialization
            .scan_registers
            .iter()
            .copied()
            .map(|register| ScanRegisterHandle {
                instance: self.instance,
                register,
            })
    }

    pub fn data_registers(&self) -> impl Iterator<Item = DataRegisterHandle> + '_ {
        let specialization = &self.model.specializations[self.instance().specialization.as_usize()];
        specialization
            .data_registers
            .iter()
            .copied()
            .map(|register| DataRegisterHandle {
                instance: self.instance,
                register,
            })
    }

    pub fn registers(&self) -> impl Iterator<Item = RegisterHandle> + '_ {
        self.scan_registers()
            .map(RegisterHandle::Scan)
            .chain(self.data_registers().map(RegisterHandle::Data))
    }

    pub fn aliases(&self) -> impl Iterator<Item = AliasHandle> + '_ {
        let specialization = &self.model.specializations[self.instance().specialization.as_usize()];
        specialization
            .aliases
            .iter()
            .copied()
            .map(|alias| AliasHandle {
                instance: self.instance,
                alias,
            })
    }

    pub fn find_child_instances(&self, pattern: &str) -> Result<Vec<InstanceId>> {
        filter_pattern(self.model, pattern, self.child_instances(), |model, id| {
            model.instance_name(id)
        })
    }

    pub fn find_child_instances_of(&self, pattern: &str) -> Result<Vec<InstanceId>> {
        let glob = compile_pattern(pattern)?;
        Ok(self
            .child_instances()
            .filter(|id| {
                let module = self.model.instance_module(*id);
                let unqualified = self.model.parsed.symbol(module.name);
                glob.as_ref()
                    .map(|glob| glob.matches(&module.qualified_name) || glob.matches(unqualified))
                    .unwrap_or(module.qualified_name == pattern || unqualified == pattern)
            })
            .collect())
    }

    pub fn find_ports(&self, pattern: &str) -> Result<Vec<PortHandle>> {
        filter_pattern(self.model, pattern, self.ports(), |model, handle| {
            model
                .parsed
                .symbol(model.ports[handle.port.as_usize()].name)
        })
    }

    pub fn find_scan_registers(&self, pattern: &str) -> Result<Vec<ScanRegisterHandle>> {
        filter_pattern(
            self.model,
            pattern,
            self.scan_registers(),
            |model, handle| {
                model
                    .parsed
                    .symbol(model.scan_registers[handle.register.as_usize()].name)
            },
        )
    }

    pub fn find_data_registers(&self, pattern: &str) -> Result<Vec<DataRegisterHandle>> {
        filter_pattern(
            self.model,
            pattern,
            self.data_registers(),
            |model, handle| {
                model
                    .parsed
                    .symbol(model.data_registers[handle.register.as_usize()].name)
            },
        )
    }

    pub fn find_registers(&self, pattern: &str) -> Result<Vec<RegisterHandle>> {
        filter_pattern(
            self.model,
            pattern,
            self.registers(),
            |model, handle| match handle {
                RegisterHandle::Scan(handle) => model
                    .parsed
                    .symbol(model.scan_registers[handle.register.as_usize()].name),
                RegisterHandle::Data(handle) => model
                    .parsed
                    .symbol(model.data_registers[handle.register.as_usize()].name),
            },
        )
    }

    pub fn find_aliases(&self, pattern: &str) -> Result<Vec<AliasHandle>> {
        filter_pattern(self.model, pattern, self.aliases(), |model, handle| {
            model
                .parsed
                .symbol(model.aliases[handle.alias.as_usize()].name)
        })
    }
}

fn find_symbols<T: Copy + Ord>(
    parsed: &ParsedIcl,
    pattern: &str,
    index: &HashMap<SymbolId, Vec<T>>,
) -> Result<Vec<T>> {
    if !has_glob(pattern) {
        return Ok(parsed
            .symbol_id(pattern)
            .and_then(|symbol| index.get(&symbol))
            .cloned()
            .unwrap_or_default());
    }

    let glob = compile_pattern(pattern)?.unwrap();
    let mut results = Vec::new();
    for (symbol, matches) in index {
        if glob.matches(parsed.symbol(*symbol)) {
            results.extend_from_slice(matches);
        }
    }
    results.sort_unstable();
    Ok(results)
}

fn filter_pattern<T: Copy>(
    model: &IclModel,
    pattern: &str,
    values: impl Iterator<Item = T>,
    name: impl Fn(&IclModel, T) -> &str,
) -> Result<Vec<T>> {
    let glob = compile_pattern(pattern)?;
    Ok(values
        .filter(|value| {
            let candidate = name(model, *value);
            glob.as_ref()
                .map(|glob| glob.matches(candidate))
                .unwrap_or(candidate == pattern)
        })
        .collect())
}

fn has_glob(pattern: &str) -> bool {
    pattern
        .bytes()
        .any(|c| matches!(c, b'*' | b'?' | b'[' | b'\\'))
}

fn compile_pattern(pattern: &str) -> Result<Option<Pattern>> {
    if !has_glob(pattern) {
        return Ok(None);
    }
    Pattern::new(pattern)
        .map(Some)
        .map_err(|e| Error::new(&format!("Invalid name pattern {pattern:?}: {e}")))
}

mod elaborate;

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_design() -> &'static str {
        r#"
            Module Leaf {
                Parameter WIDTH = 4;
                ScanInPort data[$WIDTH-1:0];
                DataOutPort result[$WIDTH-1:0];
                ScanRegister status[$WIDTH-1:0] { ScanInSource data[0]; }
                DataRegister config[$WIDTH-1:0] { ResetValue 4'b0000; }
                Alias sparse[2:0] = status[3], status[1], config[0];
                Alias nested[1:0] = sparse[2], sparse[0];
            }
            Module Top {
                ScanInPort top_port;
                Instance left Of Leaf { Parameter WIDTH = 4; InputPort data = top_port; }
                Instance right Of Leaf { Parameter WIDTH = 4; InputPort data = top_port; }
                Alias cross[1:0] = left.nested[1], right.config[0];
            }
        "#
    }

    #[test]
    fn compact_parser_and_elaboration_build_searchable_model() {
        let parsed = Parser::new().from_str(dummy_design()).unwrap();
        assert!(parsed.node_count() > 0);

        let model = parsed.elaborate_unique_root().unwrap();
        assert_eq!(model.instances.len(), 3);
        assert_eq!(model.specializations.len(), 2);
        assert_eq!(model.find_instances("*").unwrap().len(), 3);
        assert_eq!(model.find_instances("l?ft").unwrap().len(), 1);
        assert_eq!(model.find_instances_of("Leaf").unwrap().len(), 2);
        assert_eq!(model.find_instances_of("L*").unwrap().len(), 2);
        assert_eq!(model.find_ports("data").unwrap().len(), 2);
        assert_eq!(model.find_ports("*port").unwrap().len(), 1);
        assert_eq!(model.find_ports("[dr]*").unwrap().len(), 4);
        assert_eq!(model.find_scan_registers("status").unwrap().len(), 2);
        assert_eq!(model.find_data_registers("config").unwrap().len(), 2);
        assert_eq!(model.find_registers("*").unwrap().len(), 4);
        assert_eq!(model.find_aliases("sp*").unwrap().len(), 2);
        assert_eq!(model.find_aliases("*").unwrap().len(), 5);
        assert!(model.connections().iter().any(|connection| {
            connection.kind == ConnectionKind::ScanInSource
                && matches!(
                    connection.segments.first().map(|segment| &segment.target),
                    Some(ConnectionTarget::Object(ConnectionEndpoint::Port(_)))
                )
        }));
        let status = model.find_scan_registers("status").unwrap()[0];
        assert!(model
            .connections_for(ConnectionOwner::ScanRegister(status.register))
            .any(|connection| connection.kind == ConnectionKind::ScanInSource));

        let left = model.resolve_path("Top.left").unwrap();
        assert_eq!(model.instance_name(left), "left");
        assert_eq!(model.instance_path(left), "Top.left");
        assert_eq!(model.scope(left).ports().count(), 2);
        assert_eq!(model.scope(left).scan_registers().count(), 1);
        assert_eq!(model.scope(left).data_registers().count(), 1);
        assert_eq!(model.scope(left).registers().count(), 2);
        assert_eq!(model.scope(left).find_ports("d*").unwrap().len(), 1);
        assert_eq!(model.scope(left).find_registers("*").unwrap().len(), 2);

        let alias = model.scope(left).aliases().next().unwrap();
        let resolved = model.alias(alias);
        assert_eq!(resolved.width, 3);
        assert_eq!(model.alias_segments(alias).count(), 3);
        assert_eq!(
            model
                .alias_segments(alias)
                .map(|segment| segment.alias_bit_offset)
                .collect::<Vec<_>>(),
            vec![2, 1, 0]
        );

        let cross = model.find_aliases("cross").unwrap()[0];
        assert_eq!(model.alias(cross).width, 2);
        assert_eq!(model.alias_segments(cross).count(), 2);
        let bits: Vec<_> = model.alias_bits(cross).collect();
        assert_eq!(
            bits.iter().map(|bit| bit.target_index).collect::<Vec<_>>(),
            vec![3, 0]
        );
        assert_eq!(
            bits.iter()
                .map(|bit| bit.alias_bit_offset)
                .collect::<Vec<_>>(),
            vec![1, 0]
        );
        assert_eq!(
            model.parsed().symbol(bits[0].relative_instance_path[0]),
            "left"
        );
        assert_eq!(
            model.parsed().symbol(bits[1].relative_instance_path[0]),
            "right"
        );
    }

    #[test]
    fn compact_comments_are_optional() {
        let source = "// root\nModule Top { /* body */ ScanInPort input; }";
        let without = Parser::new().from_str(source).unwrap();
        let with = Parser::new().preserve_comments().from_str(source).unwrap();
        let without_comments = without
            .syntax_ids()
            .filter(|id| without.kind(*id) == SyntaxKind::Comment)
            .count();
        let with_comments = with
            .syntax_ids()
            .filter(|id| with.kind(*id) == SyntaxKind::Comment)
            .count();
        assert_eq!(without_comments, 0);
        assert_eq!(with_comments, 2);
    }

    #[test]
    fn parameter_specializations_are_shared_and_resolve_widths() {
        let source = r#"
            Module Leaf {
                Parameter WIDTH = $BASE*2;
                Parameter BASE = 2;
                LocalParameter LAST = $WIDTH-1;
                DataOutPort value[$LAST:0];
            }
            Module Top {
                Instance first Of Leaf { Parameter BASE = 3; }
                Instance second Of Leaf { Parameter BASE = 3; }
                Instance third Of Leaf { Parameter BASE = 4; }
            }
        "#;
        let parsed = Parser::new().from_str(source).unwrap();
        let model = parsed.elaborate_unique_root().unwrap();
        assert_eq!(model.specializations.len(), 3);

        for (path, width) in [("Top.first", 6), ("Top.second", 6), ("Top.third", 8)] {
            let instance = model.resolve_path(path).unwrap();
            let port = model.scope(instance).ports().next().unwrap();
            assert_eq!(model.port(port).width, width);
        }
    }

    #[test]
    fn namespaces_and_type_globs_are_resolved_consistently() {
        let source = r#"
            NameSpace Library;
            Module Leaf { ScanInPort input; }
            NameSpace;
            UseNameSpace Library;
            Module Top {
                Instance implicit Of Leaf;
                Instance explicit Of Library::Leaf;
            }
        "#;
        let parsed = Parser::new().from_str(source).unwrap();
        let model = parsed.elaborate_unique_root().unwrap();
        assert_eq!(model.find_instances_of("Leaf").unwrap().len(), 2);
        assert_eq!(model.find_instances_of("Library::Leaf").unwrap().len(), 2);
        assert_eq!(model.find_instances_of("*::L*").unwrap().len(), 2);
        assert_eq!(
            model
                .scope(model.root())
                .find_child_instances_of("L*")
                .unwrap()
                .len(),
            2
        );
    }

    #[test]
    fn elaboration_reports_parameter_hierarchy_and_alias_cycles() {
        for source in [
            "Module Top { Parameter A = $B; Parameter B = $A; }",
            "Module Top { Alias first = second; Alias second = first; }",
        ] {
            let parsed = Parser::new().from_str(source).unwrap();
            assert!(parsed.elaborate_unique_root().is_err());
        }

        let parsed = Parser::new()
            .from_str("Module Top { Instance self_ref Of Top; }")
            .unwrap();
        assert!(parsed.elaborate("Top").is_err());

        let parsed = Parser::new()
            .from_str("Module First {} Module Second {}")
            .unwrap();
        assert!(parsed.elaborate_unique_root().is_err());
        assert!(parsed.elaborate("Second").is_ok());
    }

    #[test]
    fn all_finders_reject_invalid_globs() {
        let parsed = Parser::new().from_str(dummy_design()).unwrap();
        let model = parsed.elaborate_unique_root().unwrap();
        assert!(model.find_instances("[").is_err());
        assert!(model.find_instances_of("[").is_err());
        assert!(model.find_ports("[").is_err());
        assert!(model.find_scan_registers("[").is_err());
        assert!(model.find_data_registers("[").is_err());
        assert!(model.find_registers("[").is_err());
        assert!(model.find_aliases("[").is_err());
    }

    #[test]
    fn parallel_and_sequential_parsing_are_equivalent() {
        let padding = "x".repeat(4_096);
        let mut source = String::new();
        for index in 0..256 {
            source.push_str(&format!(
                "/* {padding} */ Module Dummy_{index} {{ ScanInPort input; }}\n"
            ));
        }
        let sequential = Parser::new().threads(1).from_str(&source).unwrap();
        let parallel = Parser::new().threads(4).from_str(&source).unwrap();
        assert_eq!(sequential.node_count(), parallel.node_count());
        for (left, right) in sequential.syntax_ids().zip(parallel.syntax_ids()) {
            assert_eq!(sequential.kind(left), parallel.kind(right));
            assert_eq!(sequential.node_text(left), parallel.node_text(right));
        }
    }

    #[test]
    fn binary_cache_loads_and_invalidates() {
        let directory = tempfile::tempdir().unwrap();
        let source_path = directory.path().join("dummy.icl");
        let cache_path = directory.path().join("dummy.icl.cache");
        std::fs::write(&source_path, dummy_design()).unwrap();

        let parser = Parser::new().threads(1);
        let first = parser
            .load_or_elaborate(&source_path, None, &cache_path)
            .unwrap();
        assert!(cache_path.is_file());
        assert_eq!(first.find_instances("*").unwrap().len(), 3);

        let cached = parser
            .load_or_elaborate(&source_path, None, &cache_path)
            .unwrap();
        assert_eq!(cached.find_registers("*").unwrap().len(), 4);

        std::fs::write(
            &source_path,
            "Module Changed { ScanInPort first; ScanOutPort second; }",
        )
        .unwrap();
        let rebuilt = parser
            .load_or_elaborate(&source_path, None, &cache_path)
            .unwrap();
        assert_eq!(rebuilt.find_ports("*").unwrap().len(), 2);

        std::fs::write(&cache_path, b"not a valid model cache").unwrap();
        let recovered = parser
            .load_or_elaborate(&source_path, None, &cache_path)
            .unwrap();
        assert_eq!(recovered.find_ports("*").unwrap().len(), 2);
    }
}
