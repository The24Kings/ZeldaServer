#[derive(Debug, Clone)]
pub struct SerializeError {
    message: String,
}

#[derive(Debug, Clone)]
pub struct DeserializeError {
    message: String,
}

impl std::fmt::Display for SerializeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error serializing message: {}", self.message)
    }
}

impl SerializeError {
    pub fn new() -> Self {
        SerializeError {
            message: String::from("Serialization error"),
        }
    }

    pub fn with_message(message: &str) -> Self {
        SerializeError {
            message: String::from(message),
        }
    }
}

impl std::fmt::Display for DeserializeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error deserializing message: {}", self.message)
    }
}

impl DeserializeError {
    pub fn new() -> Self {
        DeserializeError {
            message: String::from("Deserialization error"),
        }
    }

    pub fn with_message(message: &str) -> Self {
        DeserializeError {
            message: String::from(message),
        }
    }
}
