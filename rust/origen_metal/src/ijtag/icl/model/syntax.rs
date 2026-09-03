use super::super::parser::{ICLParser, Rule};
use super::super::{AccessLinkStandard, MuxType, PortType, SignalType};
use crate::{Error, Result};
use ahash::AHashMap as HashMap;
use pest::iterators::{Pair, Pairs};
use pest::Parser as PestParser;
use rayon::prelude::*;
use std::fs;
use std::path::Path;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct SymbolId(u32);

impl SymbolId {
    pub fn as_usize(self) -> usize {
        self.0 as usize
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SourceSpan {
    pub start: u32,
    pub end: u32,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct SyntaxId(u32);

impl SyntaxId {
    pub fn as_usize(self) -> usize {
        self.0 as usize
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum SyntaxKind {
    Root,
    Comment,
    NameSpace,
    UseNameSpace,
    Module,
    Port(PortType),
    Instance,
    ModuleReference,
    ScanRegister,
    DataRegister,
    LogicSignal,
    Mux(MuxType),
    MuxSelection(MuxType),
    OneHotScanGroup,
    OneHotDataGroup,
    ScanInterface,
    ScanInterfaceChain,
    AccessLink(AccessLinkStandard),
    GenericAccessLink,
    GenericAccessLinkBody,
    BsdlInstruction,
    BsdlEntity,
    ScanInterfaces,
    ActiveSignals,
    Alias,
    Enumeration,
    EnumerationItem,
    Parameter,
    LocalParameter,
    Attribute,
    Source,
    Enable,
    RefEnum,
    DefaultLoadValue,
    ActivePolarity(bool),
    DifferentialInvOf,
    FrequencyMultiplier,
    FrequencyDivider,
    Period,
    InputPortConnection,
    AllowBroadcastOnScanInterface,
    AddressValue,
    ScanInSource,
    CaptureSource,
    ResetValue,
    WriteEnSource,
    WriteDataSource,
    ReadCallBack,
    ReadDataSource,
    WriteCallBack,
    IProcReference,
    IProcArgument,
    PortReference,
    ScanInterfaceReference,
    AccessTogether,
    ApplyEndState,
    Identifier(SymbolId),
    VectorIdentifier,
    HierarchicalIdentifier,
    Index,
    Range,
    ParameterReference(SymbolId),
    StringLiteral,
    Number,
    TimeUnit,
    Signal(SignalType),
    Concatenation,
    Alternatives,
    Invert,
    IntegerExpression,
    IntegerTerm,
    Parentheses,
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    LogicExpression,
    LogicBitwiseExpression,
    LogicEqualityExpression,
    LogicConcatenation,
    BooleanAnd,
    BooleanOr,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    BooleanNot,
    Equal,
    NotEqual,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct SyntaxNode {
    pub(crate) kind: SyntaxKind,
    pub(crate) span: SourceSpan,
    pub(crate) first_child: Option<SyntaxId>,
    pub(crate) next_sibling: Option<SyntaxId>,
}

#[derive(Debug)]
pub(super) struct ParsedStorage {
    pub(super) source: String,
    pub(super) source_file: Option<String>,
    pub(super) nodes: Vec<SyntaxNode>,
    pub(super) symbols: Vec<String>,
    pub(super) symbol_ids: HashMap<String, SymbolId>,
}

#[derive(Clone, Debug)]
pub struct ParsedIcl {
    pub(super) inner: Arc<ParsedStorage>,
}

impl ParsedIcl {
    pub(super) fn from_parts(
        source: String,
        source_file: Option<String>,
        nodes: Vec<SyntaxNode>,
        symbols: Vec<String>,
    ) -> Self {
        let symbol_ids = symbols
            .iter()
            .enumerate()
            .map(|(index, symbol)| (symbol.clone(), SymbolId(index as u32)))
            .collect();
        Self {
            inner: Arc::new(ParsedStorage {
                source,
                source_file,
                nodes,
                symbols,
                symbol_ids,
            }),
        }
    }
    pub fn source(&self) -> &str {
        &self.inner.source
    }

    pub fn source_file(&self) -> Option<&str> {
        self.inner.source_file.as_deref()
    }

    pub fn root(&self) -> SyntaxId {
        SyntaxId(0)
    }

    pub fn kind(&self, id: SyntaxId) -> SyntaxKind {
        self.inner.nodes[id.as_usize()].kind
    }

    pub fn span(&self, id: SyntaxId) -> SourceSpan {
        self.inner.nodes[id.as_usize()].span
    }

    pub fn text(&self, span: SourceSpan) -> &str {
        &self.inner.source[span.start as usize..span.end as usize]
    }

    pub fn node_text(&self, id: SyntaxId) -> &str {
        self.text(self.span(id))
    }

    pub fn symbol(&self, id: SymbolId) -> &str {
        &self.inner.symbols[id.as_usize()]
    }

    pub fn symbol_id(&self, value: &str) -> Option<SymbolId> {
        self.inner.symbol_ids.get(value).copied()
    }

    pub fn node_count(&self) -> usize {
        self.inner.nodes.len()
    }

    pub fn syntax_ids(&self) -> impl Iterator<Item = SyntaxId> + '_ {
        (0..self.inner.nodes.len()).map(|index| SyntaxId(index as u32))
    }

    pub fn children(&self, id: SyntaxId) -> Children<'_> {
        Children {
            parsed: self,
            next: self.inner.nodes[id.as_usize()].first_child,
        }
    }

    pub(crate) fn node(&self, id: SyntaxId) -> &SyntaxNode {
        &self.inner.nodes[id.as_usize()]
    }
}

pub struct Children<'a> {
    parsed: &'a ParsedIcl,
    next: Option<SyntaxId>,
}

impl Iterator for Children<'_> {
    type Item = SyntaxId;

    fn next(&mut self) -> Option<Self::Item> {
        let id = self.next?;
        self.next = self.parsed.node(id).next_sibling;
        Some(id)
    }
}

#[derive(Clone, Debug)]
pub struct Parser {
    preserve_comments: bool,
    threads: usize,
}

impl Default for Parser {
    fn default() -> Self {
        Self {
            preserve_comments: false,
            threads: std::thread::available_parallelism()
                .map(|count| count.get())
                .unwrap_or(1),
        }
    }
}

impl Parser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn preserve_comments(mut self) -> Self {
        self.preserve_comments = true;
        self
    }

    pub fn threads(mut self, threads: usize) -> Self {
        self.threads = threads.max(1);
        self
    }

    pub fn from_file(&self, path: &Path) -> Result<ParsedIcl> {
        if !path.exists() {
            return Err(Error::new(&format!(
                "File does not exist: {}",
                path.display()
            )));
        }
        let source = fs::read_to_string(path)?;
        self.parse_owned(source, Some(path.display().to_string()))
            .map_err(|e| {
                let display_path = path
                    .canonicalize()
                    .unwrap_or_else(|_| path.to_path_buf())
                    .display()
                    .to_string();
                Error::new(&format!("Error parsing file {}:\n{}", display_path, e.msg))
            })
    }

    pub fn from_str(&self, source: &str) -> Result<ParsedIcl> {
        self.parse_owned(source.to_string(), None)
    }

    pub fn load_or_elaborate(
        &self,
        source_path: &Path,
        top: Option<&str>,
        cache_path: &Path,
    ) -> Result<super::IclModel> {
        if let Some(model) =
            super::cache::load(source_path, cache_path, top, self.preserve_comments)?
        {
            return Ok(model);
        }
        let parsed = self.from_file(source_path)?;
        let model = if let Some(top) = top {
            parsed.elaborate(top)?
        } else {
            parsed.elaborate_unique_root()?
        };
        super::cache::save(&model, cache_path, top, self.preserve_comments)?;
        Ok(model)
    }

    fn parse_owned(&self, source: String, source_file: Option<String>) -> Result<ParsedIcl> {
        if source.len() > u32::MAX as usize {
            return Err(Error::new(
                "ICL sources larger than 4 GiB are not supported",
            ));
        }
        let builder = if self.threads > 1 && source.len() >= 1_000_000 {
            build_parallel(&source, self.preserve_comments, self.threads)?
        } else {
            let pairs = ICLParser::parse(Rule::icl_source, &source)
                .map_err(|e| Error::new(&e.to_string()))?;
            build(&source, pairs, self.preserve_comments)?
        };
        Ok(ParsedIcl {
            inner: Arc::new(ParsedStorage {
                source,
                source_file,
                nodes: builder.nodes,
                symbols: builder.symbols,
                symbol_ids: builder.symbol_ids,
            }),
        })
    }
}

enum Action {
    Open(SyntaxKind),
    Leaf(SyntaxKind),
    Transparent,
}

struct Builder {
    nodes: Vec<SyntaxNode>,
    last_child: Vec<Option<SyntaxId>>,
    symbols: Vec<String>,
    symbol_ids: HashMap<String, SymbolId>,
}

impl Builder {
    fn new(source_len: usize) -> Self {
        Self {
            nodes: vec![SyntaxNode {
                kind: SyntaxKind::Root,
                span: SourceSpan {
                    start: 0,
                    end: source_len as u32,
                },
                first_child: None,
                next_sibling: None,
            }],
            last_child: vec![None],
            symbols: Vec::new(),
            symbol_ids: HashMap::new(),
        }
    }

