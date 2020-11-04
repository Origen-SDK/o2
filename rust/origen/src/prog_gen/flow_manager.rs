//! Responsible for managing access to the current test program flow and for providing
//! storage for all flows in a test program generation run.
//! This basically manages a RWLock over an AST representing the current flow, allowing
//! application code to always deal with an immutable reference to an instance of this
//! struct at origen::FLOW.

use crate::generator::ast::*;
use crate::Result;
use indexmap::IndexMap;
use std::fmt;
use std::sync::RwLock;

pub struct FlowManager {
    inner: RwLock<Inner>,
}

struct Inner {
    /// Flows are represented as an AST, the last flow is the current one and so an IndexMap
    /// (ordered) instead of a regular HashMap
    flows: IndexMap<String, AST>,
    /// Selects one of the flows such that most FlowManager methods will act on that flow. By
    /// default and if no flow is selected, methods will act on the last flow in flows, which
    /// effectively is "the current flow" during test program generation.
    selected_flow: Option<String>,
}

impl FlowManager {
    pub fn new() -> FlowManager {
        FlowManager {
            inner: RwLock::new(Inner {
                flows: IndexMap::new(),
                selected_flow: None,
            }),
        }
    }

    /// Select the given flow such that the majority of FlowManager methods will act on it.
    /// Returns an error if no flow of the given name exists.
    pub fn select(&self, name: &str) -> Result<()> {
        let mut inner = self.inner.write().unwrap();
        if !inner.flows.contains_key(name) {
            return error!("No flow named '{}' exists", name);
        }
        inner.selected_flow = Some(name.to_string());
        Ok(())
    }

    /// De-selects any named flow selection made via the select() method, returning the majority
    /// of the FlowManager methods to act on the current/latest flow
    pub fn select_current(&self) {
        let mut inner = self.inner.write().unwrap();
        inner.selected_flow = None;
    }

    /// Execute the given function with an immutable reference to an index map containing all flows,
    /// the map will simply be empty if there are no flows.
    /// The result of the given function is returned.
    pub fn with_all_flows<T, F>(&self, mut func: F) -> Result<T>
    where
        F: FnMut(&IndexMap<String, AST>) -> Result<T>,
    {
        let inner = self.inner.read().unwrap();
        func(&inner.flows)
    }

    /// Execute the given function which receives the currently selected flow (or the current flow
    /// if none is selected) as an input, returning the result of the function or an error if no
    /// flow exists yet
    pub fn with_selected_flow<T, F>(&self, func: F) -> Result<T>
    where
        F: FnOnce(&AST) -> Result<T>,
    {
        let inner = self.inner.read().unwrap();
        if let Some(name) = &inner.selected_flow {
            if let Some(flow) = inner.flows.get(name) {
                return func(flow);
            } else {
                return error!("Something has gone wrong, flow '{}' no longer exists", name);
            }
        } else {
            if let Some(flow) = inner.flows.values().last() {
                return func(flow);
            }
        }
        return error!("No flow exists yet");
    }

    /// Like with_selected_flow() but with a mutable reference to the flow AST
    pub fn with_selected_flow_mut<T, F>(&self, func: F) -> Result<T>
    where
        F: FnOnce(&mut AST) -> Result<T>,
    {
        let mut inner = self.inner.write().unwrap();

        // Different approach here vs. with_selected_flow is to pacify the borrow checker
        // without cloning the flow name. It would not have been a big deal to do that, but
        // wanted to try and get it to work without cloning.
        if inner.selected_flow.is_some() {
            // No risk of unwrapping the return value here, the selected_flow is private and can only
            // be changed by the select() method which will verify that the given name matches an existing flow
            let index = inner
                .flows
                .get_index_of(inner.selected_flow.as_ref().unwrap())
                .unwrap();
            if let Some((_, flow)) = inner.flows.get_index_mut(index) {
                return func(flow);
            }
        } else {
            if let Some(flow) = inner.flows.values_mut().last() {
                return func(flow);
            }
        }
        return error!("No flow exists yet");
    }

    /// Starts a new flow, returns an error if a flow with the same name already exists.
    pub fn start(&self, name: &str) -> Result<()> {
        let mut inner = self.inner.write().unwrap();
        if inner.flows.contains_key(name) {
            return error!("A flow called '{}' already exists", name);
        }
        let mut ast = AST::new();
        ast.start(node!(PGMFlow, name.to_string()));
        inner.flows.insert(name.to_string(), ast);
        Ok(())
    }

    /// End the current flow
    pub fn end(&self) -> Result<()> {
        Ok(())
    }

