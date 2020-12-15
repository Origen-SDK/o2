#[derive(Debug)]
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

#[derive(Debug)]
pub enum PatternType {
    Main,
    Subroutine,
}

#[derive(Debug)]
pub enum PatternReferenceType {
    All,
    Origen,
    ATE,
}
