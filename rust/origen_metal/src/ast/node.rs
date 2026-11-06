use super::ast::AST;
use super::processors::ToString;
//use crate::{Error, Operation, STATUS};
use crate::ast::processor::{Processor, Return};
use crate::Result;
use std::fmt::{self, Debug, Display};
use std::io::Write;

pub trait Attrs: Clone + std::cmp::PartialEq + serde::Serialize + Display + Debug {}
impl<T: Clone + std::cmp::PartialEq + serde::Serialize + Display + Debug> Attrs for T {}

#[derive(Clone, PartialEq, Serialize, Debug)]
pub struct Node<T> {
    pub attrs: T,
    info: Info,
    pub meta: Option<Meta>,
    pub children: Vec<Box<Node<T>>>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Meta {
    pub filename: Option<String>,
    pub lineno: Option<usize>,
}

impl<T: Attrs> fmt::Display for Node<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string_val = self.to_string();
        write!(f, "{}", string_val)
    }
}

impl<T: Attrs> PartialEq<AST<T>> for Node<T> {
    fn eq(&self, ast: &AST<T>) -> bool {
        *self == ast.to_node()
    }
}

enum PostProcessAction {
    None,
    Unwrap,
}

enum Handler {
    OnNode,
    OnEndOfBlock,
    OnProcessedNode,
}

#[derive(Clone, Debug, PartialEq, Serialize, Copy)]
enum Info {
    None,
    Inline,
}

impl<T: Attrs> Node<T> {
    pub fn new(attrs: T) -> Node<T> {
        Node {
            attrs: attrs,
            info: Info::None,
            children: Vec::new(),
            meta: None,
        }
    }

    pub fn new_with_meta(attrs: T, meta: Option<Meta>) -> Node<T> {
        Node {
            attrs: attrs,
            info: Info::None,
            children: Vec::new(),
            meta: meta,
        }
    }

    pub fn new_with_children(attrs: T, children: Vec<Node<T>>) -> Node<T> {
        Node {
            attrs: attrs,
            info: Info::None,
            children: children.into_iter().map(|n| Box::new(n)).collect(),
            meta: None,
        }
    }

    pub fn unwrap(&mut self) -> Option<Node<T>> {
        match self.children.pop() {
            None => None,
            Some(n) => Some(*n),
        }
    }

    /// Returns `"<filename>:<lineno>"` if present, else `""`.
    pub fn meta_string(&self) -> String {
        if let Some(meta) = &self.meta {
            if let Some(f) = &meta.filename {
                let mut s = format!("{}", f);
                if let Some(l) = &meta.lineno {
                    s += &format!(":{}", l);
                }
                return s;
            }
        }
        "".to_string()
    }

    /// Returns a new node which is the output of the node processed by the given processor.
    /// Returning None means that the processor has decided that the node should be removed
    /// from the next stage AST.
    pub fn process(&self, processor: &mut dyn Processor<T>) -> Result<Option<Node<T>>> {
        self.process_(processor, Handler::OnNode, None)
    }

    /// Visits the node and its descendants with the processor's on_node handler, as process()
    /// does but without building a new AST, for a processor which only reads the AST, e.g. a
    /// validator. A node's children are visited, in order, when on_node returns
    /// ProcessChildren, UnwrapWithProcessedChildren or InlineWithProcessedChildren, whatever it
    /// returns is otherwise ignored. on_end_of_block and on_processed_node are not called.
    pub fn walk(&self, processor: &mut dyn Processor<T>) -> Result<()> {
        match processor.on_node(self)? {
            Return::ProcessChildren
            | Return::UnwrapWithProcessedChildren
            | Return::InlineWithProcessedChildren(_) => self.walk_children(processor),
            _ => Ok(()),
        }
    }

    /// Visits the node's children with walk()
    pub fn walk_children(&self, processor: &mut dyn Processor<T>) -> Result<()> {
        for child in &self.children {
            child.walk(processor)?;
        }
        Ok(())
    }