    /// Push a new terminal node into the AST for the current flow
    pub fn push(&self, node: Node) -> Result<()> {
        self.with_selected_flow_mut(|flow| {
            flow.push(node);
            Ok(())
        })
    }

    pub fn append(&self, nodes: &mut Vec<Node>) -> Result<()> {
        self.with_selected_flow_mut(|flow| {
            flow.append(nodes);
            Ok(())
        })
    }

    /// Push a new node into the current flow AST and leave it open, meaning that all new nodes
    /// added to the AST will be inserted as children of this node until it is closed.
    /// A reference ID is returned and the caller should save this and provide it again
    /// when calling close(). If the reference does not match the expected an error will
    /// be raised. This will catch any cases of application code forgetting to close
    /// a node before closing one of its parents.
    pub fn push_and_open(&self, node: Node) -> Result<usize> {
        self.with_selected_flow_mut(|flow| Ok(flow.push_and_open(node)))
    }

    /// Close the currently open node
    pub fn close(&self, ref_id: usize) -> Result<()> {
        self.with_selected_flow_mut(|flow| flow.close(ref_id))
    }

    /// Replace the node n - offset with the given node, use offset = 0 to
    /// replace the last node that was pushed.
    /// Fails if the AST has no children yet or if the offset is otherwise out
    /// of range.
    pub fn replace(&self, node: Node, offset: usize) -> Result<()> {
        self.with_selected_flow_mut(|flow| flow.replace(node, offset))
    }

    /// Returns a copy of node n - offset, an offset of 0 means
    /// the last node pushed.
    /// Fails if the offset is out of range.
    pub fn get(&self, offset: usize) -> Result<Node> {
        self.with_selected_flow(|flow| flow.get(offset))
    }

    /// Returns a copy of node n - offset, where an offset of 0 means
    /// the last node pushed.
    /// Differs from 'get' in that the offset will step into all nodes'
    /// children. For example, in the AST:
    ///     n1
    ///         n2
    ///             n2.1
    ///             n2.2
    ///         n3
    ///  offset | get(offset) | get_with_descendants(offset)
    ///  0      |       n3    |     n3
    ///  1      |       n2    |     n2.2
    ///  2      |       n1    |     n2.1
    ///
    /// Fails if the offset is out of range.
    pub fn get_with_descendants(&self, offset: usize) -> Result<Node> {
        self.with_selected_flow(|flow| flow.get_with_descendants(offset))
    }

    /// Insert the node at position n - offset, using offset = 0 is equivalent
    /// calling push().
    pub fn insert(&self, node: Node, offset: usize) -> Result<()> {
        self.with_selected_flow_mut(|flow| flow.insert(node, offset))
    }

    pub fn to_string(&self) -> String {
        match self.with_selected_flow(|flow| Ok(format!("{}", flow))) {
            Err(_) => "".to_string(),
            Ok(s) => s,
        }
    }

    pub fn process(&self, process_fn: &mut dyn FnMut(&Node) -> Node) -> Node {
        match self.with_selected_flow(|flow| Ok(flow.process(process_fn))) {
            Err(e) => node!(PGMFlow, format!("{}", e)),
            Ok(n) => n,
        }
    }

    /// Returns a copy of the current flow as a Node
    pub fn to_node(&self) -> Node {
        match self.with_selected_flow(|flow| Ok(flow.to_node())) {
            Err(e) => node!(PGMFlow, format!("{}", e)),
            Ok(n) => n,
        }
    }

    /// Serializes the current flow AST for import into Python
    pub fn to_pickle(&self) -> Vec<u8> {
        match self.with_selected_flow(|flow| Ok(flow.to_pickle())) {
            Err(e) => node!(PGMFlow, format!("{}", e)).to_pickle(),
            Ok(n) => n,
        }
    }
}

impl fmt::Display for FlowManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.with_selected_flow(|flow| Ok(flow.to_string())) {
            Err(e) => write!(f, "{}", e),
            Ok(n) => write!(f, "{}", n),
        }
    }
}

impl fmt::Debug for FlowManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.with_selected_flow(|flow| Ok(flow.to_string())) {
            Err(e) => write!(f, "{}", e),
            Ok(n) => write!(f, "{}", n),
        }
    }
}

impl PartialEq<AST> for FlowManager {
    fn eq(&self, ast: &AST) -> bool {
        self.to_node() == ast.to_node()
    }
}

impl PartialEq<Node> for FlowManager {
    fn eq(&self, node: &Node) -> bool {
        self.to_node() == *node
    }
}
