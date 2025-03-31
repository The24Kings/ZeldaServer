use std::io::ErrorKind;

#[derive(Debug, Clone)]
pub struct SerializeError {
    error: ErrorKind,
    message: String,
}

impl SerializeError {
    pub fn new(error: ErrorKind, message: String) -> Self {
        SerializeError {
            error,
            message
        }
    }
}

impl std::fmt::Display for SerializeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.error, self.message)
    }
}

#[derive(Debug, Clone)]
pub struct DeserializeError {
    error: ErrorKind,
    message: String,
}

impl DeserializeError {
    pub fn new(error: ErrorKind, message: String) -> Self {
        DeserializeError {
            error,
            message
        }
    }
}

impl std::fmt::Display for DeserializeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.error, self.message)
    }
}