    /// Processes the given handler's return code for this node. When `first` is given, it is used as
    /// the handler's return for this node instead of calling the handler again.
    fn process_(
        &self,
        processor: &mut dyn Processor<T>,
        handler: Handler,
        mut first: Option<Return<T>>,
    ) -> Result<Option<Node<T>>> {
        let mut node = self;
        let mut open_nodes: Vec<(Node<T>, PostProcessAction)> = vec![];
        let mut to_be_processed: Vec<Vec<&Node<T>>> = vec![];

        loop {
            let r = match first.take() {
                Some(r) => r,
                None => match handler {
                    Handler::OnNode => processor.on_node(node)?,
                    Handler::OnEndOfBlock => processor.on_end_of_block(node)?,
                    Handler::OnProcessedNode => processor.on_processed_node(node)?,
                },
            };

            match r {
                // Terminal return codes
                Return::None => {
                    if matches!(handler, Handler::OnProcessedNode) {
                        return Ok(None);
                    }
                }
                Return::Unmodified => {
                    if matches!(handler, Handler::OnProcessedNode) {
                        return Ok(Some(node.clone()));
                    }
                    open_nodes.push((node.clone(), PostProcessAction::None));
                    to_be_processed.push(vec![]);
                }
                Return::Replace(node) => {
                    open_nodes.push((node, PostProcessAction::None));
                    to_be_processed.push(vec![]);
                }
                // We can't return multiple nodes from this function, so we return them
                // wrapped in a meta-node and the process_children method will identify
                // this and remove the wrapper to inline the contained nodes.
                Return::Unwrap => {
                    open_nodes.push((
                        Node::info(node.children.clone(), node.attrs.clone(), Info::Inline),
                        PostProcessAction::None,
                    ));
                    to_be_processed.push(vec![]);
                }
                Return::Inline(nodes) => {
                    open_nodes.push((
                        Node::info(
                            nodes.into_iter().map(|n| Box::new(n)).collect(),
                            node.attrs.clone(),
                            Info::Inline,
                        ),
                        PostProcessAction::None,
                    ));
                    to_be_processed.push(vec![]);
                }
                Return::InlineBoxed(nodes) => {
                    open_nodes.push((
                        Node::info(nodes, node.attrs.clone(), Info::Inline),
                        PostProcessAction::None,
                    ));
                    to_be_processed.push(vec![]);
                }
                Return::ReplaceChildren(nodes) => {
                    open_nodes.push((
                        node.replace_unboxed_children(nodes),
                        PostProcessAction::None,
                    ));
                    to_be_processed.push(vec![]);
                }

                // Child processing return codes
                Return::ProcessChildren => {
                    open_nodes.push((node.without_children(), PostProcessAction::None));
                    let mut children: Vec<&Node<T>> = vec![];
                    for child in &node.children {
                        children.push(child);
                    }
                    children.reverse();
                    to_be_processed.push(children);
                }

                Return::UnwrapWithProcessedChildren => {
                    open_nodes.push((node.without_children(), PostProcessAction::Unwrap));
                    let mut children: Vec<&Node<T>> = vec![];
                    for child in &node.children {
                        children.push(child);
                    }
                    children.reverse();
                    to_be_processed.push(children);
                }

                Return::InlineWithProcessedChildren(nodes) => {
                    open_nodes.push((
                        Node::info(
                            nodes.into_iter().map(|n| Box::new(n)).collect(),
                            node.attrs.clone(),
                            Info::Inline,
                        ),
                        PostProcessAction::None,
                    ));
                    let mut children: Vec<&Node<T>> = vec![];
                    for child in &node.children {
                        children.push(child);
                    }
                    children.reverse();
                    to_be_processed.push(children);
                }
            }

            loop {
                if !to_be_processed.is_empty() {
                    let last_group = to_be_processed.last_mut().unwrap();
                    if !last_group.is_empty() {
                        node = last_group.pop().unwrap();
                        break;
                    }
                    to_be_processed.pop();
                    // Just completed all the children of the last open node
                    let (mut node, action) = open_nodes.pop().unwrap();
                    match action {
                        PostProcessAction::None => {}
                        PostProcessAction::Unwrap => {
                            node = Node::info(node.children, node.attrs, Info::Inline);
                        }
                    }

                    if matches!(handler, Handler::OnNode) {
                        // Call the end of block handler, giving the processor a chance to do any
                        // internal clean up or inject some more nodes at the end
                        //
                        // This is being kept around for legacy compatibility, the on_processed_node handler
                        // is the preferred way to do this now.
                        if let Some(n) = self.process_(processor, Handler::OnEndOfBlock, None)? {
                            if matches!(n.info, Info::Inline) {
                                for c in n.children {
                                    node.add_child(*c);
                                }
                            } else {
                                node.add_child(n);
                            }
                        }

                        // Call the on_processed_node handler, giving the processor a chance to work on this
                        // node now that its children have been processed. An unmodified node is kept as
                        // it is, rather than cloned with all of its (already processed) descendants.
                        match processor.on_processed_node(&node)? {
                            Return::Unmodified => {}
                            r => {
                                match node.process_(processor, Handler::OnProcessedNode, Some(r))? {
                                    Some(n) => node = n,
                                    None => continue,
                                }
                            }
                        }
                    }

                    if open_nodes.is_empty() {
                        if matches!(node.info, Info::Inline) && node.children.len() == 1 {
                            return Ok(Some(*node.children.pop().unwrap()));
                        } else {
                            return Ok(Some(node));
                        }
                    } else {
                        let (parent, _) = open_nodes.last_mut().unwrap();
                        if matches!(node.info, Info::Inline) {
                            for c in node.children {
                                parent.add_child(*c);
                            }
                        } else {
                            parent.add_child(node);
                        }
                    }
                } else {
                    if open_nodes.is_empty() {
                        return Ok(None);
                    } else if open_nodes.len() == 1 {
                        return Ok(Some(open_nodes.pop().unwrap().0));
                    } else {
                        bail!(
                            "Internal error: open_nodes should be empty or have a single node left"
                        )
                    }
                }
            }
        }
    }

