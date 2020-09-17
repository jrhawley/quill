use serde::{Serialize, Deserialize};
use std::fmt::Display;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Institution {
    name: String,
}

impl Institution {
    // Return the name of the institution
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Display for Institution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
