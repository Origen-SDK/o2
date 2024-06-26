use chrono::prelude::*;

use super::super::nodes::PAT;
use crate::Result;
use crate::{app, STATUS};
use origen_metal::ast::{Node, Processor, Return};

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
    pub fn run(node: &Node<PAT>) -> Result<Node<PAT>> {
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
    fn to_text(&self, content: &str) -> Node<PAT> {
        let spacing_length = self.indentation_length * self.section_depth;
        node!(
            PAT::Text,
            format!("{}{}", " ".to_string().repeat(spacing_length), content)
        )
    }

    /// Casts the current line as a 'Text Node' and resets the current line
    fn current_line_to_text(&self) -> Node<PAT> {
        self.to_text(&self.current_line)
    }

    /// Inserts a section boundary node
    fn section_boundary(&self) -> Node<PAT> {
        let spacing_length = self.indentation_length * self.section_depth;
        let boundary_repeat = (self.boundary_length - spacing_length) / self.boundary_string.len();
        node!(
            PAT::Text,
            format!(
                "{}{}",
                " ".to_string().repeat(spacing_length),
                self.boundary_string.repeat(boundary_repeat)
            )
        )
    }
}

impl Processor<PAT> for FlattenText {
    fn on_node(&mut self, node: &Node<PAT>) -> origen_metal::Result<Return<PAT>> {
        match &node.attrs {
            PAT::TextSection(header, lvl) => {
                // When adding a new section, if we aren't nested then we'll print a 'boundary', which will be
                // something like "<comment char>******..."
                // We'll also print the header immediately below, if one is given
                // This nodes children will be indented
                // If we're already in a nested section, then do the same but without the section boundary
                //   Nested sections are more like 'subsections' than a bonafide section
                let mut nodes: Vec<Node<PAT>> = vec![];
                if lvl.is_some() && lvl.unwrap() == 0 {
                    nodes.push(self.section_boundary());
                }
                if let Some(h) = header {
                    nodes.push(self.to_text(h));
                }
                self.section_depth += 1;
                Ok(Return::InlineWithProcessedChildren(nodes))
            }
            PAT::Text(content) => {
                if self.in_text_line {
                    // Processing a single line: append this content to the current content and eat the node
                    self.current_line += content;
                    Ok(Return::None)
                } else {
                    // Not inside a text line, so just print the indendation followed by the content
                    Ok(Return::Replace(self.to_text(content)))
                }
            }
            PAT::TextLine => {
                // Indicate that we're in a text line and process its children
                // NOTE: this assumes that the line has already been cleared, either from the initial state
                //  or from the on_end_of_block
                // If extra content is present in this node, its a bug elsewhere in the processor
                self.in_text_line = true;
                Ok(Return::UnwrapWithProcessedChildren)
            }
            PAT::TextBoundaryLine => Ok(Return::Inline(vec![self.section_boundary()])),
            PAT::User => {
                if let Err(e) = origen_metal::with_current_user(|u| {
                    self.current_line += &u.username()?;
                    Ok(())
                }) {
                    // Don't kill the pattern/program generation because the user ID isn't retrievable.
                    // Just be annoying about it.
                    log_error!("Unable to retrieve current user ID");
                    log_error!("Failed with error: \"{}\"", e.msg);
                    self.current_line += "Error - Could not retrieve current user ID";
                };
                Ok(Return::None)
            }
            PAT::Timestamp => {
                self.current_line += &Local::now().to_string();
                Ok(Return::None)
            }
            PAT::OrigenCommand(val) => {
                self.current_line += val;
                Ok(Return::None)
            }
            PAT::OS => {
                self.current_line += &whoami::os();
                Ok(Return::None)
            }
            PAT::Mode => {
                app().unwrap().with_config(|config| {
                    self.current_line += &config.mode;
                    Ok(())
                })?;
                Ok(Return::None)
            }
            PAT::TargetsStacked => {
                let mut nodes: Vec<Node<PAT>> = vec![];
                self.section_depth += 1;
                let _ = app().unwrap().with_config(|config| {
                    if let Some(targets) = &config.target {
                        for t in targets {
                            nodes.push(self.to_text(&t));
                        }
                    } else {
                        nodes.push(self.to_text("No targets have been set!"));
                    }
                    Ok(())
                });
                self.section_depth -= 1;
                Ok(Return::Inline(nodes))
            }
            PAT::AppRoot => {
                self.current_line += &app().unwrap().root.display().to_string();
                Ok(Return::None)
            }
            PAT::OrigenVersion => {
                self.current_line += &STATUS.origen_version.to_string();
                Ok(Return::None)
            }
            PAT::OrigenRoot => {
                self.current_line += &std::env::current_exe().unwrap().display().to_string();
                Ok(Return::None)
            }
            _ => Ok(Return::ProcessChildren),
        }
    }

