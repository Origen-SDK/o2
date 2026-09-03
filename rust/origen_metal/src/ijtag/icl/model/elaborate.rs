use super::*;
use crate::ijtag::icl::SignalType;
use ahash::AHashSet as HashSet;
use num_bigint::{BigInt, BigUint, Sign};
use num_traits::{Num, One, ToPrimitive, Zero};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct ModuleKey {
    namespace: Option<SymbolId>,
    name: SymbolId,
}

struct Elaborator {
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
    module_keys: HashMap<ModuleKey, ModuleDefId>,
    specializations: Vec<Specialization>,
    specialization_cache: HashMap<(ModuleDefId, Vec<(SymbolId, ParameterValue)>), SpecializationId>,
    ports: Vec<ResolvedPort>,
    scan_registers: Vec<ResolvedScanRegister>,
    data_registers: Vec<ResolvedDataRegister>,
    aliases: Vec<ResolvedAlias>,
    internal_signals: Vec<ResolvedInternalSignal>,
    alias_segments: Vec<AliasSegment>,
    connections: Vec<ResolvedConnection>,
}

pub(super) fn build(parsed: ParsedIcl, top: Option<&str>) -> Result<IclModel> {
    let mut elaborator = Elaborator::new(parsed);
    elaborator.extract_definitions()?;
    let top_module = elaborator.select_top(top)?;
    let root_specialization =
        elaborator.ensure_specialization(top_module, HashMap::new(), &mut HashSet::new())?;
    elaborator.resolve_aliases()?;
    elaborator.resolve_connections()?;

    let root_name = elaborator.modules[top_module.as_usize()].name;
    let mut instances = Vec::new();
    build_occurrences(
        &elaborator,
        root_specialization,
        root_name,
        None,
        &mut instances,
    );
    let root = InstanceId(0);

    let mut child_index = HashMap::new();
    let mut instances_by_name: HashMap<SymbolId, Vec<InstanceId>> = HashMap::new();
    let mut instances_by_type: HashMap<ModuleDefId, Vec<InstanceId>> = HashMap::new();
    let mut ports_by_name: HashMap<SymbolId, Vec<PortHandle>> = HashMap::new();
    let mut scan_registers_by_name: HashMap<SymbolId, Vec<ScanRegisterHandle>> = HashMap::new();
    let mut data_registers_by_name: HashMap<SymbolId, Vec<DataRegisterHandle>> = HashMap::new();
    let mut aliases_by_name: HashMap<SymbolId, Vec<AliasHandle>> = HashMap::new();
    let mut connections_by_owner: HashMap<ConnectionOwner, Vec<ConnectionId>> = HashMap::new();
    for (index, connection) in elaborator.connections.iter().enumerate() {
        connections_by_owner
            .entry(connection.owner)
            .or_default()
            .push(ConnectionId(index as u32));
    }

    for (index, instance) in instances.iter().enumerate() {
        let instance_id = InstanceId(index as u32);
        instances_by_name
            .entry(instance.name)
            .or_default()
            .push(instance_id);
        let specialization = &elaborator.specializations[instance.specialization.as_usize()];
        instances_by_type
            .entry(specialization.module)
            .or_default()
            .push(instance_id);
        if let Some(parent) = instance.parent {
            if child_index
                .insert((parent, instance.name), instance_id)
                .is_some()
            {
                return Err(Error::new(
                    "Duplicate child instance name in elaborated hierarchy",
                ));
            }
        }
        for port in &specialization.ports {
            let resolved = &elaborator.ports[port.as_usize()];
            ports_by_name
                .entry(resolved.name)
                .or_default()
                .push(PortHandle {
                    instance: instance_id,
                    port: *port,
                });
        }
        for register in &specialization.scan_registers {
            let resolved = &elaborator.scan_registers[register.as_usize()];
            scan_registers_by_name
                .entry(resolved.name)
                .or_default()
                .push(ScanRegisterHandle {
                    instance: instance_id,
                    register: *register,
                });
        }
        for register in &specialization.data_registers {
            let resolved = &elaborator.data_registers[register.as_usize()];
            data_registers_by_name
                .entry(resolved.name)
                .or_default()
                .push(DataRegisterHandle {
                    instance: instance_id,
                    register: *register,
                });
        }
        for alias in &specialization.aliases {
            let resolved = &elaborator.aliases[alias.as_usize()];
            aliases_by_name
                .entry(resolved.name)
                .or_default()
                .push(AliasHandle {
                    instance: instance_id,
                    alias: *alias,
                });
        }
    }

    let mut module_by_name = HashMap::new();
    for (index, module) in elaborator.modules.iter().enumerate() {
        module_by_name
            .entry(module.name)
            .or_insert(ModuleDefId(index as u32));
    }

    Ok(IclModel {
        parsed: elaborator.parsed,
        modules: elaborator.modules,
        instances_def: elaborator.instances_def,
        ports_def: elaborator.ports_def,
        scan_registers_def: elaborator.scan_registers_def,
        data_registers_def: elaborator.data_registers_def,
        aliases_def: elaborator.aliases_def,
        internal_signals_def: elaborator.internal_signals_def,
        enum_values_def: elaborator.enum_values_def,
        parameters_def: elaborator.parameters_def,
        specializations: elaborator.specializations,
        instances,
        ports: elaborator.ports,
        scan_registers: elaborator.scan_registers,
        data_registers: elaborator.data_registers,
        aliases: elaborator.aliases,
        internal_signals: elaborator.internal_signals,
        alias_segments: elaborator.alias_segments,
        connections: elaborator.connections,
        connections_by_owner,
        root,
        module_by_name,
        child_index,
        instances_by_name,
        instances_by_type,
        ports_by_name,
        scan_registers_by_name,
        data_registers_by_name,
        aliases_by_name,
    })
}

impl Elaborator {
    fn new(parsed: ParsedIcl) -> Self {
        Self {
            parsed,
            modules: Vec::new(),
            instances_def: Vec::new(),
            ports_def: Vec::new(),
            scan_registers_def: Vec::new(),
            data_registers_def: Vec::new(),
            aliases_def: Vec::new(),
            internal_signals_def: Vec::new(),
            enum_values_def: Vec::new(),
            parameters_def: Vec::new(),
            module_keys: HashMap::new(),
            specializations: Vec::new(),
            specialization_cache: HashMap::new(),
            ports: Vec::new(),
            scan_registers: Vec::new(),
            data_registers: Vec::new(),
            aliases: Vec::new(),
            internal_signals: Vec::new(),
            alias_segments: Vec::new(),
            connections: Vec::new(),
        }
    }

