use crate::TEST;

pub trait ControllerAPI {
    fn name(&self) -> String;

    fn comment(&self, message: &str) {
        TEST.push(node!(
            Comment,
            0,
            format!("{}: {}", self.name(), message).to_string()
        ));
    }
}

pub fn comment(message: &str) {
    TEST.push(node!(Comment, 0, message.to_string()));
}