    fn intern(&mut self, value: &str) -> SymbolId {
        if let Some(id) = self.symbol_ids.get(value) {
            return *id;
        }
        let id = SymbolId(self.symbols.len() as u32);
        let value = value.to_string();
        self.symbols.push(value.clone());
        self.symbol_ids.insert(value, id);
        id
    }

    fn add(&mut self, parent: SyntaxId, kind: SyntaxKind, span: SourceSpan) -> SyntaxId {
        let id = SyntaxId(self.nodes.len() as u32);
        self.nodes.push(SyntaxNode {
            kind,
            span,
            first_child: None,
            next_sibling: None,
        });
        self.last_child.push(None);
        if let Some(previous) = self.last_child[parent.as_usize()] {
            self.nodes[previous.as_usize()].next_sibling = Some(id);
        } else {
            self.nodes[parent.as_usize()].first_child = Some(id);
        }
        self.last_child[parent.as_usize()] = Some(id);
        id
    }

    fn merge(&mut self, other: Builder, offset: u32) {
        let mut symbol_map = Vec::with_capacity(other.symbols.len());
        for symbol in &other.symbols {
            symbol_map.push(self.intern(symbol));
        }
        let base = self.nodes.len() as u32 - 1;
        let remap_id = |id: SyntaxId| SyntaxId(base + id.0);
        let remap_kind = |kind: SyntaxKind| match kind {
            SyntaxKind::Identifier(symbol) => SyntaxKind::Identifier(symbol_map[symbol.as_usize()]),
            SyntaxKind::ParameterReference(symbol) => {
                SyntaxKind::ParameterReference(symbol_map[symbol.as_usize()])
            }
            other => other,
        };

        for node in other.nodes.iter().skip(1) {
            self.nodes.push(SyntaxNode {
                kind: remap_kind(node.kind),
                span: SourceSpan {
                    start: node.span.start + offset,
                    end: node.span.end + offset,
                },
                first_child: node.first_child.map(remap_id),
                next_sibling: node.next_sibling.map(remap_id),
            });
        }
        for child in other.last_child.iter().skip(1) {
            self.last_child.push(child.map(remap_id));
        }

        if let Some(local_first) = other.nodes[0].first_child {
            let first = remap_id(local_first);
            if let Some(previous) = self.last_child[0] {
                self.nodes[previous.as_usize()].next_sibling = Some(first);
            } else {
                self.nodes[0].first_child = Some(first);
            }
            self.last_child[0] = other.last_child[0].map(remap_id);
        }
    }
}

fn span(pair: &Pair<'_, Rule>) -> SourceSpan {
    let span = pair.as_span();
    SourceSpan {
        start: span.start() as u32,
        end: span.end() as u32,
    }
}

fn build(source: &str, mut pairs: Pairs<'_, Rule>, preserve_comments: bool) -> Result<Builder> {
    let root_pair = pairs.next().expect("successful ICL parse has a root pair");
    let mut builder = Builder::new(source.len());
    let mut frames: Vec<(Pairs<'_, Rule>, SyntaxId)> = vec![(root_pair.into_inner(), SyntaxId(0))];

    while !frames.is_empty() {
        let (next, parent) = {
            let (pairs, parent) = frames.last_mut().unwrap();
            (pairs.next(), *parent)
        };
        if let Some(pair) = next {
            let pair_span = span(&pair);
            match classify(&pair, &mut builder, preserve_comments) {
                Action::Leaf(kind) => {
                    builder.add(parent, kind, pair_span);
                }
                Action::Open(kind) => {
                    let id = builder.add(parent, kind, pair_span);
                    frames.push((pair.into_inner(), id));
                }
                Action::Transparent => {
                    let inner = pair.into_inner();
                    if inner.peek().is_some() {
                        frames.push((inner, parent));
                    }
                }
            }
        } else {
            frames.pop();
        }
    }

    Ok(builder)
}

#[derive(Clone, Copy)]
enum TopLevelKind {
    Module,
    NameSpace,
    UseNameSpace,
}

#[derive(Clone, Copy)]
struct TopLevelItem {
    start: usize,
    end: usize,
    kind: TopLevelKind,
}

fn build_parallel(source: &str, preserve_comments: bool, threads: usize) -> Result<Builder> {
    let items = split_top_level(source)?;
    let worker_count = threads.min(items.len()).max(1);
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(worker_count)
        .build()
        .map_err(|error| Error::new(&format!("Unable to create ICL parser workers: {error}")))?;
    let mut parsed_items: Vec<_> = pool.install(|| {
        items
            .par_iter()
            .map(|item| {
                let rule = match item.kind {
                    TopLevelKind::Module => Rule::module_source,
                    TopLevelKind::NameSpace => Rule::namespace_source,
                    TopLevelKind::UseNameSpace => Rule::use_namespace_source,
                };
                let slice = &source[item.start..item.end];
                let result = ICLParser::parse(rule, slice)
                    .map_err(|error| Error::new(&error.to_string()))
                    .and_then(|pairs| build(slice, pairs, preserve_comments));
                (item.start, result)
            })
            .collect()
    });
    parsed_items.sort_by_key(|(offset, _)| *offset);

    let mut builder = Builder::new(source.len());
    for (offset, result) in parsed_items {
        match result {
            Ok(item) => builder.merge(item, offset as u32),
            Err(error) => {
                let line = source[..offset]
                    .bytes()
                    .filter(|byte| *byte == b'\n')
                    .count()
                    + 1;
                return Err(Error::new(&format!(
                    "Error in top-level ICL item beginning near line {line}:\n{}",
                    error.msg
                )));
            }
        }
    }
    Ok(builder)
}

fn split_top_level(source: &str) -> Result<Vec<TopLevelItem>> {
    let bytes = source.as_bytes();
    let mut items: Vec<TopLevelItem> = Vec::new();
    let mut cursor = 0usize;
    while cursor < bytes.len() {
        let item_start = cursor;
        let token_start = skip_layout(bytes, cursor)?;
        if token_start == bytes.len() {
            if let Some(last) = items.last_mut() {
                last.end = bytes.len();
            }
            break;
        }
        let mut token_end = token_start;
        while token_end < bytes.len()
            && (bytes[token_end].is_ascii_alphanumeric() || bytes[token_end] == b'_')
        {
            token_end += 1;
        }
        let kind = match &source[token_start..token_end] {
            "Module" => TopLevelKind::Module,
            "NameSpace" => TopLevelKind::NameSpace,
            "UseNameSpace" => TopLevelKind::UseNameSpace,
            token => {
                return Err(Error::new(&format!(
                    "Unexpected top-level ICL token {token:?} at byte {token_start}"
                )))
            }
        };
        let end = scan_item_end(bytes, token_end, matches!(kind, TopLevelKind::Module))?;
        items.push(TopLevelItem {
            start: item_start,
            end,
            kind,
        });
        cursor = end;
    }
    if items.is_empty() {
        return Err(Error::new(
            "ICL source must contain at least one source item",
        ));
    }
    Ok(items)
}

fn skip_layout(bytes: &[u8], mut cursor: usize) -> Result<usize> {
    loop {
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if bytes.get(cursor..cursor + 2) == Some(b"//") {
            cursor += 2;
            while cursor < bytes.len() && bytes[cursor] != b'\n' {
                cursor += 1;
            }
        } else if bytes.get(cursor..cursor + 2) == Some(b"/*") {
            let Some(end) = bytes[cursor + 2..]
                .windows(2)
                .position(|window| window == b"*/")
            else {
                return Err(Error::new("Unterminated block comment in ICL source"));
            };
            cursor += end + 4;
        } else {
            return Ok(cursor);
        }
    }
}

fn scan_item_end(bytes: &[u8], mut cursor: usize, is_module: bool) -> Result<usize> {
    let mut depth = 0usize;
    let mut saw_open = false;
    while cursor < bytes.len() {
        match bytes[cursor] {
            b'"' => {
                cursor += 1;
                while cursor < bytes.len() {
                    match bytes[cursor] {
                        b'\\' => cursor = (cursor + 2).min(bytes.len()),
                        b'"' => {
                            cursor += 1;
                            break;
                        }
                        _ => cursor += 1,
                    }
                }
            }
            b'/' if bytes.get(cursor + 1) == Some(&b'/') => {
                cursor += 2;
                while cursor < bytes.len() && bytes[cursor] != b'\n' {
                    cursor += 1;
                }
            }
            b'/' if bytes.get(cursor + 1) == Some(&b'*') => {
                let Some(end) = bytes[cursor + 2..]
                    .windows(2)
                    .position(|window| window == b"*/")
                else {
                    return Err(Error::new("Unterminated block comment in ICL source"));
                };
                cursor += end + 4;
            }
            b'{' if is_module => {
                saw_open = true;
                depth += 1;
                cursor += 1;
            }
            b'}' if is_module => {
                if depth == 0 {
                    return Err(Error::new("Unexpected closing brace in ICL source"));
                }
                depth -= 1;
                cursor += 1;
                if saw_open && depth == 0 {
                    return Ok(cursor);
                }
            }
            b';' if !is_module => return Ok(cursor + 1),
            _ => cursor += 1,
        }
    }
    Err(Error::new("Unterminated top-level ICL source item"))
}

fn has_direct_child(pair: &Pair<'_, Rule>, rule: Rule) -> bool {
    pair.clone()
        .into_inner()
        .any(|child| child.as_rule() == rule)
}

fn direct_child_count(pair: &Pair<'_, Rule>, rule: Rule) -> usize {
    pair.clone()
        .into_inner()
        .filter(|child| child.as_rule() == rule)
        .count()
}

fn classify(pair: &Pair<'_, Rule>, builder: &mut Builder, preserve_comments: bool) -> Action {
    let text = pair.as_str();
    let kind = match pair.as_rule() {
        Rule::namespace_def => SyntaxKind::NameSpace,
        Rule::use_namespace_def => SyntaxKind::UseNameSpace,
        Rule::module_def => SyntaxKind::Module,
        Rule::scan_in_port_def => SyntaxKind::Port(PortType::ScanIn),
        Rule::scan_out_port_def => SyntaxKind::Port(PortType::ScanOut),
        Rule::shift_en_port_def => SyntaxKind::Port(PortType::ShiftEn),
        Rule::capture_en_port_def => SyntaxKind::Port(PortType::CaptureEn),
        Rule::update_en_port_def => SyntaxKind::Port(PortType::UpdateEn),
        Rule::data_in_port_def => SyntaxKind::Port(PortType::DataIn),
        Rule::data_out_port_def => SyntaxKind::Port(PortType::DataOut),
        Rule::to_shift_en_port_def => SyntaxKind::Port(PortType::ToShiftEn),
        Rule::to_update_en_port_def => SyntaxKind::Port(PortType::ToUpdateEn),
        Rule::to_capture_en_port_def => SyntaxKind::Port(PortType::ToCaptureEn),
        Rule::select_port_def => SyntaxKind::Port(PortType::Select),
        Rule::to_select_port_def => SyntaxKind::Port(PortType::ToSelect),
        Rule::reset_port_def => SyntaxKind::Port(PortType::Reset),
        Rule::to_reset_port_def => SyntaxKind::Port(PortType::ToReset),
        Rule::tms_port_def => SyntaxKind::Port(PortType::Tms),
        Rule::to_tms_port_def => SyntaxKind::Port(PortType::ToTms),
        Rule::tck_port_def => SyntaxKind::Port(PortType::Tck),
        Rule::to_tck_port_def => SyntaxKind::Port(PortType::ToTck),
        Rule::clock_port_def => SyntaxKind::Port(PortType::Clock),
        Rule::to_clock_port_def => SyntaxKind::Port(PortType::ToClock),
        Rule::trst_port_def => SyntaxKind::Port(PortType::Trst),
        Rule::to_trst_port_def => SyntaxKind::Port(PortType::ToTrst),
        Rule::to_ir_select_port_def => SyntaxKind::Port(PortType::ToIrSelect),
        Rule::address_port_def => SyntaxKind::Port(PortType::Address),
        Rule::write_en_port_def => SyntaxKind::Port(PortType::WriteEn),
        Rule::read_en_port_def => SyntaxKind::Port(PortType::ReadEn),
        Rule::instance_def => SyntaxKind::Instance,
        Rule::module_reference => SyntaxKind::ModuleReference,
        Rule::scan_register_def => SyntaxKind::ScanRegister,
        Rule::data_register_def => SyntaxKind::DataRegister,
        Rule::logic_signal_def => SyntaxKind::LogicSignal,
        Rule::scan_mux_def => SyntaxKind::Mux(MuxType::Scan),
        Rule::data_mux_def => SyntaxKind::Mux(MuxType::Data),
        Rule::clock_mux_def => SyntaxKind::Mux(MuxType::Clock),
        Rule::scan_mux_selection => SyntaxKind::MuxSelection(MuxType::Scan),
        Rule::data_mux_selection => SyntaxKind::MuxSelection(MuxType::Data),
        Rule::clock_mux_selection => SyntaxKind::MuxSelection(MuxType::Clock),
        Rule::one_hot_scan_group_def => SyntaxKind::OneHotScanGroup,
        Rule::one_hot_data_group_def => SyntaxKind::OneHotDataGroup,
        Rule::scan_interface_def => SyntaxKind::ScanInterface,
        Rule::scan_interface_chain_def => SyntaxKind::ScanInterfaceChain,
        Rule::access_link_1149_def => SyntaxKind::AccessLink(if text.contains("STD_1149_1_2001") {
            AccessLinkStandard::Std1149_1_2001
        } else {
            AccessLinkStandard::Std1149_1_2013
        }),
        Rule::access_link_generic_def => SyntaxKind::GenericAccessLink,
        Rule::bsdl_instruction => SyntaxKind::BsdlInstruction,
        Rule::bsdl_entity_def => SyntaxKind::BsdlEntity,
        Rule::bsdl_selection => {
            if text.trim_start().starts_with("ScanInterface") {
                SyntaxKind::ScanInterfaces
            } else {
                SyntaxKind::ActiveSignals
            }
        }
        Rule::access_link_scan_interface_name => SyntaxKind::ScanInterfaceReference,
        Rule::access_link_active_signal_name
        | Rule::one_hot_scan_group_item
        | Rule::one_hot_data_group_port_source
        | Rule::scan_interface_port_def => SyntaxKind::PortReference,
        Rule::alias_def => SyntaxKind::Alias,
        Rule::enum_def => SyntaxKind::Enumeration,
        Rule::enum_item => SyntaxKind::EnumerationItem,
        Rule::parameter_def => SyntaxKind::Parameter,
        Rule::local_parameter_def => SyntaxKind::LocalParameter,
        Rule::attribute_def => SyntaxKind::Attribute,
        Rule::source_def => SyntaxKind::Source,
        Rule::enable_def => SyntaxKind::Enable,
        Rule::ref_enum_def => SyntaxKind::RefEnum,
        Rule::default_load_value_def => SyntaxKind::DefaultLoadValue,
        Rule::active_polarity_def => {
            let polarity = pair
                .clone()
                .into_inner()
                .find(|child| child.as_rule() == Rule::polarity_value)
                .expect("validated polarity must contain a value");
            SyntaxKind::ActivePolarity(polarity.as_str() == "1")
        }
        Rule::differential_inv_of_def => SyntaxKind::DifferentialInvOf,
        Rule::frequency_multiplier_def => SyntaxKind::FrequencyMultiplier,
        Rule::frequency_divider_def => SyntaxKind::FrequencyDivider,
        Rule::period_def => SyntaxKind::Period,
        Rule::input_port_connection => SyntaxKind::InputPortConnection,
        Rule::allow_broadcast_def => SyntaxKind::AllowBroadcastOnScanInterface,
        Rule::instance_address_value | Rule::data_register_address_value => {
            SyntaxKind::AddressValue
        }
        Rule::scan_in_source_def => SyntaxKind::ScanInSource,
        Rule::capture_source_def => SyntaxKind::CaptureSource,
        Rule::reset_value_def => SyntaxKind::ResetValue,
        Rule::write_en_source_def => SyntaxKind::WriteEnSource,
        Rule::write_data_source_def => SyntaxKind::WriteDataSource,
        Rule::read_callback_def => SyntaxKind::ReadCallBack,
        Rule::read_data_source_def => SyntaxKind::ReadDataSource,
        Rule::write_callback_def => SyntaxKind::WriteCallBack,
        Rule::iproc_reference => SyntaxKind::IProcReference,
        Rule::vector_id => SyntaxKind::VectorIdentifier,
        Rule::hier_port => SyntaxKind::HierarchicalIdentifier,
        Rule::index => SyntaxKind::Index,
        Rule::range => SyntaxKind::Range,
        Rule::hier_data_signal => SyntaxKind::Signal(SignalType::HierarchicalData),
        Rule::reset_signal => SyntaxKind::Signal(SignalType::Reset),
        Rule::scan_signal => SyntaxKind::Signal(SignalType::Scan),
        Rule::data_signal => SyntaxKind::Signal(SignalType::Data),
        Rule::clock_signal => SyntaxKind::Signal(SignalType::Clock),
        Rule::tck_signal => SyntaxKind::Signal(SignalType::Tck),
        Rule::tms_signal => SyntaxKind::Signal(SignalType::Tms),
        Rule::trst_signal => SyntaxKind::Signal(SignalType::Trst),
        Rule::shift_en_signal => SyntaxKind::Signal(SignalType::ShiftEn),
        Rule::capture_en_signal => SyntaxKind::Signal(SignalType::CaptureEn),
        Rule::update_en_signal => SyntaxKind::Signal(SignalType::UpdateEn),
        Rule::concat_signal if direct_child_count(pair, Rule::inverted_signal) > 1 => {
            SyntaxKind::Concatenation
        }
        Rule::concat_hier_data_signal
            if direct_child_count(pair, Rule::inverted_hier_data_signal) > 1 =>
        {
            SyntaxKind::Concatenation
        }
        Rule::concat_number if direct_child_count(pair, Rule::inverted_number) > 1 => {
            SyntaxKind::Concatenation
        }
        Rule::concat_string => SyntaxKind::Concatenation,
        Rule::concat_number_list if direct_child_count(pair, Rule::concat_number) > 1 => {
            SyntaxKind::Alternatives
        }
        Rule::integer_expr if has_direct_child(pair, Rule::integer_add_op) => {
            SyntaxKind::IntegerExpression
        }
        Rule::integer_term if has_direct_child(pair, Rule::integer_mul_op) => {
            SyntaxKind::IntegerTerm
        }
        Rule::integer_paren | Rule::logic_paren => SyntaxKind::Parentheses,
        Rule::logic_expr | Rule::logic_bool_expr => SyntaxKind::LogicExpression,
        Rule::logic_bitwise_expr => SyntaxKind::LogicBitwiseExpression,
        Rule::logic_equality_expr => SyntaxKind::LogicEqualityExpression,
        Rule::logic_concat_expr => SyntaxKind::LogicConcatenation,
        Rule::scalar_id => return Action::Leaf(SyntaxKind::Identifier(builder.intern(text))),
        Rule::parameter_ref => {
            return Action::Leaf(SyntaxKind::ParameterReference(builder.intern(&text[1..])))
        }
        Rule::string => return Action::Leaf(SyntaxKind::StringLiteral),
        Rule::sized_bin_number
        | Rule::sized_dec_number
        | Rule::sized_hex_number
        | Rule::unsized_bin_number
        | Rule::unsized_dec_number
        | Rule::unsized_hex_number
        | Rule::pos_int => return Action::Leaf(SyntaxKind::Number),
        Rule::time_unit => return Action::Leaf(SyntaxKind::TimeUnit),
        Rule::iproc_argument => return Action::Leaf(SyntaxKind::IProcArgument),
        Rule::generic_access_link_block => return Action::Leaf(SyntaxKind::GenericAccessLinkBody),
        Rule::invert => return Action::Leaf(SyntaxKind::Invert),
        Rule::integer_add_op => {
            return Action::Leaf(if text == "+" {
                SyntaxKind::Add
            } else {
                SyntaxKind::Subtract
            })
        }
        Rule::integer_mul_op => {
            return Action::Leaf(match text {
                "*" => SyntaxKind::Multiply,
                "/" => SyntaxKind::Divide,
                _ => SyntaxKind::Modulo,
            })
        }
        Rule::logic_bool_op => {
            return Action::Leaf(if text == "&&" {
                SyntaxKind::BooleanAnd
            } else {
                SyntaxKind::BooleanOr
            })
        }
        Rule::logic_bitwise_op | Rule::logic_reduction_op => {
            return Action::Leaf(match text {
                "&" => SyntaxKind::BitwiseAnd,
                "|" => SyntaxKind::BitwiseOr,
                _ => SyntaxKind::BitwiseXor,
            })
        }
        Rule::logic_equality_op => {
            return Action::Leaf(if text == "==" {
                SyntaxKind::Equal
            } else {
                SyntaxKind::NotEqual
            })
        }
        Rule::logic_unary_op => {
            return Action::Leaf(if text == "!" {
                SyntaxKind::BooleanNot
            } else {
                SyntaxKind::Invert
            })
        }
        Rule::COMMENT if preserve_comments => return Action::Leaf(SyntaxKind::Comment),
        _ => return Action::Transparent,
    };
    Action::Open(kind)
}
