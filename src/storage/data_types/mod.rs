pub mod string;
pub mod list;

pub trait DataType {
    fn type_name(&self) -> &str;
    fn memory_usage(&self) -> usize;
}