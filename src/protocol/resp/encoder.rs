use std::io::Write as _;
use crate::protocol::resp::types::RESPType;

pub fn encode_resp(resp: &RESPType) -> Result<Vec<u8>, String> {
    let mut result = Vec::new();
    match resp {
        RESPType::SimpleString(s) => write!(&mut result, "+{}\r\n", s).map_err(|e| e.to_string())?,
        RESPType::Error(s) => write!(&mut result, "-{}\r\n", s).map_err(|e| e.to_string())?,
        RESPType::Integer(i) => write!(&mut result, ":{}\r\n", i).map_err(|e| e.to_string())?,
        RESPType::BulkString(Some(s)) => {
            write!(&mut result, "${}\r\n", s.len()).map_err(|e| e.to_string())?;
            result.extend_from_slice(s);
            result.push(b'\r');
            result.push(b'\n');
        }
        RESPType::BulkString(None) => {
            result.extend_from_slice(b"$-1\r\n");
        }
        RESPType::Array(arr) => {
            write!(&mut result, "*{}\r\n", arr.len()).map_err(|e| e.to_string())?;
            for item in arr {
                let encoded = encode_resp(item)?;
                result.extend_from_slice(&encoded);
            }
        }
    }
    Ok(result)
}