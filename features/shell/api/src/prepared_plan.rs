use crate::Command;

pub struct PreparedPlan {
    pub commands: Vec<Command>,
    pub autocomplete: bool,
}
