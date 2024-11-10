use super::DataType;

#[derive(Debug, Clone)]
pub struct RedisList {
    values: Vec<String>,
}

impl RedisList {
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    pub fn push_head(&mut self, value: String) {
        self.values.insert(0, value);
    }

    pub fn push_tail(&mut self, value: String) {
        self.values.push(value);
    }

    pub fn pop_head(&mut self) -> Option<String> {
        self.values.pop()
    }

    pub fn pop_tail(&mut self) -> Option<String> {
        if self.values.is_empty() {
            None
        } else {
            Some(self.values.remove(0))
        }
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }
}

impl DataType for RedisList {
    fn type_name(&self) -> &str {
        "list"
    }

    fn memory_usage(&self) -> usize {
        self.values.iter().map(|s| s.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_list() {
        let list = RedisList::new();
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn test_push_head() {
        let mut list = RedisList::new();
        list.push_head("first".to_string());
        list.push_head("second".to_string());
        assert_eq!(list.len(), 2);
        assert_eq!(list.pop_tail(), Some("second".to_string()));
    }

    #[test]
    fn test_push_tail() {
        let mut list = RedisList::new();
        list.push_tail("first".to_string());
        list.push_tail("second".to_string());
        assert_eq!(list.len(), 2);
        assert_eq!(list.pop_head(), Some("second".to_string()));
    }

    #[test]
    fn test_pop_head_empty() {
        let mut list = RedisList::new();
        assert_eq!(list.pop_head(), None);
    }

    #[test]
    fn test_pop_tail_empty() {
        let mut list = RedisList::new();
        assert_eq!(list.pop_tail(), None);
    }

    #[test]
    fn test_multiple_operations() {
        let mut list = RedisList::new();
        list.push_head("1".to_string());
        list.push_tail("2".to_string());
        list.push_head("0".to_string());
        assert_eq!(list.len(), 3);
        assert_eq!(list.pop_tail(), Some("0".to_string()));
        assert_eq!(list.pop_head(), Some("2".to_string()));
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn test_memory_usage() {
        let mut list = RedisList::new();
        assert_eq!(list.memory_usage(), 0);
        list.push_tail("hello".to_string());
        list.push_tail("world".to_string());
        assert_eq!(list.memory_usage(), 10);
    }

    #[test]
    fn test_type_name() {
        let list = RedisList::new();
        assert_eq!(list.type_name(), "list");
    }

    #[test]
    fn test_clone() {
        let mut list1 = RedisList::new();
        list1.push_tail("test".to_string());
        let mut list2 = list1.clone();
        assert_eq!(list1.len(), list2.len());
        assert_eq!(list1.pop_head(), list2.pop_head());
    }
}