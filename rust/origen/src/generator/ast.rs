use super::processors::ToString;
use super::stil;
use crate::generator::processor::*;
use crate::generator::TestManager;
use crate::TEST;
use crate::{Error, Result};
use num_bigint::BigUint;
use std::fmt;

#[macro_export]
macro_rules! node {
    ( $attr:ident, $( $x:expr ),* ) => {
        {
            Node::new(Attrs::$attr($( $x ),*))
        }
    };
    ( $attr:ident ) => {
        {
            Node::new(Attrs::$attr)
        }
    };
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum Attrs {
    // A meta-node type, used to indicate a node who's children should be placed inline at the given location
    _Inline,

    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    //// Test (pat gen) nodes
    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    Test(String),
    Comment(u8, String), // level, msg
    PinWrite(Id, u128),
    PinVerify(Id, u128),
    RegWrite(Id, BigUint, Option<BigUint>, Option<String>), // reg_id, data, overlay_enable, overlay_str
    RegVerify(
        Id,
        BigUint,
        Option<BigUint>,
        Option<BigUint>,
        Option<BigUint>,
        Option<String>,
    ), // reg_id, data, verify_enable, capture_enable, overlay_enable, overlay_str
    JTAGWriteIR(u32, BigUint, Option<BigUint>, Option<String>), // size, data, overlay_enable, overlay_str
    JTAGVerifyIR(
        u32,
        BigUint,
        Option<BigUint>,
        Option<BigUint>,
        Option<BigUint>,
        Option<String>,
    ), // size, data, verify_enable, capture_enable, overlay_enable, overlay_str
    JTAGWriteDR(u32, BigUint, Option<BigUint>, Option<String>), // size, data, overlay_enable, overlay_str
    JTAGVerifyDR(
        u32,
        BigUint,
        Option<BigUint>,
        Option<BigUint>,
        Option<BigUint>,
        Option<String>,
    ), // size, data, verify_enable, capture_enable, overlay_enable, overlay_str
    Cycle(u32, bool), // repeat (0 not allowed), compressable

    //// Teradyne custom nodes

    //// Advantest custom nodes

    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    //// Flow (prog gen) nodes
    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    Flow(String),

    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    //// STIL
    //////////////////////////////////////////////////////////////////////////////////////////////////////////
    STIL,
    STILUnknown,
    STILVersion(u32, u32), // major, minor
    STILHeader,
    STILTitle(String),
    STILDate(String),
    STILSource(String),
    STILHistory,
    STILAnnotation(String),
    STILInclude(String, Option<String>),
    STILSignals,
    STILSignal(String, stil::SignalType), // name, type
    STILTermination(stil::Termination),
    STILDefaultState(stil::State),
    STILBase(stil::Base, String),
    STILAlignment(stil::Alignment),
    STILScanIn(u32),
    STILScanOut(u32),
    STILDataBitCount(u32),
    STILSignalGroups(Option<String>),
    STILSignalGroup(String),
    STILSigRefExpr,
    STILName(String),
    STILExpr,
    STILSIUnit(String),
    STILEngPrefix(String),
    STILAdd,
    STILSubtract,
    STILMultiply,
    STILDivide,
    STILParens,
    STILNumberWithUnit,
    STILNumber,
    STILInteger(u64),
    STILSignedInteger(i64),
    STILPoint,
    STILExp,
    STILMinus,
    STILPatternExec(Option<String>),
    STILCategory(String),
    STILSelector(String),
    STILTimingRef(String),
    STILPatternBurstRef(String),
    STILPatternBurst(String),
    STILSignalGroupsRef(String),
    STILMacroDefs(String),
    STILProcedures(String),
    STILScanStructures(String),
    STILStart(String),
    STILStop(String),
    STILTerminations,
    STILTerminationItem,
    STILPatList,
    STILPat(String),
    STILLabel(String),
    STILTiming(Option<String>),
    STILWaveformTable(String),
    STILPeriod,
    STILInherit(String),
    STILSubWaveforms,
    STILSubWaveform,
    STILWaveforms,
    STILWaveform,
    STILWFChar(String),
    STILEvent,
    STILEventList(Vec<char>),
}

impl Node {
    /// Returns a new node which is the output of the node processed by the given processor.
    /// Returning None means that the processor has decided that the node should be removed
    /// from the next stage AST.
    pub fn process(&self, processor: &mut dyn Processor) -> Option<Node> {
        let r = processor.on_node(&self);
        self.process_return_code(r, processor)
    }
}

#[derive(Clone)]
pub struct AST {
    nodes: Vec<Node>,
}

impl AST {
    /// Create a new AST with the given node as the top-level
    pub fn new(node: Node) -> AST {
        AST { nodes: vec![node] }
    }

    /// Push a new terminal node into the AST
    pub fn push(&mut self, node: Node) {
        self.nodes.last_mut().unwrap().add_child(node);
    }

    /// Push a new node into the AST and leave it open, meaning that all new nodes
    /// added to the AST will be inserted as children of this node until it is closed.
    /// A reference ID is returned and the caller should save this and provide it again
    /// when calling close(). If the reference does not match the expected an error will
    /// be raised. This will catch any cases of AST application code forgetting to close
    /// a node before closing one of its parents.
    pub fn push_and_open(&mut self, node: Node) -> usize {
        self.nodes.push(node);
        self.nodes.len()
    }

    /// Close the currently open node
    pub fn close(&mut self, ref_id: usize) -> Result<()> {
        if self.nodes.len() != ref_id {
            return Err(Error::new(
                "Attempt to close a parent AST node without first closing all its children, it looks like you have either forgotten to close an open node or given the wrong reference ID"
            ));
        }
        if ref_id == 1 {
            return Err(Error::new("The top-level AST node can never be closed"));
        }
        let n = self.nodes.pop().unwrap();
        if let Some(node) = self.nodes.last_mut() {
            node.add_child(n);
        }
        Ok(())
    }

    /// Replace the node n - offset with the given node, use offset = 0 to
    /// replace the last node that was pushed.
    /// Fails if the AST has no children yet or if the offset is otherwise out
    /// of range.
    pub fn replace(&mut self, node: Node, mut offset: usize) -> Result<()> {
        let mut node_offset = 0;
        let mut child_offset = 0;
        let mut root_node = false;
        let node_len = self.nodes.len();
        while offset > 0 {
            let node = &self.nodes[node_len - 1 - node_offset];
            let num_children = node.children.len();
            // If node to be replaced lies in this node's children
            if num_children > offset {
                child_offset = offset;
                offset = 0;
            // If node to be replaced is this node itself
            } else if num_children == offset {
                root_node = true;
                offset = 0;
            // The node to be replaced lies outside this node
            } else {
                node_offset += 1;
                offset -= num_children + 1;
                child_offset = 0;
            }
        }
        let index = node_len - 1 - node_offset;
        let mut n = self.nodes.remove(index);
        if root_node {
            self.nodes.insert(index, node);
        } else {
            n.replace_child(node, child_offset)?;
            self.nodes.insert(index, n);
        }
        Ok(())
    }

    /// Insert the node at position n - offset, using offset = 0 is equivalent
    /// calling push().
    pub fn insert(&mut self, node: Node, mut offset: usize) -> Result<()> {
        let mut node_offset = 0;
        let mut child_offset = 0;
        let node_len = self.nodes.len();
        while offset > 0 {
            let node = &self.nodes[node_len - 1 - node_offset];
            let num_children = node.children.len();
            // If node is to be inserted into this node's children
            if num_children >= offset {
                child_offset = offset;
                offset = 0;
            // The parent node lies outside this node
            } else {
                node_offset += 1;
                offset -= num_children + 1;
                child_offset = 0;
            }
        }
        let index = node_len - 1 - node_offset;
        let mut n = self.nodes.remove(index);
        n.insert_child(node, child_offset)?;
        self.nodes.insert(index, n);
        Ok(())
    }

    /// Returns a copy of node n - offset, an offset of 0 means
    /// the last node pushed.
    /// Fails if the offset is out of range.
    pub fn get(&self, mut offset: usize) -> Result<Node> {
        let mut node_offset = 0;
        let mut child_offset = 0;
        let mut root_node = false;
        let node_len = self.nodes.len();
        while offset > 0 {
            let node = &self.nodes[node_len - 1 - node_offset];
            let num_children = node.children.len();
            // If node to be returned lies in this node's children
            if num_children > offset {
                child_offset = offset;
                offset = 0;
            // If node to be returned is this node itself
            } else if num_children == offset {
                root_node = true;
                offset = 0;
            // The node to be returned lies outside this node
            } else {
                node_offset += 1;
                offset -= num_children + 1;
                child_offset = 0;
            }
        }
        let index = node_len - 1 - node_offset;
        let n = &self.nodes[index];
        if root_node {
            Ok(self.nodes[index].clone())
        } else {
            Ok(n.get_child(child_offset)?)
        }
    }

    /// Clear the current AST and start a new one with the given node at the top-level
    pub fn start(&mut self, node: Node) {
        self.nodes.clear();
        self.nodes.push(node);
    }

    pub fn process(&self, process_fn: &dyn Fn(&Node) -> Node) -> Node {
        if self.nodes.len() > 1 {
            let node = self.to_node();
            process_fn(&node)
        } else {
            process_fn(&self.nodes[0])
        }
    }

    pub fn to_string(&self) -> String {
        if self.nodes.len() > 1 {
            let node = self.to_node();
            node.to_string()
        } else {
            self.nodes[0].to_string()
        }
    }

    /// Serializes the AST for import into Python
    pub fn to_pickle(&self) -> Vec<u8> {
        if self.nodes.len() > 1 {
            let node = self.to_node();
            node.to_pickle()
        } else {
            self.nodes[0].to_pickle()
        }
    }

    // Closes all currently open nodes into a new node but leaving the original state of the AST
    // unmodified.
    // This is like a snapshot of the current AST state, mainly useful for printing to the console
    // for debug.
    pub fn to_node(&self) -> Node {
        let mut node = self.nodes.last().unwrap().clone();
        let num = self.nodes.len();
        if num > 1 {
            for i in 1..num {
                let n = node;
                node = self.nodes[num - i - 1].clone();
                node.add_child(n);
            }
        }
        node
    }
}

impl fmt::Display for AST {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl fmt::Debug for AST {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl PartialEq<Node> for AST {
    fn eq(&self, node: &Node) -> bool {
        self.to_node() == *node
    }
}

impl PartialEq<TEST> for AST {
    fn eq(&self, test: &TEST) -> bool {
        self.to_node() == test.to_node()
    }
}

impl PartialEq<TestManager> for AST {
    fn eq(&self, test: &TestManager) -> bool {
        self.to_node() == test.to_node()
    }
}

type Id = usize;

#[derive(Clone, PartialEq, Serialize)]
pub struct Node {
    pub attrs: Attrs,
    pub meta: Option<Meta>,
    // This must remain private, potentially we could run across some limitation of this current children
    // implementation which could force us to change to (for example) an ID based system instead.
    // Ensuring all interation with this collection is via an API method will allow us to make such a
    // change under-the-hood without breaking the world.
    children: Vec<Box<Node>>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Meta {
    filename: Option<String>,
    lineno: Option<usize>,
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl PartialEq<AST> for Node {
    fn eq(&self, ast: &AST) -> bool {
        *self == ast.to_node()
    }
}

impl Node {
    pub fn new(attrs: Attrs) -> Node {
        Node {
            attrs: attrs,
            children: Vec::new(),
            meta: None,
        }
    }

    pub fn new_with_meta(attrs: Attrs, filename: Option<String>, lineno: Option<usize>) -> Node {
        Node {
            attrs: attrs,
            children: Vec::new(),
            meta: Some(Meta {
                filename: filename,
                lineno: lineno,
            }),
        }
    }

    fn inline(nodes: Vec<Box<Node>>) -> Node {
        Node {
            attrs: Attrs::_Inline,
            meta: None,
            children: nodes,
        }
    }

    pub fn to_string(&self) -> String {
        ToString::run(self)
    }

    /// Serializes the AST for import into Python
    pub fn to_pickle(&self) -> Vec<u8> {
        serde_pickle::to_vec(self, true).unwrap()
    }

    pub fn add_child(&mut self, node: Node) {
        self.children.push(Box::new(node));
    }

    pub fn insert_child(&mut self, node: Node, offset: usize) -> Result<()> {
        let len = self.children.len();
        if offset > len {
            return Err(Error::new(&format!(
                "An offset of {} was given to insert a child into a node with only {} children",
                offset, len
            )));
        }
        let index = self.children.len() - offset;
        self.children.insert(index, Box::new(node));
        Ok(())
    }

    /// Replace the child n - offset with the given node, use offset = 0 to
    /// replace the last child that was pushed.
    /// Fails if the node has no children or if the given offset is
    /// otherwise out of range.
    pub fn replace_child(&mut self, node: Node, offset: usize) -> Result<()> {
        let len = self.children.len();
        if len == 0 {
            return Err(Error::new(
                "Attempted to replace a child in a node with no children",
            ));
        } else if offset > len - 1 {
            return Err(Error::new(&format!(
                "An offset of {} was given to replace a child in a node with only {} children",
                offset, len
            )));
        }
        let index = self.children.len() - 1 - offset;
        self.children.remove(index);
        self.children.insert(index, Box::new(node));
        Ok(())
    }

    /// Returns a copy of child n - offset, an offset of 0 means
    /// the last child that was pushed.
    /// Fails if the node has no children or if the given offset is
    /// otherwise out of range.
    pub fn get_child(&self, offset: usize) -> Result<Node> {
        let len = self.children.len();
        if len == 0 {
            return Err(Error::new(
                "Attempted to get a child in a node with no children",
            ));
        } else if offset > len - 1 {
            return Err(Error::new(&format!(
                "An offset of {} was given to get a child in a node with only {} children",
                offset, len
            )));
        }
        let index = self.children.len() - 1 - offset;
        Ok(*self.children[index].clone())
    }

    fn process_return_code(&self, code: Return, processor: &mut dyn Processor) -> Option<Node> {
        match code {
            Return::None => None,
            Return::ProcessChildren => Some(self.process_children(processor)),
            Return::Unmodified => Some(self.clone()),
            Return::Replace(node) => Some(node),
            // We can't return multiple nodes from this function, so we return them
            // wrapped in a meta-node and the process_children method will identify
            // this and remove the wrapper to inline the contained nodes.
            Return::Unwrap => Some(Node::inline(self.children.clone())),
            Return::Inline(nodes) => Some(Node::inline(
                nodes.into_iter().map(|n| Box::new(n)).collect(),
            )),
            Return::InlineBoxed(nodes) => Some(Node::inline(nodes)),
        }
    }

    /// Returns a new node which is a copy of self with its children replaced
    /// by their processed counterparts.
    pub fn process_children(&self, processor: &mut dyn Processor) -> Node {
        if self.children.len() == 0 {
            return self.clone();
        }
        let mut nodes: Vec<Box<Node>> = Vec::new();
        for child in &self.children {
            if let Some(node) = child.process(processor) {
                if let Attrs::_Inline = node.attrs {
                    for c in node.children {
                        nodes.push(c);
                    }
                } else {
                    nodes.push(Box::new(node));
                }
            }
        }
        // Call the end of block handler, giving the processor a chance to do any
        // internal clean up or inject some more nodes at the end
        let r = processor.on_end_of_block(&self);
        if let Some(node) = self.process_return_code(r, processor) {
            if let Attrs::_Inline = node.attrs {
                for c in node.children {
                    nodes.push(c);
                }
            } else {
                nodes.push(Box::new(node));
            }
        }
        self.replace_children(nodes)
    }

    /// Returns a new node which is a copy of self with its children replaced
    /// by the given collection of nodes.
    pub fn replace_children(&self, nodes: Vec<Box<Node>>) -> Node {
        let new_node = Node {
            attrs: self.attrs.clone(),
            meta: self.meta.clone(),
            children: nodes,
        };
        new_node
    }

    /// Returns a new node which is a copy of self with its attrs replaced
    /// by the given attrs.
    pub fn replace_attrs(&self, attrs: Attrs) -> Node {
        let new_node = Node {
            attrs: attrs,
            meta: self.meta.clone(),
            children: self.children.clone(),
        };
        new_node
    }
}