    fn on_processed_node(&mut self, node: &Node<PAT>) -> origen_metal::Result<Return<PAT>> {
        match node.attrs {
            PAT::TextLine => {
                let n = self.current_line_to_text();
                self.in_text_line = false;
                self.current_line.clear();
                Ok(Return::Inline(vec![n]))
            }
            PAT::TextSection(_, lvl) => {
                self.section_depth -= 1;
                if lvl.is_some() && lvl.unwrap() == 0 {
                    Ok(Return::Inline(vec![self.section_boundary()]))
                } else {
                    Ok(Return::Unmodified)
                }
            }
            _ => Ok(Return::Unmodified) 
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::nodes::PAT;

    #[test]
    fn it_works() {
        let mut header = node!(PAT::PatternHeader);
        header.add_child(node!(PAT::TextBoundaryLine));
        let mut section = node!(PAT::TextSection, Some("Generated".to_string()), None);
        let mut text_lines = vec![];
        let mut text_line = node!(PAT::TextLine);
        text_line.add_child(node!(PAT::Text, "By: user".to_string()));
        text_lines.push(text_line);
        let mut text_line = node!(PAT::TextLine);
        text_line.add_child(node!(PAT::Text, "Command: origen generate pattern".to_string()));
        text_lines.push(text_line);
        section.add_children(text_lines);
        header.add_child(section);
        header.add_child(node!(PAT::TextBoundaryLine));
        let mut section = node!(PAT::TextSection, Some("Workspace".to_string()), None);
        let mut sub_section = node!(PAT::TextSection, Some("Environment".to_string()), None);
        let mut text_lines = vec![];
        let mut text_line = node!(PAT::TextLine);
        text_line.add_child(node!(PAT::Text, "Mode: development".to_string()));
        text_lines.push(text_line);
        sub_section.add_children(text_lines);
        section.add_child(sub_section);
        let mut sub_section = node!(PAT::TextSection, Some("Origen Core".to_string()), None);
        let mut text_lines = vec![];
        let mut text_line = node!(PAT::TextLine);
        text_line.add_child(node!(PAT::Text, "Version: 2.something".to_string()));
        text_lines.push(text_line);
        let mut text_line = node!(PAT::TextLine);
        text_line.add_child(node!(PAT::Text, "Executable Path: /path/to/python/python.exe".to_string()));
        text_lines.push(text_line);
        sub_section.add_children(text_lines);
        section.add_child(sub_section);
        header.add_child(section);
        header.add_child(node!(PAT::TextBoundaryLine));
 
        let header_flat = FlattenText::run(&header).expect("Text flattened");

        let mut expect = node!(PAT::PatternHeader);
        expect.add_children(vec![
            node!(PAT::Text, "******************************************************************************************".to_string()),
            node!(PAT::Text, "Generated".to_string()),
            node!(PAT::Text, "  By: user".to_string()),
            node!(PAT::Text, "  Command: origen generate pattern".to_string()),
            node!(PAT::Text, "******************************************************************************************".to_string()),
            node!(PAT::Text, "Workspace".to_string()),
            node!(PAT::Text, "  Environment".to_string()),
            node!(PAT::Text, "    Mode: development".to_string()),
            node!(PAT::Text, "  Origen Core".to_string()),
            node!(PAT::Text, "    Version: 2.something".to_string()),
            node!(PAT::Text, "    Executable Path: /path/to/python/python.exe".to_string()),
            node!(PAT::Text, "******************************************************************************************".to_string()),
        ]);

        assert_eq!(header_flat, expect);
    }
}