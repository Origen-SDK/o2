#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Pattern {
    pub pattern_type: PatternType,
    pub reference_type: PatternReferenceType,
    pub path: String,
}

impl Pattern {
    pub fn new(
        path: String,
        pattern_type: Option<PatternType>,
        reference_type: Option<PatternReferenceType>,
    ) -> Pattern {
        Pattern {
            path: path,
            pattern_type: match pattern_type {
                None => PatternType::Main,
                Some(t) => t,
            },
            reference_type: match reference_type {
                None => PatternReferenceType::All,
                Some(t) => t,
            },
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum PatternType {
    Main,
    Subroutine,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum PatternReferenceType {
    All,
    Origen,
    ATE,
}
