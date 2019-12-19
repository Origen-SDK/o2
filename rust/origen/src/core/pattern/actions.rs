//! Collects actions
pub use super::*;

pub struct Actions {
    list: Vec<action::Action>
}

impl Actions {
    pub fn new() -> Actions {
        Actions {
            list: Vec::new(),
        }
    }
    
    pub fn push(& mut self, pa: action::Action) {
        self.list.push(pa);
    }
    
    pub fn vec(&self) -> &Vec<action::Action> {
        &self.list
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn actions_struct_loads_reads() {
        let mut pattern_actions = Actions::new();
        pattern_actions.push(action::Action::Timeset("tp0".to_string()));
        pattern_actions.push(action::Action::Pin(pinaction::PinAction::new("pa0", "1", operation::Operation::Read)));
        assert_eq!((pattern_actions.vec()[0]).to_string(), "Timeset -> tp0");
        assert_eq!((pattern_actions.vec()[1]).to_string(), "PinAction -> pin: pa0, data: 1, operation: read");
    }
}