    fn extract_definitions(&mut self) -> Result<()> {
        let mut namespace = None;
        let mut root_use_namespace: Option<Option<SymbolId>> = None;
        let source_items: Vec<_> = self.parsed.children(self.parsed.root()).collect();
        for item in source_items {
            match self.parsed.kind(item) {
                SyntaxKind::NameSpace => namespace = first_direct_symbol(&self.parsed, item),
                SyntaxKind::UseNameSpace => {
                    root_use_namespace = Some(first_direct_symbol(&self.parsed, item))
                }
                SyntaxKind::Module => {
                    self.extract_module(item, namespace, root_use_namespace.unwrap_or(namespace))?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn extract_module(
        &mut self,
        syntax: SyntaxId,
        namespace: Option<SymbolId>,
        inherited_use_namespace: Option<SymbolId>,
    ) -> Result<()> {
        let name = first_direct_symbol(&self.parsed, syntax)
            .ok_or_else(|| Error::new("Module is missing its name"))?;
        let key = ModuleKey { namespace, name };
        if self.module_keys.contains_key(&key) {
            return Err(Error::new(&format!(
                "Duplicate module definition: {}",
                self.parsed.symbol(name)
            )));
        }

        let module_id = ModuleDefId(self.modules.len() as u32);
        self.module_keys.insert(key, module_id);
        let qualified_name = namespace
            .map(|namespace| {
                format!(
                    "{}::{}",
                    self.parsed.symbol(namespace),
                    self.parsed.symbol(name)
                )
            })
            .unwrap_or_else(|| self.parsed.symbol(name).to_string());
        let mut module = ModuleDef {
            name,
            namespace,
            qualified_name,
            syntax,
            instances: Vec::new(),
            ports: Vec::new(),
            scan_registers: Vec::new(),
            data_registers: Vec::new(),
            aliases: Vec::new(),
            internal_signals: Vec::new(),
            enum_values: Vec::new(),
            parameters: Vec::new(),
        };
        let mut use_namespace = inherited_use_namespace;
        let children: Vec<_> = self.parsed.children(syntax).collect();
        for child in children {
            match self.parsed.kind(child) {
                SyntaxKind::UseNameSpace => {
                    use_namespace = first_direct_symbol(&self.parsed, child)
                }
                SyntaxKind::Port(kind) => {
                    let name = declaration_name(&self.parsed, child)?;
                    let id = PortDefId(self.ports_def.len() as u32);
                    self.ports_def.push(PortDef {
                        name,
                        kind,
                        syntax: child,
                    });
                    module.ports.push(id);
                }
                SyntaxKind::Instance => {
                    module
                        .instances
                        .push(self.extract_instance(child, use_namespace)?);
                }
                SyntaxKind::ScanRegister => {
                    let name = declaration_name(&self.parsed, child)?;
                    let id = ScanRegisterDefId(self.scan_registers_def.len() as u32);
                    self.scan_registers_def.push(ScanRegisterDef {
                        name,
                        syntax: child,
                    });
                    module.scan_registers.push(id);
                }
                SyntaxKind::DataRegister => {
                    let name = declaration_name(&self.parsed, child)?;
                    let id = DataRegisterDefId(self.data_registers_def.len() as u32);
                    self.data_registers_def.push(DataRegisterDef {
                        name,
                        syntax: child,
                    });
                    module.data_registers.push(id);
                }
                SyntaxKind::Alias => {
                    let name = declaration_name(&self.parsed, child)?;
                    let id = AliasDefId(self.aliases_def.len() as u32);
                    self.aliases_def.push(AliasDef {
                        name,
                        syntax: child,
                    });
                    module.aliases.push(id);
                }
                SyntaxKind::LogicSignal => {
                    module
                        .internal_signals
                        .push(self.extract_internal_signal(child, InternalSignalType::Logic)?);
                }
                SyntaxKind::Mux(kind) => {
                    module
                        .internal_signals
                        .push(self.extract_internal_signal(child, InternalSignalType::Mux(kind))?);
                }
                SyntaxKind::OneHotScanGroup => {
                    module
                        .internal_signals
                        .push(self.extract_internal_signal(child, InternalSignalType::OneHotScan)?);
                }
                SyntaxKind::Parameter | SyntaxKind::LocalParameter => {
                    module.parameters.push(self.extract_parameter(child)?);
                }
                SyntaxKind::Enumeration => {
                    for item in self.parsed.children(child) {
                        if self.parsed.kind(item) == SyntaxKind::EnumerationItem {
                            if let Some(symbol) = first_direct_symbol(&self.parsed, item) {
                                let id = EnumValueDefId(self.enum_values_def.len() as u32);
                                self.enum_values_def.push(EnumValueDef {
                                    name: symbol,
                                    syntax: item,
                                });
                                module.enum_values.push(id);
                            }
                        }
                    }
                }
                SyntaxKind::OneHotDataGroup => {
                    module
                        .internal_signals
                        .push(self.extract_internal_signal(child, InternalSignalType::OneHotData)?);
                    self.extract_one_hot_items(child, use_namespace, &mut module)?;
                }
                _ => {}
            }
        }
        self.modules.push(module);
        Ok(())
    }

    fn extract_one_hot_items(
        &mut self,
        syntax: SyntaxId,
        use_namespace: Option<SymbolId>,
        module: &mut ModuleDef,
    ) -> Result<()> {
        let children: Vec<_> = self.parsed.children(syntax).collect();
        for child in children {
            match self.parsed.kind(child) {
                SyntaxKind::Instance => {
                    module
                        .instances
                        .push(self.extract_instance(child, use_namespace)?);
                }
                SyntaxKind::DataRegister => {
                    let name = declaration_name(&self.parsed, child)?;
                    let id = DataRegisterDefId(self.data_registers_def.len() as u32);
                    self.data_registers_def.push(DataRegisterDef {
                        name,
                        syntax: child,
                    });
                    module.data_registers.push(id);
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn extract_instance(
        &mut self,
        syntax: SyntaxId,
        use_namespace: Option<SymbolId>,
    ) -> Result<InstanceDefId> {
        let name = first_direct_symbol(&self.parsed, syntax)
            .ok_or_else(|| Error::new("Instance is missing its name"))?;
        let reference_node = self
            .parsed
            .children(syntax)
            .find(|child| self.parsed.kind(*child) == SyntaxKind::ModuleReference)
            .ok_or_else(|| Error::new("Instance is missing its module reference"))?;
        let symbols: Vec<_> = self
            .parsed
            .children(reference_node)
            .filter_map(|child| match self.parsed.kind(child) {
                SyntaxKind::Identifier(symbol) => Some(symbol),
                _ => None,
            })
            .collect();
        let module_name = *symbols
            .last()
            .ok_or_else(|| Error::new("Parameterized instance module names are not supported"))?;
        let reference_text = self.parsed.node_text(reference_node).trim_start();
        let explicitly_qualified = reference_text.contains("::");
        let namespace = if symbols.len() > 1 {
            Some(symbols[0])
        } else {
            None
        };
        let mut overrides = Vec::new();
        let children: Vec<_> = self.parsed.children(syntax).collect();
        for child in children {
            if self.parsed.kind(child) == SyntaxKind::Parameter {
                overrides.push(self.extract_parameter(child)?);
            }
        }
        let id = InstanceDefId(self.instances_def.len() as u32);
        self.instances_def.push(InstanceDef {
            name,
            module_type: ModuleReference {
                namespace,
                name: module_name,
                explicitly_qualified,
                use_namespace,
            },
            syntax,
            overrides,
        });
        Ok(id)
    }

    fn extract_parameter(&mut self, syntax: SyntaxId) -> Result<ParameterDefId> {
        let name = first_direct_symbol(&self.parsed, syntax)
            .ok_or_else(|| Error::new("Parameter is missing its name"))?;
        let id = ParameterDefId(self.parameters_def.len() as u32);
        self.parameters_def.push(ParameterDef {
            name,
            syntax,
            local: self.parsed.kind(syntax) == SyntaxKind::LocalParameter,
        });
        Ok(id)
    }

    fn extract_internal_signal(
        &mut self,
        syntax: SyntaxId,
        kind: InternalSignalType,
    ) -> Result<InternalSignalDefId> {
        let name = declaration_name(&self.parsed, syntax)?;
        let id = InternalSignalDefId(self.internal_signals_def.len() as u32);
        self.internal_signals_def
            .push(InternalSignalDef { name, kind, syntax });
        Ok(id)
    }

    fn resolve_module_reference(&self, reference: ModuleReference) -> Result<ModuleDefId> {
        if reference.explicitly_qualified {
            return self
                .module_keys
                .get(&ModuleKey {
                    namespace: reference.namespace,
                    name: reference.name,
                })
                .copied()
                .ok_or_else(|| self.unknown_module(reference.name));
        }
        if let Some(namespace) = reference.use_namespace {
            if let Some(module) = self.module_keys.get(&ModuleKey {
                namespace: Some(namespace),
                name: reference.name,
            }) {
                return Ok(*module);
            }
        }
        self.module_keys
            .get(&ModuleKey {
                namespace: None,
                name: reference.name,
            })
            .copied()
            .ok_or_else(|| self.unknown_module(reference.name))
    }

    fn unknown_module(&self, name: SymbolId) -> Error {
        Error::new(&format!(
            "Unable to resolve module type {}",
            self.parsed.symbol(name)
        ))
    }

    fn select_top(&self, top: Option<&str>) -> Result<ModuleDefId> {
        if let Some(top) = top {
            let (namespace, name) = if let Some((namespace, name)) = top.rsplit_once("::") {
                (Some(namespace), name)
            } else {
                (None, top)
            };
            let name = self
                .parsed
                .symbol_id(name)
                .ok_or_else(|| Error::new(&format!("Unknown top module: {top}")))?;
            let namespace = namespace
                .map(|value| {
                    self.parsed
                        .symbol_id(value)
                        .ok_or_else(|| Error::new(&format!("Unknown top module: {top}")))
                })
                .transpose()?;
            return self
                .module_keys
                .get(&ModuleKey { namespace, name })
                .copied()
                .ok_or_else(|| Error::new(&format!("Unknown top module: {top}")));
        }

        let mut referenced = HashSet::new();
        for instance in &self.instances_def {
            referenced.insert(self.resolve_module_reference(instance.module_type)?);
        }
        let roots: Vec<_> = (0..self.modules.len())
            .map(|index| ModuleDefId(index as u32))
            .filter(|module| !referenced.contains(module))
            .collect();
        if roots.len() != 1 {
            return Err(Error::new(&format!(
                "Expected exactly one unreferenced root module, found {}",
                roots.len()
            )));
        }
        Ok(roots[0])
    }

    fn ensure_specialization(
        &mut self,
        module_id: ModuleDefId,
        overrides: HashMap<SymbolId, ParameterValue>,
        active: &mut HashSet<ModuleDefId>,
    ) -> Result<SpecializationId> {
        if !active.insert(module_id) {
            return Err(Error::new("Cycle detected in ICL module hierarchy"));
        }
        let module = self.modules[module_id.as_usize()].clone();
        let environment = self.parameter_environment(&module, overrides)?;
        let key_parameters: Vec<_> = module
            .parameters
            .iter()
            .filter_map(|id| {
                let parameter = &self.parameters_def[id.as_usize()];
                (!parameter.local).then(|| {
                    (
                        parameter.name,
                        environment.get(&parameter.name).unwrap().clone(),
                    )
                })
            })
            .collect();
        let key = (module_id, key_parameters.clone());
        if let Some(existing) = self.specialization_cache.get(&key) {
            active.remove(&module_id);
            return Ok(*existing);
        }

        let specialization_id = SpecializationId(self.specializations.len() as u32);
        let resolved_parameters = module
            .parameters
            .iter()
            .map(|id| {
                let parameter = &self.parameters_def[id.as_usize()];
                (
                    parameter.name,
                    environment.get(&parameter.name).unwrap().clone(),
                )
            })
            .collect();
        self.specializations.push(Specialization {
            module: module_id,
            parameters: resolved_parameters,
            ports: Vec::new(),
            scan_registers: Vec::new(),
            data_registers: Vec::new(),
            aliases: Vec::new(),
            internal_signals: Vec::new(),
            enum_values: Vec::new(),
            connections: Vec::new(),
            child_specializations: Vec::new(),
        });
        self.specialization_cache.insert(key, specialization_id);

        let mut ports = Vec::with_capacity(module.ports.len());
        for definition in &module.ports {
            let parsed = &self.ports_def[definition.as_usize()];
            let (first_index, last_index, width) =
                resolved_shape(&self.parsed, parsed.syntax, &environment)?;
            let id = PortId(self.ports.len() as u32);
            self.ports.push(ResolvedPort {
                definition: *definition,
                specialization: specialization_id,
                name: parsed.name,
                kind: parsed.kind,
                width,
                first_index,
                last_index,
                default_load_value: resolved_property_value(
                    &self.parsed,
                    parsed.syntax,
                    SyntaxKind::DefaultLoadValue,
                    &environment,
                )?,
                enum_ref: property_symbol(&self.parsed, parsed.syntax, SyntaxKind::RefEnum),
                active_polarity: active_polarity(&self.parsed, parsed.syntax),
            });
            ports.push(id);
        }
        let mut scan_registers = Vec::with_capacity(module.scan_registers.len());
        for definition in &module.scan_registers {
            let parsed = &self.scan_registers_def[definition.as_usize()];
            let (first_index, last_index, width) =
                resolved_shape(&self.parsed, parsed.syntax, &environment)?;
            let id = ScanRegisterId(self.scan_registers.len() as u32);
            self.scan_registers.push(ResolvedScanRegister {
                definition: *definition,
                specialization: specialization_id,
                name: parsed.name,
                width,
                first_index,
                last_index,
                default_load_value: resolved_property_value(
                    &self.parsed,
                    parsed.syntax,
                    SyntaxKind::DefaultLoadValue,
                    &environment,
                )?,
                reset_value: resolved_property_value(
                    &self.parsed,
                    parsed.syntax,
                    SyntaxKind::ResetValue,
                    &environment,
                )?,
                enum_ref: property_symbol(&self.parsed, parsed.syntax, SyntaxKind::RefEnum),
            });
            scan_registers.push(id);
        }
        let mut data_registers = Vec::with_capacity(module.data_registers.len());
        for definition in &module.data_registers {
            let parsed = &self.data_registers_def[definition.as_usize()];
            let (first_index, last_index, width) =
                resolved_shape(&self.parsed, parsed.syntax, &environment)?;
            let id = DataRegisterId(self.data_registers.len() as u32);
            self.data_registers.push(ResolvedDataRegister {
                definition: *definition,
                specialization: specialization_id,
                name: parsed.name,
                width,
                first_index,
                last_index,
                default_load_value: resolved_property_value(
                    &self.parsed,
                    parsed.syntax,
                    SyntaxKind::DefaultLoadValue,
                    &environment,
                )?,
                reset_value: resolved_property_value(
                    &self.parsed,
                    parsed.syntax,
                    SyntaxKind::ResetValue,
                    &environment,
                )?,
                enum_ref: property_symbol(&self.parsed, parsed.syntax, SyntaxKind::RefEnum),
            });
            data_registers.push(id);
        }
        let mut aliases = Vec::with_capacity(module.aliases.len());
        for definition in &module.aliases {
            let parsed = &self.aliases_def[definition.as_usize()];
            let id = AliasId(self.aliases.len() as u32);
            self.aliases.push(ResolvedAlias {
                definition: *definition,
                specialization: specialization_id,
                name: parsed.name,
                width: 0,
                first_index: 0,
                last_index: 0,
                segments: Vec::new(),
            });
            aliases.push(id);
        }
        let mut enum_values = Vec::with_capacity(module.enum_values.len());
        for definition in &module.enum_values {
            let definition = &self.enum_values_def[definition.as_usize()];
            let value = evaluate_parameter(&self.parsed, definition.syntax, &environment)?
                .ok_or_else(|| {
                    Error::new(&format!(
                        "Unable to resolve enumeration value {}",
                        self.parsed.symbol(definition.name)
                    ))
                })?;
            enum_values.push((definition.name, value));
        }
        let mut internal_signals = Vec::with_capacity(module.internal_signals.len());
        for definition in &module.internal_signals {
            let parsed = &self.internal_signals_def[definition.as_usize()];
            let (first_index, last_index, width) =
                resolved_shape(&self.parsed, parsed.syntax, &environment)?;
            let id = InternalSignalId(self.internal_signals.len() as u32);
            self.internal_signals.push(ResolvedInternalSignal {
                definition: *definition,
                specialization: specialization_id,
                name: parsed.name,
                kind: parsed.kind,
                width,
                first_index,
                last_index,
            });
            internal_signals.push(id);
        }
        {
            let specialization = &mut self.specializations[specialization_id.as_usize()];
            specialization.ports = ports;
            specialization.scan_registers = scan_registers;
            specialization.data_registers = data_registers;
            specialization.aliases = aliases;
            specialization.internal_signals = internal_signals;
            specialization.enum_values = enum_values;
        }

        let mut child_specializations = Vec::with_capacity(module.instances.len());
        for instance_id in &module.instances {
            let instance = self.instances_def[instance_id.as_usize()].clone();
            let child_module = self.resolve_module_reference(instance.module_type)?;
            let mut child_overrides = HashMap::new();
            for override_id in &instance.overrides {
                let parameter = &self.parameters_def[override_id.as_usize()];
                let value = evaluate_parameter(&self.parsed, parameter.syntax, &environment)?
                    .ok_or_else(|| {
                        Error::new(&format!(
                            "Unable to resolve parameter override {}",
                            self.parsed.symbol(parameter.name)
                        ))
                    })?;
                child_overrides.insert(parameter.name, value);
            }
            let child_specialization =
                self.ensure_specialization(child_module, child_overrides, active)?;
            child_specializations.push((*instance_id, child_specialization));
        }
        self.specializations[specialization_id.as_usize()].child_specializations =
            child_specializations;
        active.remove(&module_id);
        Ok(specialization_id)
    }

    fn parameter_environment(
        &self,
        module: &ModuleDef,
        overrides: HashMap<SymbolId, ParameterValue>,
    ) -> Result<HashMap<SymbolId, ParameterValue>> {
        let allowed: HashSet<_> = module
            .parameters
            .iter()
            .filter_map(|id| {
                let parameter = &self.parameters_def[id.as_usize()];
                (!parameter.local).then_some(parameter.name)
            })
            .collect();
        for name in overrides.keys() {
            if !allowed.contains(name) {
                return Err(Error::new(&format!(
                    "Override supplied for unknown or local parameter {}",
                    self.parsed.symbol(*name)
                )));
            }
        }

        let mut environment = overrides;
        let mut pending: Vec<_> = module
            .parameters
            .iter()
            .copied()
            .filter(|id| {
                let parameter = &self.parameters_def[id.as_usize()];
                parameter.local || !environment.contains_key(&parameter.name)
            })
            .collect();
        while !pending.is_empty() {
            let before = pending.len();
            pending.retain(|id| {
                let parameter = &self.parameters_def[id.as_usize()];
                match evaluate_parameter(&self.parsed, parameter.syntax, &environment) {
                    Ok(Some(value)) => {
                        environment.insert(parameter.name, value);
                        false
                    }
                    Ok(None) => true,
                    Err(_) => true,
                }
            });
            if pending.len() == before {
                let names = pending
                    .iter()
                    .map(|id| self.parsed.symbol(self.parameters_def[id.as_usize()].name))
                    .collect::<Vec<_>>()
                    .join(", ");
                return Err(Error::new(&format!(
                    "Unable to resolve parameter dependency cycle or missing reference: {names}"
                )));
            }
        }
        Ok(environment)
    }

    fn resolve_connections(&mut self) -> Result<()> {
        for specialization_index in 0..self.specializations.len() {
            let specialization_id = SpecializationId(specialization_index as u32);
            let specialization = self.specializations[specialization_index].clone();
            let module = self.modules[specialization.module.as_usize()].clone();
            let environment: HashMap<_, _> = specialization.parameters.iter().cloned().collect();
            let mut connections = Vec::new();

            for (definition, resolved) in module.ports.iter().zip(&specialization.ports) {
                let syntax = self.ports_def[definition.as_usize()].syntax;
                self.resolve_properties(
                    syntax,
                    ConnectionOwner::Port(*resolved),
                    specialization_id,
                    &environment,
                    &mut connections,
                )?;
            }
            for (definition, resolved) in module
                .scan_registers
                .iter()
                .zip(&specialization.scan_registers)
            {
                let syntax = self.scan_registers_def[definition.as_usize()].syntax;
                self.resolve_properties(
                    syntax,
                    ConnectionOwner::ScanRegister(*resolved),
                    specialization_id,
                    &environment,
                    &mut connections,
                )?;
            }
            for (definition, resolved) in module
                .data_registers
                .iter()
                .zip(&specialization.data_registers)
            {
                let syntax = self.data_registers_def[definition.as_usize()].syntax;
                self.resolve_properties(
                    syntax,
                    ConnectionOwner::DataRegister(*resolved),
                    specialization_id,
                    &environment,
                    &mut connections,
                )?;
            }
            for (definition, resolved) in module
                .internal_signals
                .iter()
                .zip(&specialization.internal_signals)
            {
                let syntax = self.internal_signals_def[definition.as_usize()].syntax;
                match self.internal_signals_def[definition.as_usize()].kind {
                    InternalSignalType::Logic => {
                        if let Some(expression) = self.parsed.children(syntax).find(|child| {
                            matches!(self.parsed.kind(*child), SyntaxKind::LogicExpression)
                        }) {
                            self.add_connection(
                                ConnectionOwner::InternalSignal(*resolved),
                                ConnectionKind::LogicExpression,
                                expression,
                                specialization_id,
                                &environment,
                                &mut connections,
                            )?;
                        }
                    }
                    InternalSignalType::Mux(_) => {
                        let mut skipped_name = false;
                        let syntax_children: Vec<_> = self.parsed.children(syntax).collect();
                        for child in syntax_children {
                            if !skipped_name
                                && matches!(
                                    self.parsed.kind(child),
                                    SyntaxKind::Identifier(_) | SyntaxKind::VectorIdentifier
                                )
                            {
                                skipped_name = true;
                                continue;
                            }
                            match self.parsed.kind(child) {
                                SyntaxKind::MuxSelection(_) => self.add_connection(
                                    ConnectionOwner::InternalSignal(*resolved),
                                    ConnectionKind::MuxSelection,
                                    child,
                                    specialization_id,
                                    &environment,
                                    &mut connections,
                                )?,
                                SyntaxKind::Concatenation
                                | SyntaxKind::Identifier(_)
                                | SyntaxKind::VectorIdentifier
                                | SyntaxKind::HierarchicalIdentifier => self.add_connection(
                                    ConnectionOwner::InternalSignal(*resolved),
                                    ConnectionKind::MuxSelect,
                                    child,
                                    specialization_id,
                                    &environment,
                                    &mut connections,
                                )?,
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }

            for (definition, child_specialization) in &specialization.child_specializations {
                let instance = self.instances_def[definition.as_usize()].clone();
                let inputs: Vec<_> = self
                    .parsed
                    .children(instance.syntax)
                    .filter(|child| self.parsed.kind(*child) == SyntaxKind::InputPortConnection)
                    .collect();
                for input in inputs {
                    let target_name = declaration_name(&self.parsed, input)?;
                    let port = self.specializations[child_specialization.as_usize()]
                        .ports
                        .iter()
                        .find(|id| self.ports[id.as_usize()].name == target_name)
                        .copied()
                        .ok_or_else(|| {
                            Error::new(&format!(
                                "Unable to resolve instance input port {}",
                                self.parsed.symbol(target_name)
                            ))
                        })?;
                    self.add_connection_skipping_name(
                        ConnectionOwner::InstanceInput {
                            instance: *definition,
                            port,
                        },
                        ConnectionKind::InstanceInput,
                        input,
                        specialization_id,
                        &environment,
                        &mut connections,
                    )?;
                }
            }
            self.specializations[specialization_index].connections = connections;
        }
        Ok(())
    }

    fn resolve_properties(
        &mut self,
        syntax: SyntaxId,
        owner: ConnectionOwner,
        specialization: SpecializationId,
        environment: &HashMap<SymbolId, ParameterValue>,
        output: &mut Vec<ConnectionId>,
    ) -> Result<()> {
        let property_nodes: Vec<_> = self.parsed.children(syntax).collect();
        for child in property_nodes {
            let kind = match self.parsed.kind(child) {
                SyntaxKind::Source => Some(ConnectionKind::Source),
                SyntaxKind::Enable => Some(ConnectionKind::Enable),
                SyntaxKind::ScanInSource => Some(ConnectionKind::ScanInSource),
                SyntaxKind::CaptureSource => Some(ConnectionKind::CaptureSource),
                SyntaxKind::WriteEnSource => Some(ConnectionKind::WriteEnSource),
                SyntaxKind::WriteDataSource => Some(ConnectionKind::WriteDataSource),
                SyntaxKind::ReadDataSource => Some(ConnectionKind::ReadDataSource),
                _ => None,
            };
            if let Some(kind) = kind {
                self.add_connection(owner, kind, child, specialization, environment, output)?;
            }
        }
        Ok(())
    }

    fn add_connection_skipping_name(
        &mut self,
        owner: ConnectionOwner,
        kind: ConnectionKind,
        syntax: SyntaxId,
        specialization: SpecializationId,
        environment: &HashMap<SymbolId, ParameterValue>,
        output: &mut Vec<ConnectionId>,
    ) -> Result<()> {
        let mut children: Vec<_> = self.parsed.children(syntax).collect();
        if let Some(position) = children.iter().position(|child| {
            matches!(
                self.parsed.kind(*child),
                SyntaxKind::Identifier(_) | SyntaxKind::VectorIdentifier
            )
        }) {
            children.drain(..=position);
        }
        self.add_connection_from_nodes(
            owner,
            kind,
            syntax,
            &children,
            specialization,
            environment,
            output,
        )
    }

    fn add_connection(
        &mut self,
        owner: ConnectionOwner,
        kind: ConnectionKind,
        syntax: SyntaxId,
        specialization: SpecializationId,
        environment: &HashMap<SymbolId, ParameterValue>,
        output: &mut Vec<ConnectionId>,
    ) -> Result<()> {
        let children: Vec<_> = if matches!(
            self.parsed.kind(syntax),
            SyntaxKind::Identifier(_)
                | SyntaxKind::VectorIdentifier
                | SyntaxKind::HierarchicalIdentifier
                | SyntaxKind::Number
                | SyntaxKind::ParameterReference(_)
                | SyntaxKind::Signal(_)
        ) {
            vec![syntax]
        } else {
            self.parsed.children(syntax).collect()
        };
        self.add_connection_from_nodes(
            owner,
            kind,
            syntax,
            &children,
            specialization,
            environment,
            output,
        )
    }

    fn add_connection_from_nodes(
        &mut self,
        owner: ConnectionOwner,
        kind: ConnectionKind,
        syntax: SyntaxId,
        nodes: &[SyntaxId],
        specialization: SpecializationId,
        environment: &HashMap<SymbolId, ParameterValue>,
        output: &mut Vec<ConnectionId>,
    ) -> Result<()> {
        let mut segments = Vec::new();
        self.collect_connection_segments(nodes, specialization, environment, false, &mut segments)?;
        if !segments.is_empty() {
            let id = ConnectionId(self.connections.len() as u32);
            self.connections.push(ResolvedConnection {
                specialization,
                owner,
                kind,
                source: self.parsed.span(syntax),
                segments,
            });
            output.push(id);
        }
        Ok(())
    }

    fn collect_connection_segments(
        &self,
        nodes: &[SyntaxId],
        specialization: SpecializationId,
        environment: &HashMap<SymbolId, ParameterValue>,
        inherited_inversion: bool,
        output: &mut Vec<ConnectionSegment>,
    ) -> Result<()> {
        let mut inverted = inherited_inversion;
        for node in nodes {
            match self.parsed.kind(*node) {
                SyntaxKind::Invert | SyntaxKind::BooleanNot => {
                    inverted = !inverted;
                }
                SyntaxKind::Number | SyntaxKind::ParameterReference(_) => {
                    if let Some(value) = evaluate_value(&self.parsed, *node, environment)? {
                        output.push(ConnectionSegment {
                            relative_instance_path: Vec::new(),
                            target: ConnectionTarget::Constant(if inverted {
                                invert_value(value)?
                            } else {
                                value
                            }),
                            selection: BitSelection::Whole,
                            inverted: false,
                        });
                    }
                    inverted = false;
                }
                SyntaxKind::Identifier(_) | SyntaxKind::VectorIdentifier => {
                    let (path, name, selection) = local_target(&self.parsed, *node, environment)?;
                    if path.is_empty() {
                        if let Some(value) = self.enum_value(specialization, name) {
                            output.push(ConnectionSegment {
                                relative_instance_path: Vec::new(),
                                target: ConnectionTarget::Constant(if inverted {
                                    invert_value(value.clone())?
                                } else {
                                    value.clone()
                                }),
                                selection: BitSelection::Whole,
                                inverted: false,
                            });
                            inverted = false;
                            continue;
                        }
                    }
                    let target = self.connection_endpoint(specialization, &path, name)?;
                    output.push(ConnectionSegment {
                        relative_instance_path: path,
                        target: ConnectionTarget::Object(target),
                        selection,
                        inverted,
                    });
                    inverted = false;
                }
                SyntaxKind::HierarchicalIdentifier
                | SyntaxKind::Signal(SignalType::HierarchicalData) => {
                    let (path, name, selection) = signal_target(&self.parsed, *node, environment)?;
                    let target = self.connection_endpoint(specialization, &path, name)?;
                    output.push(ConnectionSegment {
                        relative_instance_path: path,
                        target: ConnectionTarget::Object(target),
                        selection,
                        inverted,
                    });
                    inverted = false;
                }
                _ => {
                    let children: Vec<_> = self.parsed.children(*node).collect();
                    self.collect_connection_segments(
                        &children,
                        specialization,
                        environment,
                        inverted,
                        output,
                    )?;
                    inverted = false;
                }
            }
        }
        Ok(())
    }

    fn connection_endpoint(
        &self,
        mut specialization: SpecializationId,
        path: &[SymbolId],
        name: SymbolId,
    ) -> Result<ConnectionEndpoint> {
        for component in path {
            specialization = self
                .child_specialization(specialization, *component)
                .ok_or_else(|| {
                    Error::new(&format!(
                        "Unable to resolve signal path component {}",
                        self.parsed.symbol(*component)
                    ))
                })?;
        }
        if path.is_empty() {
            if let Some((definition, _)) = self.specializations[specialization.as_usize()]
                .child_specializations
                .iter()
                .find(|(definition, _)| self.instances_def[definition.as_usize()].name == name)
            {
                return Ok(ConnectionEndpoint::Instance(*definition));
            }
        }
        if let Some((endpoint, _, _, _)) = self.local_endpoint(specialization, name) {
            return Ok(match endpoint {
                AliasEndpoint::Port(id) => ConnectionEndpoint::Port(id),
                AliasEndpoint::ScanRegister(id) => ConnectionEndpoint::ScanRegister(id),
                AliasEndpoint::DataRegister(id) => ConnectionEndpoint::DataRegister(id),
                AliasEndpoint::InternalSignal(id) => ConnectionEndpoint::InternalSignal(id),
            });
        }
        if let Some(alias) = self.local_alias(specialization, name) {
            return Ok(ConnectionEndpoint::Alias(alias));
        }
        Err(Error::new(&format!(
            "Unable to resolve signal reference {} in module specialization {}",
            self.parsed.symbol(name),
            self.modules[self.specializations[specialization.as_usize()]
                .module
                .as_usize()]
            .qualified_name
        )))
    }

    fn enum_value(
        &self,
        specialization: SpecializationId,
        name: SymbolId,
    ) -> Option<&ParameterValue> {
        self.specializations[specialization.as_usize()]
            .enum_values
            .iter()
            .find_map(|(symbol, value)| (*symbol == name).then_some(value))
    }

    fn resolve_aliases(&mut self) -> Result<()> {
        let mut states = vec![0u8; self.aliases.len()];
        for index in 0..self.aliases.len() {
            self.resolve_alias(AliasId(index as u32), &mut states)?;
        }
        Ok(())
    }

    fn resolve_alias(&mut self, alias_id: AliasId, states: &mut [u8]) -> Result<()> {
        match states[alias_id.as_usize()] {
            2 => return Ok(()),
            1 => return Err(Error::new("Cycle detected in alias definitions")),
            _ => states[alias_id.as_usize()] = 1,
        }
        let alias = self.aliases[alias_id.as_usize()].clone();
        let definition = self.aliases_def[alias.definition.as_usize()].clone();
        let environment: HashMap<_, _> = self.specializations[alias.specialization.as_usize()]
            .parameters
            .iter()
            .cloned()
            .collect();
        let targets = alias_target_nodes(&self.parsed, definition.syntax);
        let mut resolved = Vec::new();
        for (inverted, target_node) in targets {
            let (path, name, selection) = signal_target(&self.parsed, target_node, &environment)?;
            let mut target_specialization = alias.specialization;
            for component in &path {
                target_specialization = self
                    .child_specialization(target_specialization, *component)
                    .ok_or_else(|| {
                        Error::new(&format!(
                            "Unable to resolve alias instance path component {}",
                            self.parsed.symbol(*component)
                        ))
                    })?;
            }
            if let Some((endpoint, first_index, last_index, endpoint_width)) =
                self.local_endpoint(target_specialization, name)
            {
                let selected_width = selection.width(endpoint_width);
                validate_selection(selection, first_index, last_index)?;
                resolved.push((path, endpoint, selection, inverted, selected_width));
            } else if let Some(target_alias) = self.local_alias(target_specialization, name) {
                self.resolve_alias(target_alias, states)?;
                let nested = self.aliases[target_alias.as_usize()].clone();
                validate_selection(selection, nested.first_index, nested.last_index)?;
                for (segment, width) in self.select_alias_segments(&nested, selection)? {
                    let mut combined_path = path.clone();
                    combined_path.extend(segment.relative_instance_path);
                    resolved.push((
                        combined_path,
                        segment.target,
                        segment.selection,
                        inverted ^ segment.inverted,
                        width,
                    ));
                }
            } else {
                return Err(Error::new(&format!(
                    "Unable to resolve alias target {}",
                    self.parsed.symbol(name)
                )));
            }
        }

        let mut offset = 0;
        let mut segment_ids = Vec::with_capacity(resolved.len());
        for (path, target, selection, inverted, width) in resolved.into_iter().rev() {
            let id = AliasSegmentId(self.alias_segments.len() as u32);
            self.alias_segments.push(AliasSegment {
                relative_instance_path: path,
                target,
                selection,
                inverted,
                alias_bit_offset: offset,
            });
            segment_ids.push(id);
            offset += width;
        }
        segment_ids.reverse();
        let declared_shape = declared_vector_shape(&self.parsed, definition.syntax, &environment)?;
        if let Some((_, _, declared_width)) = declared_shape {
            if declared_width != offset {
                return Err(Error::new(&format!(
                    "Alias {} declares width {} but resolves to {} bits",
                    self.parsed.symbol(alias.name),
                    declared_width,
                    offset
                )));
            }
        }
        self.aliases[alias_id.as_usize()].width = offset;
        self.aliases[alias_id.as_usize()].first_index = declared_shape
            .map(|shape| shape.0)
            .unwrap_or(offset.saturating_sub(1));
        self.aliases[alias_id.as_usize()].last_index =
            declared_shape.map(|shape| shape.1).unwrap_or(0);
        self.aliases[alias_id.as_usize()].segments = segment_ids;
        states[alias_id.as_usize()] = 2;
        Ok(())
    }

    fn child_specialization(
        &self,
        specialization: SpecializationId,
        name: SymbolId,
    ) -> Option<SpecializationId> {
        self.specializations[specialization.as_usize()]
            .child_specializations
            .iter()
            .find_map(|(definition, child)| {
                (self.instances_def[definition.as_usize()].name == name).then_some(*child)
            })
    }

    fn local_endpoint(
        &self,
        specialization: SpecializationId,
        name: SymbolId,
    ) -> Option<(AliasEndpoint, u32, u32, u32)> {
        let specialization = &self.specializations[specialization.as_usize()];
        for id in &specialization.ports {
            let port = &self.ports[id.as_usize()];
            if port.name == name {
                return Some((
                    AliasEndpoint::Port(*id),
                    port.first_index,
                    port.last_index,
                    port.width,
                ));
            }
        }
        for id in &specialization.scan_registers {
            let register = &self.scan_registers[id.as_usize()];
            if register.name == name {
                return Some((
                    AliasEndpoint::ScanRegister(*id),
                    register.first_index,
                    register.last_index,
                    register.width,
                ));
            }
        }
        for id in &specialization.data_registers {
            let register = &self.data_registers[id.as_usize()];
            if register.name == name {
                return Some((
                    AliasEndpoint::DataRegister(*id),
                    register.first_index,
                    register.last_index,
                    register.width,
                ));
            }
        }
        for id in &specialization.internal_signals {
            let signal = &self.internal_signals[id.as_usize()];
            if signal.name == name {
                return Some((
                    AliasEndpoint::InternalSignal(*id),
                    signal.first_index,
                    signal.last_index,
                    signal.width,
                ));
            }
        }
        None
    }

    fn local_alias(&self, specialization: SpecializationId, name: SymbolId) -> Option<AliasId> {
        self.specializations[specialization.as_usize()]
            .aliases
            .iter()
            .find_map(|id| (self.aliases[id.as_usize()].name == name).then_some(*id))
    }

    fn select_alias_segments(
        &self,
        alias: &ResolvedAlias,
        selection: BitSelection,
    ) -> Result<Vec<(AliasSegment, u32)>> {
        if selection == BitSelection::Whole {
            return Ok(alias
                .segments
                .iter()
                .map(|id| {
                    let segment = self.alias_segments[id.as_usize()].clone();
                    let width = self.segment_width(&segment);
                    (segment, width)
                })
                .collect());
        }
        let indices: Vec<u32> = match selection {
            BitSelection::Index(index) => vec![index],
            BitSelection::Range { first, last } if first >= last => (last..=first).rev().collect(),
            BitSelection::Range { first, last } => (first..=last).collect(),
            BitSelection::Whole => unreachable!(),
        };
        let mut selected = Vec::with_capacity(indices.len());
        for index in indices {
            let alias_offset = index.abs_diff(alias.last_index);
            let segment = alias
                .segments
                .iter()
                .map(|id| &self.alias_segments[id.as_usize()])
                .find(|segment| {
                    let width = self.segment_width(segment);
                    (segment.alias_bit_offset..segment.alias_bit_offset + width)
                        .contains(&alias_offset)
                })
                .ok_or_else(|| Error::new("Alias selection could not be mapped to a target bit"))?;
            let local_offset = alias_offset - segment.alias_bit_offset;
            let target_index = match segment.selection {
                BitSelection::Whole => {
                    let (_, last, _) = self.endpoint_shape(segment.target);
                    if self.endpoint_shape(segment.target).0 >= last {
                        last + local_offset
                    } else {
                        last - local_offset
                    }
                }
                BitSelection::Index(index) => index,
                BitSelection::Range { first, last } if first >= last => last + local_offset,
                BitSelection::Range { last, .. } => last - local_offset,
            };
            let mut segment = segment.clone();
            segment.selection = BitSelection::Index(target_index);
            selected.push((segment, 1));
        }
        Ok(selected)
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

    fn segment_width(&self, segment: &AliasSegment) -> u32 {
        segment
            .selection
            .width(self.endpoint_shape(segment.target).2)
    }
}

fn first_direct_symbol(parsed: &ParsedIcl, node: SyntaxId) -> Option<SymbolId> {
    parsed
        .children(node)
        .find_map(|child| match parsed.kind(child) {
            SyntaxKind::Identifier(symbol) => Some(symbol),
            SyntaxKind::VectorIdentifier => first_direct_symbol(parsed, child),
            _ => None,
        })
}

fn declaration_name(parsed: &ParsedIcl, node: SyntaxId) -> Result<SymbolId> {
    first_direct_symbol(parsed, node)
        .ok_or_else(|| Error::new("ICL declaration is missing its name"))
}

fn parameter_value_nodes(parsed: &ParsedIcl, syntax: SyntaxId) -> Vec<SyntaxId> {
    let mut skipped_name = false;
    parsed
        .children(syntax)
        .filter(|child| {
            if !skipped_name
                && matches!(
                    parsed.kind(*child),
                    SyntaxKind::Identifier(_) | SyntaxKind::VectorIdentifier
                )
            {
                skipped_name = true;
                false
            } else {
                !matches!(parsed.kind(*child), SyntaxKind::Comment)
            }
        })
        .collect()
}

fn evaluate_parameter(
    parsed: &ParsedIcl,
    syntax: SyntaxId,
    environment: &HashMap<SymbolId, ParameterValue>,
) -> Result<Option<ParameterValue>> {
    evaluate_sequence(parsed, &parameter_value_nodes(parsed, syntax), environment)
}

fn evaluate_sequence(
    parsed: &ParsedIcl,
    nodes: &[SyntaxId],
    environment: &HashMap<SymbolId, ParameterValue>,
) -> Result<Option<ParameterValue>> {
    if nodes.is_empty() {
        return Err(Error::new("Missing ICL value"));
    }
    if nodes.len() == 1 {
        return evaluate_value(parsed, nodes[0], environment);
    }
    let mut inverted = false;
    let mut values = Vec::new();
    for node in nodes {
        if parsed.kind(*node) == SyntaxKind::Invert {
            inverted = !inverted;
            continue;
        }
        let Some(mut value) = evaluate_value(parsed, *node, environment)? else {
            return Ok(None);
        };
        if inverted {
            value = invert_value(value)?;
            inverted = false;
        }
        values.push(value);
    }
    concatenate(values).map(Some)
}

fn evaluate_value(
    parsed: &ParsedIcl,
    node: SyntaxId,
    environment: &HashMap<SymbolId, ParameterValue>,
) -> Result<Option<ParameterValue>> {
    match parsed.kind(node) {
        SyntaxKind::Number => parse_number(parsed, parsed.node_text(node), environment),
        SyntaxKind::StringLiteral => Ok(Some(ParameterValue::String(unquote(
            parsed.node_text(node),
        )))),
        SyntaxKind::Identifier(symbol) => Ok(Some(ParameterValue::Symbol(symbol))),
        SyntaxKind::ParameterReference(symbol) => Ok(environment.get(&symbol).cloned()),
        SyntaxKind::Concatenation => {
            let children: Vec<_> = parsed.children(node).collect();
            evaluate_sequence(parsed, &children, environment)
        }
        SyntaxKind::Parentheses | SyntaxKind::Index => {
            let child = parsed
                .children(node)
                .next()
                .ok_or_else(|| Error::new("Empty expression"))?;
            evaluate_value(parsed, child, environment)
        }
        SyntaxKind::IntegerExpression | SyntaxKind::IntegerTerm => {
            evaluate_integer_expression(parsed, node, environment)
                .map(|value| value.map(ParameterValue::Integer))
        }
        _ => Err(Error::new(
            "Unsupported value expression during elaboration",
        )),
    }
}

fn evaluate_integer_expression(
    parsed: &ParsedIcl,
    node: SyntaxId,
    environment: &HashMap<SymbolId, ParameterValue>,
) -> Result<Option<BigInt>> {
    let mut children = parsed.children(node);
    let Some(first) = children.next() else {
        return Err(Error::new("Empty integer expression"));
    };
    let Some(mut value) = evaluate_integer(parsed, first, environment)? else {
        return Ok(None);
    };
    while let Some(operator) = children.next() {
        let rhs_node = children
            .next()
            .ok_or_else(|| Error::new("Integer operator is missing its right operand"))?;
        let Some(rhs) = evaluate_integer(parsed, rhs_node, environment)? else {
            return Ok(None);
        };
        value = match parsed.kind(operator) {
            SyntaxKind::Add => value + rhs,
            SyntaxKind::Subtract => value - rhs,
            SyntaxKind::Multiply => value * rhs,
            SyntaxKind::Divide if rhs.is_zero() => return Err(Error::new("Division by zero")),
            SyntaxKind::Divide => value / rhs,
            SyntaxKind::Modulo if rhs.is_zero() => return Err(Error::new("Modulo by zero")),
            SyntaxKind::Modulo => value % rhs,
            _ => return Err(Error::new("Unexpected integer expression operator")),
        };
    }
    Ok(Some(value))
}

fn evaluate_integer(
    parsed: &ParsedIcl,
    node: SyntaxId,
    environment: &HashMap<SymbolId, ParameterValue>,
) -> Result<Option<BigInt>> {
    match evaluate_value(parsed, node, environment)? {
        None => Ok(None),
        Some(ParameterValue::Integer(value)) => Ok(Some(value)),
        Some(ParameterValue::Bits { value, unknown, .. }) if unknown.is_zero() => {
            Ok(Some(BigInt::from_biguint(Sign::Plus, value)))
        }
        Some(ParameterValue::Bits { .. }) => {
            Err(Error::new("Unknown bits cannot be used as an integer"))
        }
        Some(ParameterValue::String(_)) => Err(Error::new("A string cannot be used as an integer")),
        Some(ParameterValue::Symbol(_)) => {
            Err(Error::new("A symbolic value cannot be used as an integer"))
        }
    }
}

fn parse_number(
    parsed: &ParsedIcl,
    raw: &str,
    environment: &HashMap<SymbolId, ParameterValue>,
) -> Result<Option<ParameterValue>> {
    let compact: String = raw
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '_')
        .collect();
    let Some(quote) = compact.find('\'') else {
        let value = BigInt::parse_bytes(compact.as_bytes(), 10)
            .ok_or_else(|| Error::new(&format!("Invalid decimal number: {raw}")))?;
        return Ok(Some(ParameterValue::Integer(value)));
    };
    let size_text = &compact[..quote];
    let base = compact
        .as_bytes()
        .get(quote + 1)
        .copied()
        .ok_or_else(|| Error::new(&format!("Missing base in number: {raw}")))?
        as char;
    let digits = &compact[quote + 2..];
    let explicit_width = if size_text.is_empty() {
        None
    } else if let Some(name) = size_text.strip_prefix('$') {
        let Some(symbol) = parsed.symbol_id(name) else {
            return Ok(None);
        };
        let Some(value) = environment.get(&symbol) else {
            return Ok(None);
        };
        Some(parameter_as_u32(value)?)
    } else {
        Some(
            size_text
                .parse::<u32>()
                .map_err(|_| Error::new(&format!("Invalid number width: {raw}")))?,
        )
    };

    let radix = match base.to_ascii_lowercase() {
        'b' => 2,
        'd' => 10,
        'h' => 16,
        _ => return Err(Error::new(&format!("Invalid number base: {raw}"))),
    };
    let mut value = BigUint::zero();
    let mut unknown = BigUint::zero();
    if radix == 10 {
        value = BigUint::from_str_radix(digits, 10)
            .map_err(|_| Error::new(&format!("Invalid decimal number: {raw}")))?;
    } else {
        let bits_per_digit = if radix == 2 { 1 } else { 4 };
        for digit in digits.chars() {
            value <<= bits_per_digit;
            unknown <<= bits_per_digit;
            if digit == 'x' || digit == 'X' {
                unknown |= (BigUint::one() << bits_per_digit) - BigUint::one();
            } else {
                value |= BigUint::from(
                    digit
                        .to_digit(radix)
                        .ok_or_else(|| Error::new(&format!("Invalid based number: {raw}")))?,
                );
            }
        }
    }
    let inferred = if radix == 2 {
        digits.len() as u32
    } else if radix == 16 {
        digits.len() as u32 * 4
    } else {
        value.bits().max(1) as u32
    };
    let width = explicit_width.unwrap_or(inferred);
    if value.bits() > width as u64 || unknown.bits() > width as u64 {
        return Err(Error::new(&format!(
            "Number does not fit its declared width: {raw}"
        )));
    }
    Ok(Some(ParameterValue::Bits {
        width,
        value,
        unknown,
    }))
}

fn parameter_as_u32(value: &ParameterValue) -> Result<u32> {
    match value {
        ParameterValue::Integer(value) => value
            .to_u32()
            .ok_or_else(|| Error::new("Parameter value is not a valid u32")),
        ParameterValue::Bits { value, unknown, .. } if unknown.is_zero() => value
            .to_u32()
            .ok_or_else(|| Error::new("Parameter value is not a valid u32")),
        _ => Err(Error::new("Parameter value is not a known integer")),
    }
}

fn invert_value(value: ParameterValue) -> Result<ParameterValue> {
    match value {
        ParameterValue::Bits {
            width,
            value,
            unknown,
        } => {
            let mask = (BigUint::one() << width) - BigUint::one();
            Ok(ParameterValue::Bits {
                width,
                value: (&mask ^ value) & (&mask ^ &unknown),
                unknown,
            })
        }
        _ => Err(Error::new("Only sized bit values can be inverted")),
    }
}

fn concatenate(values: Vec<ParameterValue>) -> Result<ParameterValue> {
    if values
        .iter()
        .all(|value| matches!(value, ParameterValue::String(_)))
    {
        let mut result = String::new();
        for value in values {
            if let ParameterValue::String(value) = value {
                result.push_str(&value);
            }
        }
        return Ok(ParameterValue::String(result));
    }
    let mut width = 0u32;
    let mut result = BigUint::zero();
    let mut unknown_result = BigUint::zero();
    for value in values {
        let (item_width, item, unknown) = match value {
            ParameterValue::Integer(value) if value.sign() != Sign::Minus => {
                let value = value.to_biguint().unwrap();
                (value.bits().max(1) as u32, value, BigUint::zero())
            }
            ParameterValue::Bits {
                width,
                value,
                unknown,
            } => (width, value, unknown),
            _ => {
                return Err(Error::new(
                    "Cannot concatenate mixed string and numeric values",
                ))
            }
        };
        result = (result << item_width) | item;
        unknown_result = (unknown_result << item_width) | unknown;
        width = width
            .checked_add(item_width)
            .ok_or_else(|| Error::new("Concatenated value is too wide"))?;
    }
    Ok(ParameterValue::Bits {
        width,
        value: result,
        unknown: unknown_result,
    })
}

fn unquote(raw: &str) -> String {
    let body = &raw[1..raw.len() - 1];
    let mut result = String::with_capacity(body.len());
    let mut chars = body.chars();
    while let Some(character) = chars.next() {
        if character == '\\' {
            if let Some(escaped) = chars.next() {
                result.push(escaped);
            }
        } else {
            result.push(character);
        }
    }
    result
}

fn resolved_property_value(
    parsed: &ParsedIcl,
    declaration: SyntaxId,
    property_kind: SyntaxKind,
    environment: &HashMap<SymbolId, ParameterValue>,
) -> Result<Option<ParameterValue>> {
    let Some(property) = parsed
        .children(declaration)
        .find(|child| parsed.kind(*child) == property_kind)
    else {
        return Ok(None);
    };
    let children: Vec<_> = parsed.children(property).collect();
    evaluate_sequence(parsed, &children, environment)
}

fn property_symbol(
    parsed: &ParsedIcl,
    declaration: SyntaxId,
    property_kind: SyntaxKind,
) -> Option<SymbolId> {
    parsed
        .children(declaration)
        .find(|child| parsed.kind(*child) == property_kind)
        .and_then(|property| first_direct_symbol(parsed, property))
}

fn active_polarity(parsed: &ParsedIcl, declaration: SyntaxId) -> Option<bool> {
    parsed
        .children(declaration)
        .find_map(|child| match parsed.kind(child) {
            SyntaxKind::ActivePolarity(value) => Some(value),
            _ => None,
        })
}

fn resolved_shape(
    parsed: &ParsedIcl,
    declaration: SyntaxId,
    environment: &HashMap<SymbolId, ParameterValue>,
) -> Result<(u32, u32, u32)> {
    let vector = parsed
        .children(declaration)
        .find(|child| parsed.kind(*child) == SyntaxKind::VectorIdentifier);
    let Some(vector) = vector else {
        return Ok((0, 0, 1));
    };
    if let Some(range) = parsed
        .children(vector)
        .find(|child| parsed.kind(*child) == SyntaxKind::Range)
    {
        let expressions: Vec<_> = parsed.children(range).collect();
        if expressions.len() != 2 {
            return Err(Error::new("Malformed vector range"));
        }
        let first = evaluate_integer(parsed, expressions[0], environment)?
            .ok_or_else(|| Error::new("Unresolved vector range"))?
            .to_u32()
            .ok_or_else(|| Error::new("Vector index is not a valid u32"))?;
        let last = evaluate_integer(parsed, expressions[1], environment)?
            .ok_or_else(|| Error::new("Unresolved vector range"))?
            .to_u32()
            .ok_or_else(|| Error::new("Vector index is not a valid u32"))?;
        return Ok((first, last, first.abs_diff(last) + 1));
    }
    let index = parsed
        .children(vector)
        .find(|child| parsed.kind(*child) == SyntaxKind::Index)
        .ok_or_else(|| Error::new("Malformed vector index"))?;
    let expression = parsed
        .children(index)
        .next()
        .ok_or_else(|| Error::new("Malformed vector index"))?;
    let value = evaluate_integer(parsed, expression, environment)?
        .ok_or_else(|| Error::new("Unresolved vector index"))?
        .to_u32()
        .ok_or_else(|| Error::new("Vector index is not a valid u32"))?;
    Ok((value, value, 1))
}

fn declared_vector_shape(
    parsed: &ParsedIcl,
    declaration: SyntaxId,
    environment: &HashMap<SymbolId, ParameterValue>,
) -> Result<Option<(u32, u32, u32)>> {
    if parsed
        .children(declaration)
        .any(|child| parsed.kind(child) == SyntaxKind::VectorIdentifier)
    {
        Ok(Some(resolved_shape(parsed, declaration, environment)?))
    } else {
        Ok(None)
    }
}

fn alias_target_nodes(parsed: &ParsedIcl, alias: SyntaxId) -> Vec<(bool, SyntaxId)> {
    fn scan(parsed: &ParsedIcl, parent: SyntaxId, output: &mut Vec<(bool, SyntaxId)>) {
        let mut inverted = false;
        for child in parsed.children(parent) {
            match parsed.kind(child) {
                SyntaxKind::Invert => inverted = !inverted,
                SyntaxKind::Signal(SignalType::HierarchicalData) => {
                    output.push((inverted, child));
                    inverted = false;
                }
                SyntaxKind::Concatenation => scan(parsed, child, output),
                _ => {}
            }
        }
    }
    let mut output = Vec::new();
    scan(parsed, alias, &mut output);
    output
}

fn signal_target(
    parsed: &ParsedIcl,
    signal: SyntaxId,
    environment: &HashMap<SymbolId, ParameterValue>,
) -> Result<(Vec<SymbolId>, SymbolId, BitSelection)> {
    let components: Vec<_> = parsed
        .children(signal)
        .filter(|child| {
            matches!(
                parsed.kind(*child),
                SyntaxKind::Identifier(_) | SyntaxKind::VectorIdentifier
            )
        })
        .collect();
    let target = *components
        .last()
        .ok_or_else(|| Error::new("Alias target is empty"))?;
    let name = first_direct_symbol(parsed, target)
        .or_else(|| match parsed.kind(target) {
            SyntaxKind::Identifier(symbol) => Some(symbol),
            _ => None,
        })
        .ok_or_else(|| Error::new("Alias target has no name"))?;
    let path = components[..components.len() - 1]
        .iter()
        .filter_map(|component| match parsed.kind(*component) {
            SyntaxKind::Identifier(symbol) => Some(symbol),
            _ => first_direct_symbol(parsed, *component),
        })
        .collect();
    let selection = match parsed.kind(target) {
        SyntaxKind::Identifier(_) => BitSelection::Whole,
        SyntaxKind::VectorIdentifier => {
            if let Some(range) = parsed
                .children(target)
                .find(|child| parsed.kind(*child) == SyntaxKind::Range)
            {
                let expressions: Vec<_> = parsed.children(range).collect();
                let first = evaluate_integer(parsed, expressions[0], environment)?
                    .ok_or_else(|| Error::new("Unresolved alias range"))?
                    .to_u32()
                    .ok_or_else(|| Error::new("Alias index is not a valid u32"))?;
                let last = evaluate_integer(parsed, expressions[1], environment)?
                    .ok_or_else(|| Error::new("Unresolved alias range"))?
                    .to_u32()
                    .ok_or_else(|| Error::new("Alias index is not a valid u32"))?;
                BitSelection::Range { first, last }
            } else {
                let index = parsed
                    .children(target)
                    .find(|child| parsed.kind(*child) == SyntaxKind::Index)
                    .and_then(|index| parsed.children(index).next())
                    .ok_or_else(|| Error::new("Malformed alias index"))?;
                let value = evaluate_integer(parsed, index, environment)?
                    .ok_or_else(|| Error::new("Unresolved alias index"))?
                    .to_u32()
                    .ok_or_else(|| Error::new("Alias index is not a valid u32"))?;
                BitSelection::Index(value)
            }
        }
        _ => return Err(Error::new("Malformed alias target")),
    };
    Ok((path, name, selection))
}

fn local_target(
    parsed: &ParsedIcl,
    target: SyntaxId,
    environment: &HashMap<SymbolId, ParameterValue>,
) -> Result<(Vec<SymbolId>, SymbolId, BitSelection)> {
    match parsed.kind(target) {
        SyntaxKind::Identifier(name) => Ok((Vec::new(), name, BitSelection::Whole)),
        SyntaxKind::VectorIdentifier => {
            let name = first_direct_symbol(parsed, target)
                .ok_or_else(|| Error::new("Signal target has no name"))?;
            let selection = if let Some(range) = parsed
                .children(target)
                .find(|child| parsed.kind(*child) == SyntaxKind::Range)
            {
                let expressions: Vec<_> = parsed.children(range).collect();
                let first = evaluate_integer(parsed, expressions[0], environment)?
                    .ok_or_else(|| Error::new("Unresolved signal range"))?
                    .to_u32()
                    .ok_or_else(|| Error::new("Signal index is not a valid u32"))?;
                let last = evaluate_integer(parsed, expressions[1], environment)?
                    .ok_or_else(|| Error::new("Unresolved signal range"))?
                    .to_u32()
                    .ok_or_else(|| Error::new("Signal index is not a valid u32"))?;
                BitSelection::Range { first, last }
            } else {
                let expression = parsed
                    .children(target)
                    .find(|child| parsed.kind(*child) == SyntaxKind::Index)
                    .and_then(|index| parsed.children(index).next())
                    .ok_or_else(|| Error::new("Malformed signal index"))?;
                let index = evaluate_integer(parsed, expression, environment)?
                    .ok_or_else(|| Error::new("Unresolved signal index"))?
                    .to_u32()
                    .ok_or_else(|| Error::new("Signal index is not a valid u32"))?;
                BitSelection::Index(index)
            };
            Ok((Vec::new(), name, selection))
        }
        _ => Err(Error::new("Malformed local signal target")),
    }
}

fn validate_selection(selection: BitSelection, first_index: u32, last_index: u32) -> Result<()> {
    let low = first_index.min(last_index);
    let high = first_index.max(last_index);
    match selection {
        BitSelection::Whole => Ok(()),
        BitSelection::Index(index) if (low..=high).contains(&index) => Ok(()),
        BitSelection::Range { first, last }
            if (low..=high).contains(&first) && (low..=high).contains(&last) =>
        {
            Ok(())
        }
        _ => Err(Error::new("Alias selection is outside the target object")),
    }
}

fn build_occurrences(
    elaborator: &Elaborator,
    specialization: SpecializationId,
    name: SymbolId,
    parent: Option<InstanceId>,
    output: &mut Vec<Instance>,
) -> InstanceId {
    let id = InstanceId(output.len() as u32);
    output.push(Instance {
        name,
        parent,
        specialization,
        children: Vec::new(),
    });
    let child_specs = elaborator.specializations[specialization.as_usize()]
        .child_specializations
        .clone();
    let mut children = Vec::with_capacity(child_specs.len());
    for (definition, child_specialization) in child_specs {
        let child_name = elaborator.instances_def[definition.as_usize()].name;
        children.push(build_occurrences(
            elaborator,
            child_specialization,
            child_name,
            Some(id),
            output,
        ));
    }
    output[id.as_usize()].children = children;
    id
}
