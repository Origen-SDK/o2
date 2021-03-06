use super::processors::ToString;
use crate::generator::ast::*;
use crate::generator::processor::*;
use crate::{Error, Operation, STATUS};
use std::fmt;

#[derive(Clone, PartialEq, Serialize)]
pub struct Node {
    pub attrs: Attrs,
    pub meta: Option<Meta>,
    pub children: Vec<Box<Node>>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Meta {
    pub filename: Option<String>,
    pub lineno: Option<usize>,
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

    pub fn new_with_meta(attrs: Attrs, meta: Option<Meta>) -> Node {
        Node {
            attrs: attrs,
            children: Vec::new(),
            meta: meta,
        }
    }

    pub fn new_with_children(attrs: Attrs, children: Vec<Node>) -> Node {
        Node {
            attrs: attrs,
            children: children.into_iter().map(|n| Box::new(n)).collect(),
            meta: None,
        }
    }

    pub fn unwrap(&mut self) -> Option<Node> {
        match self.children.pop() {
            None => None,
            Some(n) => Some(*n),
        }
    }

    pub fn error(&self, error: Error) -> Result<()> {
        // Messaging may need to be slightly different for patgen
        if STATUS.operation() == Operation::GenerateFlow {
            let help = {
                if self.meta.is_some() && self.meta.as_ref().unwrap().filename.is_some() {
                    let mut m = self
                        .meta
                        .as_ref()
                        .unwrap()
                        .filename
                        .as_ref()
                        .unwrap()
                        .to_string();
                    if let Some(l) = self.meta.as_ref().unwrap().lineno {
                        m += &format!(":{}", l);
                    }
                    m
                } else {
                    if STATUS.is_debug_enabled() {
                        // Don't display children since it's potentially huge
                        let n = self.replace_children(vec![]);
                        format!("Sorry, no flow source information was found, here is the flow node that failed if it helps:\n{}", n)
                    } else {
                        "Run again with the --debug switch to try and trace this back to a flow source file location".to_string()
                    }
                }
            };
            error!("{}\n{}", error, &help)
        } else {
            Err(error)
        }
    }

    /// Returns a new node which is the output of the node processed by the given processor.
    /// Returning None means that the processor has decided that the node should be removed
    /// from the next stage AST.
    pub fn process(&self, processor: &mut dyn Processor) -> Result<Option<Node>> {
        let r = processor.on_node(&self)?;
        self.process_return_code(r, processor)
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

    pub fn add_children(&mut self, nodes: Vec<Node>) -> &Self {
        for n in nodes {
            self.add_child(n);
        }
        self
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

    pub fn depth(&self) -> usize {
        let mut depth = 0;
        for n in self.children.iter() {
            depth += n.depth();
        }
        depth
    }

    pub fn get_descendant(&self, offset: usize, depth: &mut usize) -> Option<Node> {
        for n in self.children.iter().rev() {
            if let Some(node) = n.get_descendant(offset, depth) {
                return Some(node);
            }
        }
        if offset == *depth {
            Some(self.clone())
        } else {
            *depth += 1;
            None
        }
    }

    pub fn process_return_code(
        &self,
        code: Return,
        processor: &mut dyn Processor,
    ) -> Result<Option<Node>> {
        match code {
            Return::None => Ok(None),
            Return::ProcessChildren => Ok(Some(self.process_and_update_children(processor)?)),
            Return::Unmodified => Ok(Some(self.clone())),
            Return::Replace(node) => Ok(Some(node)),
            // We can't return multiple nodes from this function, so we return them
            // wrapped in a meta-node and the process_children method will identify
            // this and remove the wrapper to inline the contained nodes.
            Return::Unwrap => Ok(Some(Node::inline(self.children.clone()))),
            Return::Inline(nodes) => Ok(Some(Node::inline(
                nodes.into_iter().map(|n| Box::new(n)).collect(),
            ))),
            Return::InlineBoxed(nodes) => Ok(Some(Node::inline(nodes))),
            Return::UnwrapWithProcessedChildren => {
                let nodes = self.process_children(processor)?;
                Ok(Some(Node::inline(
                    nodes.into_iter().map(|n| Box::new(n)).collect(),
                )))
            }
            Return::InlineWithProcessedChildren(mut nodes) => {
                nodes.append(&mut self.process_children(processor)?);
                Ok(Some(Node::inline(
                    nodes.into_iter().map(|n| Box::new(n)).collect(),
                )))
            }
            Return::ReplaceChildren(nodes) => {
                let new_node = self.replace_unboxed_children(nodes);
                Ok(Some(new_node))
            }
        }
    }

    /// Returns a new node which is a copy of self with its children replaced
    /// by their processed counterparts.
    pub fn process_and_update_children(&self, processor: &mut dyn Processor) -> Result<Node> {
        if self.children.len() == 0 {
            return Ok(self.clone());
        }
        Ok(self.replace_children(self.process_and_box_children(processor)?))
    }

    /// Returns processed versions of the node's children, each wrapped in a Box
    pub fn process_and_box_children(
        &self,
        processor: &mut dyn Processor,
    ) -> Result<Vec<Box<Node>>> {
        let mut nodes: Vec<Box<Node>> = Vec::new();
        for child in &self.children {
            if let Some(node) = child.process(processor)? {
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
        let r = processor.on_end_of_block(&self)?;
        if let Some(node) = self.process_return_code(r, processor)? {
            if let Attrs::_Inline = node.attrs {
                for c in node.children {
                    nodes.push(c);
                }
            } else {
                nodes.push(Box::new(node));
            }
        }
        Ok(nodes)
    }

    /// Returns processed versions of the node's children
    pub fn process_children(&self, processor: &mut dyn Processor) -> Result<Vec<Node>> {
        let mut nodes: Vec<Node> = Vec::new();
        for child in &self.children {
            if let Some(node) = child.process(processor)? {
                if let Attrs::_Inline = node.attrs {
                    for c in node.children {
                        nodes.push(*c);
                    }
                } else {
                    nodes.push(node);
                }
            }
        }
        // Call the end of block handler, giving the processor a chance to do any
        // internal clean up or inject some more nodes at the end
        let r = processor.on_end_of_block(&self)?;
        if let Some(node) = self.process_return_code(r, processor)? {
            if let Attrs::_Inline = node.attrs {
                for c in node.children {
                    nodes.push(*c);
                }
            } else {
                nodes.push(node);
            }
        }
        Ok(nodes)
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

    /// Returns a new node which is a copy of self with its children replaced
    /// by the given collection of nodes.
    pub fn replace_unboxed_children(&self, nodes: Vec<Node>) -> Node {
        let new_node = Node {
            attrs: self.attrs.clone(),
            meta: self.meta.clone(),
            children: nodes.into_iter().map(|n| Box::new(n)).collect(),
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
