use crate::protocol::resp::types::RESPType;
use std::str;

// ref: https://redis.io/docs/latest/develop/reference/protocol-spec

const CRLF: &[u8] = b"\r\n";

pub fn parse_resp(input: &[u8]) -> Result<(RESPType, usize), String> {
    if input.is_empty() {
        return Err("Empty input".to_string());
    }

    match input[0] {
        b'+' => parse_simple_string(input),
        b'-' => parse_error(input),
        b':' => parse_integer(input),
        b'$' => parse_bulk_string(input),
        b'*' => parse_array(input),
        _ => Err(format!("Invalid RESP prefix: {}", input[0])),
    }
}

fn parse_simple_string(input: &[u8]) -> Result<(RESPType, usize), String> {
    let (s, len) = read_until_crlf(&input[1..])?;
    Ok((RESPType::SimpleString(s), len + 1))
}

fn parse_integer(input: &[u8]) -> Result<(RESPType, usize), String> {
    let (s, len) = read_until_crlf(&input[1..])?;
    let num = s.parse::<i64>().map_err(|e| e.to_string())?;
    Ok((RESPType::Integer(num), len + 1))
}

fn parse_bulk_string(input: &[u8]) -> Result<(RESPType, usize), String> {
    let (length_str, len1) = read_until_crlf(&input[1..])?;
    let length = length_str.parse::<i64>().map_err(|e| e.to_string())?;

    if length == -1 {
        return Ok((RESPType::BulkString(None), len1 + 1));
    }

    let start = len1 + 1;
    let end = start + length as usize;
    
    if input.len() < end + 2 {
        return Err("Incomplete bulk string".to_string());
    }

    let data = input[start..end].to_vec();
    if &input[end..end + 2] != CRLF {
        return Err("Missing CRLF".to_string());
    }

    Ok((RESPType::BulkString(Some(data)), end + 2))
}

fn parse_error(input: &[u8]) -> Result<(RESPType, usize), String> {
    let (s, len) = read_until_crlf(&input[1..])?;
    Ok((RESPType::Error(s), len + 1))
}

fn parse_array(input: &[u8]) -> Result<(RESPType, usize), String> {
    let (length_str, mut pos) = read_until_crlf(&input[1..])?;
    let length = length_str.parse::<i64>().map_err(|e| e.to_string())?;
   
    pos += 1;

    if length == -1 {
        return Ok((RESPType::Array(vec![]), pos));
    }

    let mut items = Vec::with_capacity(length as usize);
    
    for _ in 0..length {
        let (item, len) = parse_resp(&input[pos..])?;
        items.push(item);
        pos += len;
    }

    // total length is start_pos + length of all items
    Ok((RESPType::Array(items), pos))
}

fn read_until_crlf(input: &[u8]) -> Result<(String, usize), String> {
    if let Some(pos) = find_subsequence(input, CRLF) {
        let s = str::from_utf8(&input[..pos])
            .map_err(|e| e.to_string())?
            .to_string();
        Ok((s, pos + 2))
    } else {
        Err("CRLF not found".to_string())
    }
}

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len())
        .position(|window| window == needle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_string() {
        let input = b"+OK\r\n";
        let (result, len) = parse_resp(input).unwrap();
        assert_eq!(result, RESPType::SimpleString("OK".to_string()));
        assert_eq!(len, 5);
    }

    #[test]
    fn test_error() {
        let input = b"-Error message\r\n";
        let (result, len) = parse_resp(input).unwrap();
        assert_eq!(result, RESPType::Error("Error message".to_string()));
        assert_eq!(len, 16);
    }

    #[test]
    fn test_integer() {
        let input = b":1234\r\n";
        let (result, len) = parse_resp(input).unwrap();
        assert_eq!(result, RESPType::Integer(1234));
        assert_eq!(len, 7);
    }

    #[test]
    fn test_bulk_string() {
        let input = b"$5\r\nhello\r\n";
        let (result, len) = parse_resp(input).unwrap();
        assert_eq!(result, RESPType::BulkString(Some(b"hello".to_vec())));
        assert_eq!(len, 11);
    }

    #[test]
    fn test_null_bulk_string() {
        let input = b"$-1\r\n";
        let (result, len) = parse_resp(input).unwrap();
        assert_eq!(result, RESPType::BulkString(None));
        assert_eq!(len, 5);
    }

    #[test]
    fn test_array() {
        let input = b"*2\r\n$5\r\nhello\r\n$5\r\nworld\r\n";
        let (result, len) = parse_resp(input).unwrap();
        match result {
            RESPType::Array(items) => {
                assert_eq!(items.len(), 2);
                assert_eq!(items[0], RESPType::BulkString(Some(b"hello".to_vec())));
                assert_eq!(items[1], RESPType::BulkString(Some(b"world".to_vec())));
            }
            _ => panic!("Expected array"),
        }
        assert_eq!(len, 26);
    }

    #[test]
    fn test_empty_array() {
        let input = b"*0\r\n";
        let (result, len) = parse_resp(input).unwrap();
        assert_eq!(result, RESPType::Array(vec![]));
        assert_eq!(len, 4);
    }

    #[test]
    fn test_null_array() {
        let input = b"*-1\r\n";
        let (result, len) = parse_resp(input).unwrap();
        assert_eq!(result, RESPType::Array(vec![]));
        assert_eq!(len, 5);
    }
}