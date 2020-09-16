use serde::{Serialize, Deserialize};
use std::fmt::Display;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Institution {
    name: String,
}

impl Display for Institution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
