use super::DataType;

#[derive(Debug, Clone)]
pub struct RedisString {
    value: String,
}

impl RedisString {
    pub fn new(value: String) -> Self {
        Self { value }
    }

    pub fn get(&self) -> &str {
        &self.value
    }

    pub fn set(&mut self, value: String) {
        self.value = value;
    }
}

impl DataType for RedisString {
    fn type_name(&self) -> &str {
        "string"
    }

    fn memory_usage(&self) -> usize {
        self.value.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let s = RedisString::new("hello".to_string());
        assert_eq!(s.get(), "hello");
    }

    #[test]
    fn test_get() {
        let s = RedisString::new("test value".to_string());
        assert_eq!(s.get(), "test value");
    }

    #[test]
    fn test_set() {
        let mut s = RedisString::new("initial".to_string());
        s.set("updated".to_string());
        assert_eq!(s.get(), "updated");
    }

    #[test]
    fn test_type_name() {
        let s = RedisString::new("test".to_string());
        assert_eq!(s.type_name(), "string");
    }

    #[test]
    fn test_memory_usage() {
        let s = RedisString::new("hello".to_string());
        assert_eq!(s.memory_usage(), 5);

        let empty = RedisString::new("".to_string());
        assert_eq!(empty.memory_usage(), 0);
    }

    #[test]
    fn test_clone() {
        let s1 = RedisString::new("original".to_string());
        let s2 = s1.clone();
        assert_eq!(s1.get(), s2.get());
    }
}