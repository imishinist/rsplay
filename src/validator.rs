use std::error::Error;
use std::fmt::{self, Formatter};

use reqwest::Response;

use crate::data;

#[derive(Debug)]
pub enum ValidationError {
    Status { expected: u16, got: u16 },
}

impl Error for ValidationError {
    fn description(&self) -> &str {
        match *self {
            ValidationError::Status {
                expected: _,
                got: _,
            } => "status invalid",
        }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            ValidationError::Status { expected, got } => write!(
                f,
                "validation error: status invalid: expected={}, got={}",
                expected, got
            ),
        }
    }
}

#[derive(Clone)]
pub struct Validator {
    validations: Vec<data::Validation>,
}

impl Validator {
    pub fn new(validations: Option<Vec<data::Validation>>) -> Self {
        let validations = match validations {
            Some(v) => v,
            None => Vec::new(),
        };
        Self { validations }
    }

    pub fn validate(&self, response: Response) -> Result<(), ValidationError> {
        let status = response.status();

        for v in self.validations.iter() {
            if v.status_code != status.as_u16() {
                return Err(ValidationError::Status {
                    expected: v.status_code,
                    got: status.as_u16(),
                });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::validator::ValidationError;

    #[test]
    fn error_test() {
        let table = vec![(
            ValidationError::Status {
                expected: 200,
                got: 200,
            },
            "validation error: status invalid: expected=200, got=200",
        )];

        for (err, res) in table {
            assert_eq!(format!("{}", err), res);
        }
    }
}
