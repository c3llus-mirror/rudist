use std::fmt;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RESPType {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(Option<Vec<u8>>),
    Array(Vec<RESPType>),
}

impl RESPType {
    pub fn is_bulk(&self) -> bool {
        matches!(self, RESPType::BulkString(_))
    }

    pub fn is_array(&self) -> bool {
        matches!(self, RESPType::Array(_))
    }

    pub fn as_bytes(&self) -> Result<&[u8], String> {
        match self {
            RESPType::BulkString(Some(bytes)) => Ok(bytes),
            RESPType::BulkString(None) => Err("bulk string is none".to_string()),
            _ => Err("not a bulk string".to_string())
        }
    }

    pub fn parse(input: &[u8]) -> Result<Self, String> {
        if input.is_empty() {
            return Err("empty input".to_string());
        }

        match input[0] as char {
            '*' => {
                // parse array
                let mut parts = Vec::new();
                let mut pos = 0;
                
                // skip first line (*3\r\n)
                while pos < input.len() && input[pos] != b'\n' { pos += 1; }
                pos += 1;

                // parse each bulk string
                while pos < input.len() {
                    if input[pos] as char != '$' {
                        return Err("expected bulk string".to_string());
                    }
                    
                    // skip length
                    while pos < input.len() && input[pos] != b'\n' { pos += 1; }
                    pos += 1;

                    // read until \r\n
                    let mut value = Vec::new();
                    while pos < input.len() && input[pos] != b'\r' {
                        value.push(input[pos]);
                        pos += 1;
                    }
                    parts.push(RESPType::BulkString(Some(value)));
                    pos += 2; // skip \r\n
                }
                
                Ok(RESPType::Array(parts))
            },
            // ...
            _ => Err("invalid resp format".to_string())
        }
    }

    pub fn encode(&self) -> String {

        match self {
            RESPType::SimpleString(s) => format!("+{}\r\n", s),
            RESPType::Error(e) => format!("-{}\r\n", e),
            RESPType::Integer(i) => format!(":{}\r\n", i),
            RESPType::BulkString(Some(bytes)) => format!("${}\r\n{}\r\n", bytes.len(), String::from_utf8_lossy(bytes)),
            RESPType::BulkString(None) => "$-1\r\n".to_string(),
            RESPType::Array(array) => {
                let mut encoded = format!("*{}\r\n", array.len());
                for item in array {
                    encoded.push_str(&item.encode());
                }
                encoded
            }
        }
    }
}

impl fmt::Display for RESPType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RESPType::SimpleString(s) => write!(f, "+{}", s),
            RESPType::Error(s) => write!(f, "-{}", s),
            RESPType::Integer(i) => write!(f, ":{})", i),
            RESPType::BulkString(Some(v)) => write!(f, "${}\r\n{:?}\r\n", v.len(), v),
            RESPType::BulkString(None) => write!(f, "$-1\r\n"), // null bulk string
            RESPType::Array(arr) => {
                write!(f, "*{}", arr.len())?;
                for item in arr {
                    write!(f, "\r\n{}", item)?;
                }
                Ok(())
            },
        }
    }
}
