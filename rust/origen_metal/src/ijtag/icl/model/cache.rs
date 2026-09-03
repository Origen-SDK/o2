use super::syntax::SyntaxNode;
use super::*;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;

const CACHE_MAGIC: [u8; 8] = *b"O2ICL001";
const CACHE_SCHEMA: u32 = 4;

#[derive(Deserialize, Serialize)]
struct CacheFile {
    magic: [u8; 8],
    schema: u32,
    source_hash: [u8; 32],
    top: Option<String>,
    preserve_comments: bool,
    nodes: Vec<SyntaxNode>,
    symbols: Vec<String>,
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

#[derive(Serialize)]
struct CacheWrite<'a> {
    magic: [u8; 8],
    schema: u32,
    source_hash: [u8; 32],
    top: Option<&'a str>,
    preserve_comments: bool,
    nodes: &'a [SyntaxNode],
    symbols: &'a [String],
    modules: &'a [ModuleDef],
    instances_def: &'a [InstanceDef],
    ports_def: &'a [PortDef],
    scan_registers_def: &'a [ScanRegisterDef],
    data_registers_def: &'a [DataRegisterDef],
    aliases_def: &'a [AliasDef],
    internal_signals_def: &'a [InternalSignalDef],
    enum_values_def: &'a [EnumValueDef],
    parameters_def: &'a [ParameterDef],
    specializations: &'a [Specialization],
    instances: &'a [Instance],
    ports: &'a [ResolvedPort],
    scan_registers: &'a [ResolvedScanRegister],
    data_registers: &'a [ResolvedDataRegister],
    aliases: &'a [ResolvedAlias],
    internal_signals: &'a [ResolvedInternalSignal],
    alias_segments: &'a [AliasSegment],
    connections: &'a [ResolvedConnection],
    connections_by_owner: &'a HashMap<ConnectionOwner, Vec<ConnectionId>>,
    root: InstanceId,
    module_by_name: &'a HashMap<SymbolId, ModuleDefId>,
    child_index: &'a HashMap<(InstanceId, SymbolId), InstanceId>,
    instances_by_name: &'a HashMap<SymbolId, Vec<InstanceId>>,
    instances_by_type: &'a HashMap<ModuleDefId, Vec<InstanceId>>,
    ports_by_name: &'a HashMap<SymbolId, Vec<PortHandle>>,
    scan_registers_by_name: &'a HashMap<SymbolId, Vec<ScanRegisterHandle>>,
    data_registers_by_name: &'a HashMap<SymbolId, Vec<DataRegisterHandle>>,
    aliases_by_name: &'a HashMap<SymbolId, Vec<AliasHandle>>,
}

pub(super) fn load(
    source_path: &Path,
    cache_path: &Path,
    top: Option<&str>,
    preserve_comments: bool,
) -> Result<Option<IclModel>> {
    if !cache_path.is_file() {
        return Ok(None);
    }
    let source = fs::read_to_string(source_path)?;
    let file = match File::open(cache_path) {
        Ok(file) => file,
        Err(_) => return Ok(None),
    };
    let mut reader = BufReader::new(file);
    let cache: CacheFile =
        match bincode::serde::decode_from_std_read(&mut reader, bincode::config::standard()) {
            Ok(cache) => cache,
            Err(_) => return Ok(None),
        };
    if cache.magic != CACHE_MAGIC
        || cache.schema != CACHE_SCHEMA
        || cache.source_hash != source_hash(source.as_bytes())
        || cache.top.as_deref() != top
        || cache.preserve_comments != preserve_comments
    {
        return Ok(None);
    }

    let parsed = ParsedIcl::from_parts(
        source,
        Some(source_path.display().to_string()),
        cache.nodes,
        cache.symbols,
    );
    Ok(Some(IclModel {
        parsed,
        modules: cache.modules,
        instances_def: cache.instances_def,
        ports_def: cache.ports_def,
        scan_registers_def: cache.scan_registers_def,
        data_registers_def: cache.data_registers_def,
        aliases_def: cache.aliases_def,
        internal_signals_def: cache.internal_signals_def,
        enum_values_def: cache.enum_values_def,
        parameters_def: cache.parameters_def,
        specializations: cache.specializations,
        instances: cache.instances,
        ports: cache.ports,
        scan_registers: cache.scan_registers,
        data_registers: cache.data_registers,
        aliases: cache.aliases,
        internal_signals: cache.internal_signals,
        alias_segments: cache.alias_segments,
        connections: cache.connections,
        connections_by_owner: cache.connections_by_owner,
        root: cache.root,
        module_by_name: cache.module_by_name,
        child_index: cache.child_index,
        instances_by_name: cache.instances_by_name,
        instances_by_type: cache.instances_by_type,
        ports_by_name: cache.ports_by_name,
        scan_registers_by_name: cache.scan_registers_by_name,
        data_registers_by_name: cache.data_registers_by_name,
        aliases_by_name: cache.aliases_by_name,
    }))
}

pub(super) fn save(
    model: &IclModel,
    cache_path: &Path,
    top: Option<&str>,
    preserve_comments: bool,
) -> Result<()> {
    let cache = CacheWrite {
        magic: CACHE_MAGIC,
        schema: CACHE_SCHEMA,
        source_hash: source_hash(model.parsed.source().as_bytes()),
        top,
        preserve_comments,
        nodes: &model.parsed.inner.nodes,
        symbols: &model.parsed.inner.symbols,
        modules: &model.modules,
        instances_def: &model.instances_def,
        ports_def: &model.ports_def,
        scan_registers_def: &model.scan_registers_def,
        data_registers_def: &model.data_registers_def,
        aliases_def: &model.aliases_def,
        internal_signals_def: &model.internal_signals_def,
        enum_values_def: &model.enum_values_def,
        parameters_def: &model.parameters_def,
        specializations: &model.specializations,
        instances: &model.instances,
        ports: &model.ports,
        scan_registers: &model.scan_registers,
        data_registers: &model.data_registers,
        aliases: &model.aliases,
        internal_signals: &model.internal_signals,
        alias_segments: &model.alias_segments,
        connections: &model.connections,
        connections_by_owner: &model.connections_by_owner,
        root: model.root,
        module_by_name: &model.module_by_name,
        child_index: &model.child_index,
        instances_by_name: &model.instances_by_name,
        instances_by_type: &model.instances_by_type,
        ports_by_name: &model.ports_by_name,
        scan_registers_by_name: &model.scan_registers_by_name,
        data_registers_by_name: &model.data_registers_by_name,
        aliases_by_name: &model.aliases_by_name,
    };

    let directory = cache_path.parent().unwrap_or_else(|| Path::new("."));
    let mut temporary = tempfile::NamedTempFile::new_in(directory)?;
    {
        let mut writer = BufWriter::new(temporary.as_file_mut());
        bincode::serde::encode_into_std_write(&cache, &mut writer, bincode::config::standard())
            .map_err(|error| Error::new(&format!("Unable to encode ICL model cache: {error}")))?;
        writer.flush()?;
    }
    temporary
        .persist(cache_path)
        .map_err(|error| Error::new(&format!("Unable to persist ICL model cache: {error}")))?;
    Ok(())
}

fn source_hash(source: &[u8]) -> [u8; 32] {
    *blake3::hash(source).as_bytes()
}
