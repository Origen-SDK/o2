use chrono::prelude::*;

use crate::generator::ast::*;
use crate::generator::processor::*;
use crate::{app_config, producer, STATUS};

/// Flattens nested text, textlines, text sections, etc. into 'text' types.
/// Also evaluates text placeholder or shorthand nodes, such User, Timestamp, etc.
pub struct FlattenText {
    current_line: String,
    section_depth: usize,
    in_text_line: bool,

    // The following can be freely changed by the caller
    pub boundary_string: String,
    pub boundary_length: usize,
    pub indentation_length: usize,
}

impl FlattenText {
    pub fn run(node: &Node) -> Result<Node> {
        let mut p = FlattenText {
            current_line: "".to_string(),
            section_depth: 0,
            in_text_line: false,
            boundary_string: "*".to_string(),
            boundary_length: 90,
            indentation_length: 2,
        };
        Ok(node.process(&mut p)?.unwrap())
    }

    // Some helper methods

    /// Casts the content as a 'Text Node'
    fn to_text(&self, content: &str) -> Node {
        let spacing_length = self.indentation_length * self.section_depth;
        node!(
            Text,
            format!("{}{}", " ".to_string().repeat(spacing_length), content)
        )
    }

    /// Casts the current line as a 'Text Node' and resets the current line
    fn current_line_to_text(&self) -> Node {
        self.to_text(&self.current_line)
    }

    /// Inserts a section boundary node
    fn section_boundary(&self) -> Node {
        let spacing_length = self.indentation_length * self.section_depth;
        let boundary_repeat = (self.boundary_length - spacing_length) / self.boundary_string.len();
        node!(
            Text,
            format!(
                "{}{}",
                " ".to_string().repeat(spacing_length),
                self.boundary_string.repeat(boundary_repeat)
            )
        )
    }
}

impl Processor for FlattenText {
    fn on_node(&mut self, node: &Node) -> Result<Return> {
        match &node.attrs {
            Attrs::TextSection(header, _lvl) => {
                // When adding a new section, if we aren't nested then we'll print a 'boundary', which will be
                // something like "<comment char>******..."
                // We'll also print the header immediately below, if one is given
                // This nodes children will be indented
                // If we're already in a nested section, then do the same but without the section boundary
                //   Nested sections are more like 'subsections' than a bonafide section
                let mut nodes: Vec<Node> = vec![];
                if self.section_depth == 0 {
                    nodes.push(self.section_boundary());
                }
                if let Some(h) = header {
                    nodes.push(self.to_text(h));
                }
                self.section_depth += 1;
                Ok(Return::InlineWithProcessedChildren(nodes))
            }
            Attrs::Text(content) => {
                if self.in_text_line {
                    // Processing a single line: append this content to the current content and eat the node
                    self.current_line += content;
                    Ok(Return::None)
                } else {
                    // Not inside a text line, so just print the indendation followed by the content
                    Ok(Return::Replace(self.to_text(content)))
                }
            }
            Attrs::TextLine => {
                // Indicate that we're in a text line and process its children
                // NOTE: this assumes that the line has already been cleared, either from the initial state
                //  or from the on_end_of_block
                // If extra content is present in this node, its a bug elsewhere in the processor
                self.in_text_line = true;
                Ok(Return::UnwrapWithProcessedChildren)
            }
            Attrs::User => {
                self.current_line += &whoami::username();
                Ok(Return::None)
            }
            Attrs::Timestamp => {
                self.current_line += &Local::now().to_string();
                Ok(Return::None)
            }
            Attrs::CurrentCommand => {
                self.current_line += &producer().jobs.last().unwrap().command;
                Ok(Return::None)
            }
            Attrs::OS => {
                self.current_line += &whoami::os();
                Ok(Return::None)
            }
            Attrs::Mode => {
                self.current_line += &app_config().mode;
                Ok(Return::None)
            }
            Attrs::TargetsStacked => {
                let mut nodes: Vec<Node> = vec![];
                self.section_depth += 1;
                if let Some(targets) = &app_config().target {
                    for t in targets {
                        nodes.push(self.to_text(&t));
                    }
                } else {
                    nodes.push(self.to_text("No targets have been set!"));
                }
                self.section_depth -= 1;
                Ok(Return::Inline(nodes))
            }
            Attrs::AppRoot => {
                self.current_line += &STATUS.root.display().to_string();
                Ok(Return::None)
            }
            Attrs::OrigenVersion => {
                self.current_line += &STATUS.origen_version.to_string();
                Ok(Return::None)
            }
            Attrs::OrigenRoot => {
                self.current_line += &std::env::current_exe().unwrap().display().to_string();
                Ok(Return::None)
            }
            _ => Ok(Return::ProcessChildren),
        }
    }

    fn on_end_of_block(&mut self, node: &Node) -> Result<Return> {
        match node.attrs {
            Attrs::TextLine => {
                let n = self.current_line_to_text();
                self.in_text_line = false;
                self.current_line.clear();
                Ok(Return::Inline(vec![n]))
            }
            Attrs::TextSection(_, _) => {
                self.section_depth -= 1;
                Ok(Return::None)
            }
            _ => Ok(Return::None),
        }
    }
}
