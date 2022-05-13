use crate::{TypedValueVec, TypedValueMap};

type AsNoun = String;
type AsVerb = String;

#[derive(Debug, Clone)]
pub enum OutcomeState {
    Success(AsNoun, AsVerb),
    Fail(AsNoun, AsVerb),
    Error(AsNoun, AsVerb),
}

impl OutcomeState {
    pub fn success() -> Self {
        Self::Success("Success".to_string(), "Succeeded".to_string())
    }

    pub fn pass() -> Self {
        Self::Success("Pass".to_string(), "Passed".to_string())
    }

    pub fn fail() -> Self {
        Self::Fail("Fail".to_string(), "Failed".to_string())
    }

    pub fn error() -> Self {
        Self::Error("Error".to_string(), "Errored".to_string())
    }

    pub fn as_verb(&self) -> String {
        match self {
            Self::Success(_, verb) => verb.to_string(),
            Self::Fail(_, verb) => verb.to_string(),
            Self::Error(_, verb) => verb.to_string(),
        }
    }

    pub fn as_noun(&self) -> String {
        match self {
            Self::Success(noun, _) => noun.to_string(),
            Self::Fail(noun, _) => noun.to_string(),
            Self::Error(noun, _) => noun.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Outcome {
    pub state: OutcomeState,
    pub message: Option<String>,
    pub positional_results: Option<TypedValueVec>,
    pub keyword_results: Option<TypedValueMap>,
    pub metadata: Option<TypedValueMap>,
    pub inferred: Option<bool>
}

impl std::fmt::Display for Outcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_verb())
    }
}

impl Outcome {
    pub fn new(state: OutcomeState) -> Self {
        Self {
            state,
            message: None,
            positional_results: None,
            keyword_results: None,
            metadata: None,
            inferred: None
        }
    }

    pub fn new_success() -> Self {
        Self::new(OutcomeState::success())
    }

    pub fn new_success_with_msg(message: impl std::fmt::Display) -> Self {
        let mut s = Self::new_success();
        s.set_msg(message);
        s
    }

    pub fn new_succeeded() -> Self {
        Self::new(OutcomeState::success())
    }

    pub fn new_succeeded_with_msg(message: impl std::fmt::Display) -> Self {
        let mut s = Self::new_succeeded();
        s.set_msg(message);
        s
    }

    pub fn new_pass() -> Self {
        Self::new(OutcomeState::pass())
    }

    pub fn new_passed() -> Self {
        Self::new(OutcomeState::pass())
    }

    pub fn new_fail() -> Self {
        Self::new(OutcomeState::fail())
    }

    pub fn new_failed() -> Self {
        Self::new(OutcomeState::fail())
    }

    pub fn new_err() -> Self {
        Self::new(OutcomeState::error())
    }

    pub fn new_error() -> Self {
        Self::new(OutcomeState::error())
    }

    pub fn new_errored() -> Self {
        Self::new(OutcomeState::error())
    }

    pub fn new_success_or_fail(success: bool) -> Self {
        if success {
            Self::new_success()
        } else {
            Self::new_fail()
        }
    }

    pub fn new_pass_or_fail(pass: bool) -> Self {
        if pass {
            Self::new_pass()
        } else {
            Self::new_fail()
        }
    }

    pub fn succeeded(&self) -> bool {
        match self.state {
            OutcomeState::Success(_, _) => true,
            _ => false,
        }
    }

    pub fn passed(&self) -> bool {
        self.succeeded()
    }

    pub fn failed(&self) -> bool {
        match self.state {
            OutcomeState::Fail(_, _) => true,
            _ => false,
        }
    }

    pub fn errored(&self) -> bool {
        match self.state {
            OutcomeState::Error(_, _) => true,
            _ => false,
        }
    }

    pub fn set_msg(&mut self, message: impl std::fmt::Display) -> &mut Self {
        self.message = Some(message.to_string());
        self
    }

    pub fn message(&self) -> &Option<String> {
        &self.message
    }

    pub fn msg(&self) -> &Option<String> {
        self.message()
    }

    pub fn msg_or_default(&self) -> &str {
        match &self.message {
            Some(m) => m,
            None => "No message was provided!"
        }
    }


    // pub fn add_metadata(&mut self, key: &str, m: Metadata) -> Result<&mut Self> {
    //     if self.metadata.is_none() {
    //         self.metadata = Some(IndexMap::new());
    //     }

    //     self.metadata.as_mut().unwrap().insert(key.to_string(), m);
    //     Ok(self)
    // }

    pub fn gist(&self) {
        match &self.state {
            OutcomeState::Success(_, _) => {
                display_greenln!("{}", self.as_verb());
            }
            OutcomeState::Fail(_, _) => {
                display_redln!("{}", self.as_verb());
            }
            OutcomeState::Error(_, _) => {
                display_redln!("{}", self.as_verb());
            }
        }
    }

    pub fn summarize_and_exit(&self) {
        match &self.state {
            OutcomeState::Success(n, _) => {
                display_greenln!("{}", self.as_verb());
                if n == "Pass" {
                    exit_pass!();
                } else {
                    exit_success!();
                }
            }
            OutcomeState::Fail(_, _) => {
                display_redln!("{}", self.as_verb());
                exit_fail!();
            }
            OutcomeState::Error(_, _) => {
                display_redln!("{}", self.as_verb());
                exit_error!();
            }
        }
    }

    pub fn as_verb(&self) -> String {
        if let Some(m) = self.message.as_ref() {
            format!("{} with message: {}", self.state.as_verb(), m)
        } else {
            format!("{}", self.state.as_verb())
        }
    }
}
