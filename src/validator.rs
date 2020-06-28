use serde::{Deserialize, Serialize};

pub struct Validator {
    validation: Vec<Definition>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Definition {
    name: String,
    status_code: u32,
}