    /// Creates a new node which is used to pass information back to a caller, this is a hack to pass
    /// flow control information back when the return type is a Result<Option<Node<T>>>
    fn info(nodes: Vec<Box<Node<T>>>, example_attrs: T, info: Info) -> Node<T> {
        Node {
            // This example_attrs argument requirement is ugly, but required without significant
            // upstream changes. The attrs will be ignored downstream whenever inline = true and this
            // is purely to support this working with any type of Node.
            // Also this is an internal function and so we can live with it.
            attrs: example_attrs,
            info: info,
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

    /// Writes the AST to the given file to allow it to be reviewed for debugging purposes
    pub fn to_file<P: AsRef<std::path::Path>>(&self, path: P) -> Result<()> {
        let mut f = std::fs::File::create(path)?;
        writeln!(&mut f, "{:#?}", self)?;
        Ok(())
    }

    pub fn add_child(&mut self, node: Node<T>) {
        self.children.push(Box::new(node));
    }

    pub fn add_children(&mut self, nodes: Vec<Node<T>>) -> &Self {
        for n in nodes {
            self.add_child(n);
        }
        self
    }

    pub fn insert_child(&mut self, node: Node<T>, offset: usize) -> Result<()> {
        let len = self.children.len();
        if offset > len {
            bail!(
                "An offset of {} was given to insert a child into a node with only {} children",
                offset,
                len
            );
        }
        let index = self.children.len() - offset;
        self.children.insert(index, Box::new(node));
        Ok(())
    }

    /// Replace the child n - offset with the given node, use offset = 0 to
    /// replace the last child that was pushed.
    /// Fails if the node has no children or if the given offset is
    /// otherwise out of range.
    pub fn replace_child(&mut self, node: Node<T>, offset: usize) -> Result<()> {
        let len = self.children.len();
        if len == 0 {
            bail!("Attempted to replace a child in a node with no children");
        } else if offset > len - 1 {
            bail!(
                "An offset of {} was given to replace a child in a node with only {} children",
                offset,
                len
            );
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
    pub fn get_child(&self, offset: usize) -> Result<Node<T>> {
        let len = self.children.len();
        if len == 0 {
            bail!("Attempted to get a child in a node with no children");
        } else if offset > len - 1 {
            bail!(
                "An offset of {} was given to get a child in a node with only {} children",
                offset,
                len
            );
        }
        let index = self.children.len() - 1 - offset;
        Ok(*self.children[index].clone())
    }

    /// Removes the child node at the given offset and returns it
    pub fn remove_child(&mut self, offset: usize) -> Result<Node<T>> {
        let len = self.children.len();
        if len == 0 {
            bail!("Attempted to remove a child in a node with no children");
        } else if offset > len - 1 {
            bail!(
                "An offset of {} was given to remove a child in a node with only {} children",
                offset,
                len
            );
        }
        Ok(*self.children.remove(offset))
    }

    pub fn depth(&self) -> usize {
        let mut depth = 0;
        for n in self.children.iter() {
            depth += n.depth();
        }
        depth
    }

    pub fn get_descendant(&self, offset: usize, depth: &mut usize) -> Option<Node<T>> {
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

    /// Returns a new node which is a copy of self with its children replaced
    /// by their processed counterparts.
    pub fn process_and_update_children(&self, processor: &mut dyn Processor<T>) -> Result<Node<T>> {
        if self.children.len() == 0 {
            return Ok(self.clone());
        }
        Ok(self.replace_children(self.process_and_box_children(processor)?))
    }

    /// Returns processed versions of the node's children, each wrapped in a Box
    pub fn process_and_box_children(
        &self,
        processor: &mut dyn Processor<T>,
    ) -> Result<Vec<Box<Node<T>>>> {
        let mut nodes: Vec<Box<Node<T>>> = Vec::new();
        for child in &self.children {
            if let Some(node) = child.process(processor)? {
                if matches!(node.info, Info::Inline) {
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
        if let Some(node) = self.process_(processor, Handler::OnEndOfBlock, None)? {
            if matches!(node.info, Info::Inline) {
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
    pub fn process_children(&self, processor: &mut dyn Processor<T>) -> Result<Vec<Node<T>>> {
        let mut nodes: Vec<Node<T>> = Vec::new();
        for child in &self.children {
            if let Some(node) = child.process(processor)? {
                if matches!(node.info, Info::Inline) {
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
        if let Some(node) = self.process_(processor, Handler::OnEndOfBlock, None)? {
            if matches!(node.info, Info::Inline) {
                for c in node.children {
                    nodes.push(*c);
                }
            } else {
                nodes.push(node);
            }
        }
        Ok(nodes)
    }

    /// Returns a new node which is a copy of self with its children removed
    pub fn without_children(&self) -> Node<T> {
        self.replace_children(vec![])
    }

    /// Returns a new node which is a copy of self with its children replaced
    /// by the given collection of nodes.
    pub fn replace_children(&self, nodes: Vec<Box<Node<T>>>) -> Node<T> {
        let new_node = Node {
            attrs: self.attrs.clone(),
            info: self.info,
            meta: self.meta.clone(),
            children: nodes,
        };
        new_node
    }

    /// Returns a new node which is a copy of self with its children replaced
    /// by the given collection of nodes.
    pub fn replace_unboxed_children(&self, nodes: Vec<Node<T>>) -> Node<T> {
        let new_node = Node {
            attrs: self.attrs.clone(),
            info: self.info,
            meta: self.meta.clone(),
            children: nodes.into_iter().map(|n| Box::new(n)).collect(),
        };
        new_node
    }

    /// Returns a new node which is a copy of self with its attrs replaced
    /// by the given attrs.
    pub fn replace_attrs(&self, attrs: T) -> Node<T> {
        let new_node = Node {
            attrs: attrs,
            info: self.info,
            meta: self.meta.clone(),
            children: self.children.clone(),
        };
        new_node
    }

    /// Ensures the the given node type is present in the nodes immediate children,
    /// inserting it if not
    pub fn ensure_node_present(&mut self, attrs: T) {
        if self.children.iter().any(|c| c.attrs == attrs) {
            return;
        }
        self.children.push(Box::new(Node::new(attrs)));
    }

    /// Returns a new node which is a copy of self with its components replaced by the given values
    pub fn updated(
        &self,
        attrs: Option<T>,
        children: Option<Vec<Box<Node<T>>>>,
        meta: Option<Meta>,
    ) -> Node<T> {
        Node {
            attrs: match attrs {
                Some(x) => x,
                None => self.attrs.clone(),
            },
            info: self.info,
            children: match children {
                Some(x) => x,
                None => self.children.clone(),
            },
            meta: match meta {
                Some(x) => Some(x),
                None => self.meta.clone(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::{Node, Processor, Return};
    use crate::Result;

    #[derive(Clone, PartialEq, Serialize, Debug)]
    enum T {
        Group(String),
        Leaf(i64),
    }

    impl std::fmt::Display for T {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self)
        }
    }

    fn tree() -> Node<T> {
        node!(T::Group, "root".to_string() =>
            node!(T::Leaf, 1),
            node!(T::Group, "a".to_string() =>
                node!(T::Leaf, 2),
                node!(T::Group, "b".to_string() =>
                    node!(T::Leaf, 3),
                ),
            ),
            node!(T::Group, "skip".to_string() =>
                node!(T::Leaf, 4),
            ),
            node!(T::Group, "drop".to_string() =>
                node!(T::Leaf, 5),
            ),
        )
    }

    /// Records every handler call, doesn't descend into "skip" groups, drops "drop" groups once
    /// processed and renames "b" groups once processed
    #[derive(Default)]
    struct Recorder {
        on_node: Vec<String>,
        on_processed_node: Vec<String>,
    }

    impl Processor<T> for Recorder {
        fn on_node(&mut self, node: &Node<T>) -> Result<Return<T>> {
            self.on_node.push(node.attrs.to_string());
            Ok(match &node.attrs {
                T::Group(name) if name == "skip" => Return::Unmodified,
                _ => Return::ProcessChildren,
            })
        }

        fn on_processed_node(&mut self, node: &Node<T>) -> Result<Return<T>> {
            self.on_processed_node.push(node.attrs.to_string());
            Ok(match &node.attrs {
                T::Group(name) if name == "drop" => Return::None,
                T::Group(name) if name == "b" => {
                    Return::Replace(node.replace_attrs(T::Group("c".to_string())))
                }
                _ => Return::Unmodified,
            })
        }
    }

    #[test]
    fn processed_nodes_are_handled_once_and_kept_replaced_or_removed() -> Result<()> {
        let mut p = Recorder::default();
        let output = tree().process(&mut p)?.unwrap();
        let expected = node!(T::Group, "root".to_string() =>
            node!(T::Leaf, 1),
            node!(T::Group, "a".to_string() =>
                node!(T::Leaf, 2),
                node!(T::Group, "c".to_string() =>
                    node!(T::Leaf, 3),
                ),
            ),
            node!(T::Group, "skip".to_string() =>
                node!(T::Leaf, 4),
            ),
        );
        assert_eq!(output, expected);
        assert_eq!(
            p.on_processed_node,
            vec![
                "Leaf(1)",
                "Leaf(2)",
                "Leaf(3)",
                "Group(\"b\")",
                "Group(\"a\")",
                "Group(\"skip\")",
                "Leaf(5)",
                "Group(\"drop\")",
                "Group(\"root\")",
            ]
        );
        Ok(())
    }

    #[test]
    fn walk_visits_the_nodes_that_process_visits() -> Result<()> {
        let mut processed = Recorder::default();
        tree().process(&mut processed)?;
        let mut walked = Recorder::default();
        tree().walk(&mut walked)?;
        assert_eq!(walked.on_node, processed.on_node);
        assert!(!walked.on_node.contains(&"Leaf(4)".to_string()));
        assert!(walked.on_processed_node.is_empty());
        Ok(())
    }
}
