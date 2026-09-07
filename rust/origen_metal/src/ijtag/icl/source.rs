use crate::{Error, Result};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub(crate) struct ExpandedSource {
    pub(crate) text: String,
    pub(crate) map: SourceMap,
    pub(crate) entry: String,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct SourceMap {
    chunks: Vec<OriginChunk>,
}

#[derive(Clone, Debug)]
struct OriginChunk {
    start: u32,
    end: u32,
    file: String,
    line: usize,
    column: usize,
}

impl SourceMap {
    pub(crate) fn single(source: &str, file: Option<&str>) -> Self {
        let file = file.unwrap_or("<string>").to_string();
        let mut map = Self::default();
        let mut offset = 0usize;
        for (index, line) in source.split_inclusive('\n').enumerate() {
            map.push(offset, offset + line.len(), file.clone(), index + 1, 1);
            offset += line.len();
        }
        if source.is_empty() {
            map.push(0, 0, file, 1, 1);
        }
        map
    }

    fn push(&mut self, start: usize, end: usize, file: String, line: usize, column: usize) {
        self.chunks.push(OriginChunk {
            start: start as u32,
            end: end as u32,
            file,
            line,
            column,
        });
    }

    pub(crate) fn location(&self, offset: u32, expanded: &str) -> String {
        let index = self.chunks.partition_point(|chunk| chunk.end <= offset);
        let chunk = self.chunks.get(index).or_else(|| self.chunks.last());
        let Some(chunk) = chunk else {
            return "<unknown>:1:1".to_string();
        };
        let within = offset.saturating_sub(chunk.start) as usize;
        let end = (chunk.start as usize + within).min(expanded.len());
        let start = (chunk.start as usize).min(end);
        let prefix = &expanded[start..end];
        let added_lines = prefix.bytes().filter(|byte| *byte == b'\n').count();
        let column = if let Some(last_newline) = prefix.rfind('\n') {
            prefix[last_newline + 1..].chars().count() + 1
        } else {
            chunk.column + prefix.chars().count()
        };
        format!("{}:{}:{}", chunk.file, chunk.line + added_lines, column)
    }
}

pub(crate) fn from_str(source: &str, file: Option<&str>) -> Result<ExpandedSource> {
    reject_preprocessor_directives(source, file.unwrap_or("<string>"))?;
    Ok(ExpandedSource {
        text: source.to_string(),
        map: SourceMap::single(source, file),
        entry: file.unwrap_or("<string>").to_string(),
    })
}

pub(crate) fn from_file(path: &Path) -> Result<ExpandedSource> {
    if !path.exists() {
        return Err(Error::new(&format!(
            "File does not exist: {}",
            path.display()
        )));
    }
    let canonical = path.canonicalize()?;
    let mut output = String::new();
    let mut map = SourceMap::default();
    let mut stack = Vec::new();
    expand_file(&canonical, &mut stack, &mut output, &mut map)?;
    Ok(ExpandedSource {
        text: output,
        map,
        entry: canonical.display().to_string(),
    })
}

fn expand_file(
    path: &Path,
    stack: &mut Vec<PathBuf>,
    output: &mut String,
    map: &mut SourceMap,
) -> Result<()> {
    let canonical = path.canonicalize().map_err(|error| {
        Error::new(&format!(
            "Unable to read included ICL file {}: {error}",
            path.display()
        ))
    })?;
    if let Some(position) = stack.iter().position(|active| active == &canonical) {
        let mut chain: Vec<_> = stack[position..]
            .iter()
            .map(|path| path.display().to_string())
            .collect();
        chain.push(canonical.display().to_string());
        return Err(Error::new(&format!(
            "ICL include cycle detected: {}",
            chain.join(" -> ")
        )));
    }
    let source = fs::read_to_string(&canonical).map_err(|error| {
        Error::new(&format!(
            "Unable to read ICL file {}: {error}",
            canonical.display()
        ))
    })?;
    stack.push(canonical.clone());
    let file = canonical.display().to_string();
    let mut in_block_comment = false;
    for (line_index, line) in source.split_inclusive('\n').enumerate() {
        let line_number = line_index + 1;
        let trimmed = line.trim_start_matches([' ', '\t']);
        if !in_block_comment && is_include_directive(trimmed) {
            let include = parse_include(trimmed)
                .map_err(|message| Error::new(&format!("{file}:{line_number}:1: {message}")))?;
            let include_path = Path::new(&include);
            if include_path.is_absolute() {
                return Err(Error::new(&format!(
                    "{file}:{line_number}:1: absolute ICL include paths are not supported: {include}"
                )));
            }
            let resolved = canonical
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join(include_path);
            expand_file(&resolved, stack, output, map).map_err(|error| {
                Error::new(&format!(
                    "{file}:{line_number}:1: while including {include:?}: {}",
                    error.msg
                ))
            })?;
            if line.ends_with('\n') {
                let start = output.len();
                output.push('\n');
                map.push(start, output.len(), file.clone(), line_number, line.len());
            }
            continue;
        }
        if !in_block_comment && is_unsupported_preprocessor_directive(trimmed) {
            return Err(Error::new(&format!(
                "{file}:{line_number}:1: unsupported preprocessor directive; only #include \"relative/path.icl\" is supported"
            )));
        }
        let start = output.len();
        output.push_str(line);
        map.push(start, output.len(), file.clone(), line_number, 1);
        update_block_comment_state(line, &mut in_block_comment);
    }
    stack.pop();
    Ok(())
}

fn parse_include(line: &str) -> std::result::Result<String, String> {
    let rest = line
        .strip_prefix('#')
        .expect("caller checked preprocessor prefix")
        .trim_start();
    let rest = rest.strip_prefix("include").ok_or_else(|| {
        "unsupported preprocessor directive; only #include \"relative/path.icl\" is supported"
            .to_string()
    })?;
    if rest.starts_with(|character: char| character.is_ascii_alphanumeric() || character == '_') {
        return Err("unsupported preprocessor directive".to_string());
    }
    let rest = rest.trim_start();
    let Some(rest) = rest.strip_prefix('"') else {
        return Err("#include requires a quoted relative path".to_string());
    };
    let Some(end) = rest.find('"') else {
        return Err("unterminated quoted path in #include".to_string());
    };
    let path = &rest[..end];
    if path.is_empty() {
        return Err("#include path cannot be empty".to_string());
    }
    let trailing = rest[end + 1..].trim();
    let trailing = trailing.strip_prefix(';').unwrap_or(trailing).trim_start();
    if !(trailing.is_empty() || trailing.starts_with("//")) {
        return Err("unexpected text after #include path".to_string());
    }
    Ok(path.to_string())
}

fn reject_preprocessor_directives(source: &str, file: &str) -> Result<()> {
    let mut in_block_comment = false;
    for (index, line) in source.split_inclusive('\n').enumerate() {
        let trimmed = line.trim_start_matches([' ', '\t']);
        if !in_block_comment
            && (is_include_directive(trimmed) || is_unsupported_preprocessor_directive(trimmed))
        {
            let message = if is_include_directive(trimmed) {
                "#include requires file-based parsing; use Parser::from_file or preprocess the input"
            } else {
                "unsupported preprocessor directive; only file-based #include is supported"
            };
            return Err(Error::new(&format!("{file}:{}:1: {message}", index + 1)));
        }
        update_block_comment_state(line, &mut in_block_comment);
    }
    Ok(())
}

fn is_include_directive(line: &str) -> bool {
    line.strip_prefix('#')
        .map(str::trim_start)
        .is_some_and(|line| is_directive(line, "include"))
}

fn is_unsupported_preprocessor_directive(line: &str) -> bool {
    let Some(line) = line.strip_prefix('#').map(str::trim_start) else {
        return false;
    };
    [
        "define", "undef", "if", "ifdef", "ifndef", "elif", "else", "endif", "pragma",
    ]
    .iter()
    .any(|directive| is_directive(line, directive))
}

fn is_directive(line: &str, directive: &str) -> bool {
    line.strip_prefix(directive).is_some_and(|rest| {
        !rest.starts_with(|character: char| character.is_ascii_alphanumeric() || character == '_')
    })
}

fn update_block_comment_state(line: &str, state: &mut bool) {
    let bytes = line.as_bytes();
    let mut cursor = 0usize;
    while cursor + 1 < bytes.len() {
        if *state {
            if &bytes[cursor..cursor + 2] == b"*/" {
                *state = false;
                cursor += 2;
            } else {
                cursor += 1;
            }
        } else if &bytes[cursor..cursor + 2] == b"//" {
            break;
        } else if &bytes[cursor..cursor + 2] == b"/*" {
            *state = true;
            cursor += 2;
        } else if bytes[cursor] == b'"' {
            cursor += 1;
            while cursor < bytes.len() {
                if bytes[cursor] == b'\\' {
                    cursor += 2;
                } else if bytes[cursor] == b'"' {
                    cursor += 1;
                    break;
                } else {
                    cursor += 1;
                }
            }
        } else {
            cursor += 1;
        }
    }
}
